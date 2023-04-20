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
use std::time::{SystemTime, UNIX_EPOCH};

const RANDOM_USERNAME_LENGTH: usize = 8;

fn current_time_stamp() -> f64 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Cant get timestamp");

    since_the_epoch.as_secs_f64()
}

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
    if verification.expired {
        return Err(Expired("This Verification code is expired".to_string()));
    } else {
        // Now we must expire the verification code
        // we used it
        let mut verification: ActiveVerificationcode = verification.clone().into();

        verification.expired = Set(true);
        let _ = verification.update(conn).await else {
            return Err(InternalError);
        };
    }

    let current_time = current_time_stamp();

    if current_time as i64 - verification.clone().created_at.timestamp() >= 70 {
        return Err(Expired("This Verification code is expired".to_string()));
    }

    // Now create the token for the user
    // Some salts
    let bytes = random_bytes().to_vec();

    let mut token_generator = TokenGenerator::new(&bytes);
    token_generator.generate();

    let Some(result) = token_generator.get_result() else {
        return Err(InternalError);
    };

    // check if user exists or we must create a new user?
    let Ok(user) = UserEntity::find()
        .filter(user::Column::Email.eq(verification.clone().email))
        .one(conn).await else {
            return Err(InternalError);
        };

    // random chars for username
    let random_username = random_string(RANDOM_USERNAME_LENGTH);

    let user = if user == None {
        let new_user = UserModel {
            name: Set(format!("u{}", random_username)),
            email: Set(verification.email),
            ..Default::default()
        };

        new_user.insert(conn).await.unwrap()
    } else {
        // This unwrap is safe
        user.unwrap()
    };

    let token_hash = {
        let result_bytes = result.as_bytes().to_vec();
        token_generator.set_source(&result_bytes);
        token_generator.generate();
        token_generator.get_result().unwrap()
    };

    // Now create token
    let new_token = TokenModel {
        user_id: Set(user.id),
        token_hash: Set(token_hash),
        ..Default::default()
    };

    new_token.insert(conn).await.unwrap();

    Ok("Verifyed")
}
