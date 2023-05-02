use std::env;
use std::io;

mod core_routers;
mod error;
mod middlewares;
mod email;

use core_routers::account::{verify, send_verification, get_token};
use crate::email::email::EmailManager;
use middlewares::token_checker::TokenValidator;

use actix_web::{web, App, HttpServer, dev::ServiceRequest, Error};
use auth::token::TokenAuth;
use dotenvy::dotenv;
use migration::{Migrator, MigratorTrait};
use sea_orm::{Database, DatabaseConnection, DbErr};
use lettre::transport::smtp::authentication::Credentials;

pub fn create_emailer() -> EmailManager {
    dotenv().ok();

    let host = env::var("SMTP_HOST").expect("SMTP_HOST must be set");
    let port = env::var("SMTP_PORT").expect("SMTP_PORT must be set");
    let from = env::var("SMTP_FROM").expect("SMTP_FROM must be set");
    let username = env::var("SMTP_USERNAME").expect("SMTP_USERNAME must be set");
    let password = env::var("SMTP_PASSWORD").expect("SMTP_PASSWORD must be set");

    let credentials = Credentials::new(username, password);

    EmailManager::new(&host, port.parse().unwrap(), credentials, from)
        .expect("Cant create EmailManager")
}

async fn establish_db_connection() -> Result<DatabaseConnection, DbErr> {
    dotenv().ok();

    let db_url_env = env::var("DATABASE_URL").expect("DATABASE_URL is not set in the .env file");

    let db: DatabaseConnection = Database::connect(db_url_env).await?;

    Ok(db)
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    let database_conn = establish_db_connection()
        .await
        .expect("Cant Connect To the database");

    Migrator::up(&database_conn, None)
        .await
        .expect("Cant run the migrations");

    let emailer = create_emailer();
    let token_validator = TokenValidator::new(database_conn.clone());
    let token_auth = TokenAuth::new(token_validator.clone());

    HttpServer::new(move || 
            App::new()
            .app_data(web::Data::new(database_conn.clone()))
            .app_data(web::Data::new(emailer.clone()))
            .service(
                web::scope("/account")
                    .route("/send_verification", web::post().to(send_verification::send_verification_email))
                    .route("/verify/{hash}", web::get().to(verify::verify))
                    .route("/get_token/{verification_id}", web::get().to(get_token::get_token))
        )                            )
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
