use actix_web::web;
use entity::user::{self, ActiveModel as UserModel, Entity as UserEntity};
use crate::AuthResult;
use crate::error::router_error::RouterError;
use sea_orm::{EntityTrait, ColumnTrait, QueryFilter, DatabaseConnection};
use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
pub struct UserProfile {
    username: String,
    email: String,
}

pub async fn get_profile(
    db_conn: web::Data<DatabaseConnection>,
    data: web::ReqData<AuthResult>
) -> Result<web::Json<UserProfile>, RouterError> {
    use crate::error::router_error::RouterError::*;

    let conn = db_conn.get_ref();
    let user = data.into_inner();

    // check if user exists or we must create a new user?
    let Ok(Some(user)) = UserEntity::find()
        .filter(user::Column::Id.eq(user.user_id))
        .one(conn).await else {
            return Err(InternalError);
        };

    Ok(web::Json(UserProfile {
        username: user.name,
        email: user.email
    }))
}
