use crate::error::router_error::RouterError;
use actix_web::web;
use entity::email_verification::{self, ActiveModel as EmailVerificationModel, Entity as EmailVerificationEntitiy};
use sea_orm::ActiveValue::Set;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, InsertResult};
use hash::{random_bytes, hash_bytes};
use serde::Deserialize;

/// Creates the verification code in debug mode
/// return is the url of verification
pub async fn create_verification_url_debug(
    db_conn: &DatabaseConnection,
    user_email: String
) -> Result<String, String> {
    // Create the new verification code with email
    // TODO: expire the last verification code if exists
    //
    // Hash the random bytes
    let r_bytes = random_bytes();
    let hash = hash_bytes(r_bytes);

    let new_verification = EmailVerificationModel {
        email: Set(user_email),
        verification_hash: Set(hash.clone()),
        ..Default::default()
    };

    let res: InsertResult<EmailVerificationModel> = EmailVerificationEntitiy::insert(new_verification)
        .exec(db_conn)
        .await
        .unwrap();

    Ok(format!("/verify?code={}", hash))
}

#[derive(Deserialize)]
pub struct VerificationUserInfo {
    email: String,
}

pub async fn send_verification_email(
    user_info: web::Json<VerificationUserInfo>,
    db_conn: web::Data<DatabaseConnection>
) -> Result<String, RouterError> {
    // First we check if this build is the debug build
    // if it is then send the url in the response
    // only for test use cases

    let user_info = user_info.into_inner();

    let url = if cfg!(debug_assertions) {
        create_verification_url_debug(db_conn.get_ref(), user_info.email).await.unwrap()
    } else {
        String::from("cant")
    };

    Ok(url)
}

