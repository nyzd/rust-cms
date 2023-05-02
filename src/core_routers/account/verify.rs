use super::{current_time_stamp, generate_uuid};
use actix_web::web;
use sea_orm::{ActiveModelTrait, EntityTrait, DatabaseConnection, ColumnTrait, QueryFilter};
use sea_orm::ActiveValue::Set;
use crate::error::router_error::RouterError;
use entity::email_verification;
use entity::email_verification::{Entity as EmailVerificationEntitiy, ActiveModel as ActiveVerificationcode};
use entity::token::{ActiveModel as TokenModel};
use entity::user::{self, ActiveModel as UserModel, Entity as UserEntity};
use auth::token::TokenGenerator;
use hash::{random_bytes, random_string};

const RANDOM_USERNAME_LENGTH: usize = 8;

/// Get the code from the user(in query)
/// and check if its exists
/// in the email_verification table
/// and after that create a valid token for user and,
/// return it in the response data
pub async fn verify<'a>(
    db_conn: web::Data<DatabaseConnection>,
    query: web::Path<String>
) -> Result<&'a str, RouterError> {
    use crate::error::router_error::RouterError::*;

    let req_code = query.into_inner();
    let conn = db_conn.get_ref();

    // Get the verification with code

    let Ok(Some(verification)) = EmailVerificationEntitiy::find()
        .filter(email_verification::Column::VerificationHash.eq(req_code))
        .one(conn).await else {
            return Err(NotFound("Verification Code not found".to_string()));
        };

    // Check if the verification code is not expired or
    // Its been to long when verification code exists
    if verification.verified {
        return Err(Expired("This Verification code is already verified".to_string()));
    } else {
        // Now we must expire the verification code
        // we used it
        let mut verification: ActiveVerificationcode = verification.clone().into();

        verification.verified = Set(true);
        let _ = verification.update(conn).await else {
            return Err(InternalError);
        };
    }

    let current_time = current_time_stamp();

    // Is the verification code expired
    if current_time as i64 - verification.clone().created_at.timestamp() >= 70 {
        return Err(Expired("This Verification code is expired".to_string()));
    }

    // check if user exists or we must create a new user?
    let user = UserEntity::find()
        .filter(user::Column::Email.eq(verification.clone().email))
        .one(conn).await.unwrap();

    // random chars for username
    let random_username = random_string(RANDOM_USERNAME_LENGTH);
    let uuid = generate_uuid();

    if user == None {
        let new_user = UserModel {
            name: Set(format!("u{}", random_username)),
            email: Set(verification.clone().email),
            uu_id: Set(uuid),
            ..Default::default()
        };

        new_user.insert(conn).await.unwrap();
    };

    Ok("Your account has been verifed")
}
