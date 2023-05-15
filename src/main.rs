use std::env;
use std::io;
use std::println;
use std::vec;

mod core_routers;
mod email;
mod error;
mod middlewares;

pub use middlewares::token_checker::AuthResult;
use plugin_manager;
use plugin_manager::config::PluginConfig;
use plugin_manager::manager::Plugin;
use plugin_manager::manager::PluginBuilder;
use plugin_manager::manager::PluginManager;
use plugin_manager::manager::PluginMetadata;

use crate::email::email::EmailManager;
use core_routers::account::{get_token, profile, send_verification, verify};
use middlewares::token_checker::TokenValidator;

use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use auth::token::TokenAuth;
use dotenvy::dotenv;
use lettre::transport::smtp::authentication::Credentials;
use migration::{Migrator, MigratorTrait};
use sea_orm::{Database, DatabaseConnection, DbErr};

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

// THIS IS JUST FOR TEST
fn init_hello_world(p_manager: &mut PluginManager<PluginBuilder>) {
    let inc = include_str!("../builtin_plugins/hello_world/hello_world.wasm");

    let pg = PluginBuilder::new(
        PluginMetadata {
            name: "hello-world".to_string(),
            version: "0.0.1".to_string(),
        },
        inc.to_string(),
    );

    p_manager.add(pg);
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

    let mut p_manager: PluginManager<PluginBuilder> = PluginManager::new();

    init_hello_world(&mut p_manager);

    HttpServer::new(move || {
        // Set All to the cors
        let cors = Cors::permissive();

        App::new()
            .wrap(cors)
            .app_data(web::Data::new(database_conn.clone()))
            .app_data(web::Data::new(emailer.clone()))
            .service(
                web::scope("/account")
                    .route(
                        "/send_verification",
                        web::post().to(send_verification::send_verification_email),
                    )
                    .route("/verify/{hash}", web::get().to(verify::verify))
                    .route(
                        "/get_token/{verification_id}",
                        web::get().to(get_token::get_token),
                    )
                    .route(
                        "/profile",
                        web::get().to(profile::get_profile).wrap(token_auth.clone()),
                    ),
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
