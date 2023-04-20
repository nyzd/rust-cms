use actix_web::web;
use sea_orm::DatabaseConnection;
use crate::error::router_error::RouterError;

pub async fn signin(db_conn: web::Data<DatabaseConnection>) -> Result<String, RouterError> {
    Ok("Success".to_string())
}
