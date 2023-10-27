use std::env;
use std::io;
use std::io::Read;

mod core_routers;
mod email;
mod error;
mod middlewares;

pub use middlewares::token_checker::AuthResult;
use plugin_manager;
use plugin_manager::config::PluginConfig;
use plugin_manager::manager::PluginSystem;
use plugin_manager::manager::{PluginBuilder, PluginSystemReader, PluginSystemWriter};

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

pub fn init_plugin_system() -> (
    PluginSystemWriter<PluginBuilder>,
    PluginSystemReader<PluginBuilder>,
) {
    let (write, read) = PluginSystem::get_left_right();
    let (w, r) = (
        PluginSystemWriter(write),
        // We use here the factory
        // for multi thread share
        PluginSystemReader(read.factory()),
    );

    return (w, r);
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    let database_conn = establish_db_connection()
        .await
        .expect("Can't Connect To the database");

    Migrator::up(&database_conn, None)
        .await
        .expect("Can't run the migrations");

    let emailer = create_emailer();
    let token_validator = TokenValidator::new(database_conn.clone());
    let token_auth = TokenAuth::new(token_validator.clone());

    let (mut w, r) = init_plugin_system();
    let inc = include_bytes!("../builtin_plugins/hello-world/hello.wasm");
    let p_conf = PluginConfig::try_from(
        include_bytes!("../builtin_plugins/hello-world/config.json").to_vec(),
    )
    .unwrap();

    w.add_from_config(inc.bytes().map(|b| b.unwrap()).collect::<Vec<u8>>(), p_conf)
        .unwrap();

    w.publish();

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

#[cfg(test)]
mod tests {
    use plugin_manager::manager::{Plugin, PluginBuilder, PluginMetadata};

    #[test]
    fn plugin_test() {
        let inc = include_bytes!("../builtin_plugins/hello-world/hello.wasm");
        let plugin = PluginBuilder::new(
            PluginMetadata {
                ..Default::default()
            },
            inc.to_vec(),
        );

        let mut wasm = plugin.build().unwrap();
        let func = wasm.instance.exports.get_function("hello").unwrap();
        let memory = wasm.instance.exports.get_memory("memory").unwrap();

        let memview = memory.view(&wasm.store);

        let mut buf: Vec<u8> = memview.copy_to_vec().unwrap();
        buf.retain(|i| i != &0u8);

        let _res = match func.call(&mut wasm.store, &[]).unwrap().get(0).unwrap() {
            plugin_manager::wasmer::Value::I32(n) => n,

            _ => panic!("Expected i32"),
        };

        let mut string = buf.into_iter();
        while let Some(data) = string.next() {
            print!("{}", char::from(data));
        }
    }
}
