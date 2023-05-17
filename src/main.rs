use std::env;
use std::io;

mod core_routers;
mod email;
mod error;
mod middlewares;

pub use middlewares::token_checker::AuthResult;
use plugin_manager;
use plugin_manager::manager::PluginSystem;
use plugin_manager::manager::{
    PluginBuilder, PluginMetadata, PluginSystemReader, PluginSystemWriter,
};

use crate::email::email::EmailManager;
use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use auth::token::TokenAuth;
use core_routers::account::{get_token, profile, send_verification, verify};
use core_routers::plugin::run_plugin;
use dotenvy::dotenv;
use lettre::transport::smtp::authentication::Credentials;
use middlewares::token_checker::TokenValidator;
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
fn init_hello_world(s_writer: &mut PluginSystemWriter<PluginBuilder>) {
    let inc = include_str!("../builtin_plugins/hello_world/hello_world.wasm");

    let pg = PluginBuilder::new(
        PluginMetadata {
            name: "hello-world".to_string(),
            version: "0.0.1".to_string(),
        },
        inc.to_string(),
    );

    s_writer.add(pg);
    s_writer.publish();
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
    let (write, read) = PluginSystem::get_left_right();
    let (mut w, r) = (
        PluginSystemWriter(write),
        // We use here the factory
        // for multi thread share
        PluginSystemReader(read.factory()),
    );
    init_hello_world(&mut w);

    // Create the data
    let data: web::Data<PluginSystemReader<PluginBuilder>> = web::Data::new(r);

    HttpServer::new(move || {
        // Set All to the cors
        let cors = Cors::permissive();

        App::new()
            .wrap(cors)
            .app_data(web::Data::new(database_conn.clone()))
            .app_data(web::Data::new(emailer.clone()))
            .app_data(data.clone())
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
            .service(
                web::scope("/plugin")
                    .route("/call", web::post().to(run_plugin::run_plugin_function)),
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
