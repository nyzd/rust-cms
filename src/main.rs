use std::env;
use std::io;

mod core_routers;
mod error;
mod middlewares;

use core_routers::account::{verify, send_verification};

use actix_web::{web, App, HttpServer};
use dotenvy::dotenv;
use middlewares::token_checker::TokenValidator;
use migration::{Migrator, MigratorTrait};
use sea_orm::{Database, DatabaseConnection, DbErr};

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

    let token_checker_obj = TokenValidator::new(database_conn.clone());

    HttpServer::new(move || 
            App::new()
            .app_data(web::Data::new(database_conn.clone()))
            .service(
                web::scope("/account")
                     .route("/verify/{hash}", web::get().to(verify::verify))
                     .route("/send_verification", web::post().to(send_verification::send_verification_email))
            )
        )
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
