use hash::random_bytes;
use actix_web::web;
use crate::error::router_error::RouterError;
use super::current_time_stamp;
use sea_orm::ActiveValue::Set;
use entity::email_verification;
use entity::email_verification::{Entity as EmailVerificationEntitiy, ActiveModel as ActiveVerificationcode};
use entity::token::{ActiveModel as TokenModel};
use sea_orm::{ActiveModelTrait, EntityTrait, DatabaseConnection, ColumnTrait, QueryFilter};
use entity::user::{self, ActiveModel as UserModel, Entity as UserEntity};
use auth::token::TokenGenerator;

/// Creates a token and returns it
/// if the verification id is correct
pub async fn get_token(
    db_conn: web::Data<DatabaseConnection>,
    query: web::Path<String>
) -> Result<String, RouterError> {
    use crate::error::router_error::RouterError::*;

    let verification_id = query.into_inner();
    let conn = db_conn.get_ref();

    // Get the verification if it exists
    let Ok(Some(verification)) = EmailVerificationEntitiy::find()
        .filter(email_verification::Column::UuId.eq(verification_id))
        .one(conn).await else {
            return Err(NotFound("Verification with this id not found".to_string()));
        };

    // Get the user
    let Ok(Some(user)) = UserEntity::find()
        .filter(user::Column::Email.eq(verification.clone().email))
        .one(conn).await else {
            return Err(InternalError);
        };

    if !verification.verified {
        return Err(Auth("This verification is not verifyed".to_string()));
    }

    // Check if the verification code is not expired or
    // Its been to long when verification code exists
    if verification.used {
        return Err(Used("This Verification code is already used".to_string()));
    }

    let current_time = current_time_stamp();

    // Is verification code is expired
    if current_time as i64 - verification.clone().created_at.timestamp() >= 70 {
        return Err(Expired("This Verification code is expired".to_string()));
    }

    // Now we must expire the verification code
    // we used it
    let mut verification_clone: ActiveVerificationcode = verification.clone().into();

    verification_clone.used = Set(true);
    let _ = verification_clone.update(conn).await else {
        return Err(InternalError);
    };

    // Now we must create a token and return it
    // Some salts
    let bytes = random_bytes().to_vec();

    let mut token_generator = TokenGenerator::new(&bytes);
    token_generator.generate();

    // We must return this resut in response
    let Some(result) = token_generator.get_result() else {
        return Err(InternalError);
    };

    // This token will just put in database
    // do not return this to the user
    let token_hash = {
        let result_bytes = result.clone().as_bytes().to_vec();
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

    return Ok(result);
}
