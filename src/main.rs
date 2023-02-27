use actix_web::{web, App, HttpServer};

use sea_orm::{Database, DbConn, DbErr};

mod config;
mod db;
mod web_api;

// use std::sync::Mutex;

use crate::{config::Config, db::DbProvider};

// use crate::db::db_provider::{self, Service};

async fn establish_connection(conf: &Config) -> Result<DbConn, DbErr> {
    Database::connect(&conf.database_url).await
}

#[actix_web::main]
async fn main() {
    let conf = Config::init();
    let db_con = establish_connection(&conf).await.unwrap();
    let provider = DbProvider::new(db_con);


    let addr = "localhost:8000";
    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(provider.clone()))
            .app_data(web::Data::new(conf.clone()))
            .route("/", web::get().to(web_api::homepage_endpoint))
            .route("/sign_in/google", web::post().to(web_api::google_sign_in_endpoint))
    })
    .bind(addr)
    .unwrap()
    .run();
    println!("Server live at http://{}", addr);
    server.await.unwrap();
}
