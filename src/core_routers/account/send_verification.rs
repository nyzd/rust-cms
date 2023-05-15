use crate::error::router_error::RouterError;
use actix_web::web;
use entity::email_verification::{self, ActiveModel as EmailVerificationModel, Entity as EmailVerificationEntitiy, Model as VerificationModel};
use sea_orm::ActiveValue::Set;
use sea_orm::{DatabaseConnection, EntityTrait, TryIntoModel, ActiveModelTrait, QueryOrder, QuerySelect};
use hash::{random_bytes, hash_bytes};
use serde::Deserialize;
use crate::EmailManager;
use std::env;
use super::{generate_uuid, current_time_stamp};

#[derive(Clone, Debug)]
struct VerificationInfo {
    hash: String,
    uuid: String,
}

async fn new_verification(
    db_conn: &DatabaseConnection,
    user_email: &String
) -> Result<VerificationInfo, RouterError> {
    // Hash the random bytes
    let r_bytes = random_bytes();
    let hash = hash_bytes(r_bytes);
    let uuid = generate_uuid();

    let new_verification = EmailVerificationModel {
        email: Set(user_email.to_string()),
        verification_hash: Set(hash.clone()),
        verified: Set(false),
        used: Set(false),
        uu_id: Set(uuid.clone()),
        ..Default::default()
    };

    let Ok(res) = new_verification.insert(db_conn).await else {
            return Err(RouterError::InternalError);
        };

    Ok(VerificationInfo {
        hash,
        uuid,
    })
}

/// Creates the verification code in debug mode
/// return is the url of verification
pub async fn create_verification_url_debug(
    db_conn: &DatabaseConnection,
    user_email: String
) -> Result<String, RouterError> {
    let new_verification_info = new_verification(&db_conn, &user_email).await?;

    Ok(format!("/verify/{}", new_verification_info.hash))
}

pub async fn send_verification_url(
    emailer: web::Data<EmailManager>,
    db_conn: &DatabaseConnection,
    user_email: String
) -> Result<String, RouterError> {
    use crate::error::router_error::RouterError::*;

    let Ok(last_verification) = EmailVerificationEntitiy::find()
        .order_by_desc(email_verification::Column::CreatedAt)
        .limit(1)
        .one(db_conn).await else {
            return Err(InternalError);
        };

    let current_time = current_time_stamp();

    if last_verification != None {
        if current_time as i64 - last_verification.unwrap().created_at.timestamp() <= 20 {
            return Err(Used("Verification is already sended please wait and try again".to_string()));
        }
    }

    let new_verification_info = new_verification(&db_conn, &user_email).await?;

    // Now send email to user 
    // but first we must get the api url in order to
    // create the validation link
    let api_url = env::var("API_URL").expect("API_URL must be set");
    let verification_link = format!("{}/account/verify/{}", api_url, new_verification_info.hash);
    let body = format!(r#"<html><body><a href="{}">Click to verify you email</a></body></html>"#, verification_link);

    let Ok(new_email) = emailer.send_email(
        &user_email,
        "Verification Link",
        body
    ).await else {
        return Err(InternalError);
    };

    Ok(new_verification_info.uuid)
}

#[derive(Deserialize)]
pub struct VerificationUserInfo {
    email: String,
}

pub async fn send_verification_email(
    user_info: web::Json<VerificationUserInfo>,
    emailer: web::Data<EmailManager>,
    db_conn: web::Data<DatabaseConnection>
) -> Result<String, RouterError> {
    // First we check if this build is the debug build
    // if it is then send the url in the response
    // only for test use cases

    let user_info = user_info.into_inner();

    let result = if cfg!(debug_assertions) {
        create_verification_url_debug(db_conn.get_ref(), user_info.email).await?
    } else {
        send_verification_url(emailer, db_conn.get_ref(), user_info.email).await?
    };

    Ok(result)
}

