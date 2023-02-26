use actix_web::{web, App, HttpResponse, HttpServer, Responder};


use sea_orm::{Database, DbConn, DbErr};

mod db;
mod config;

// use std::sync::Mutex;

use sailfish::TemplateOnce;

use crate::{db::DbProvider, config::Config};

// use crate::db::db_provider::{self, Service};

#[derive(TemplateOnce)]
#[template(path = "home.stpl")]
struct Home<'a> {
    head_title: &'a str,
}

async fn homepage() -> impl Responder {
    HttpResponse::Ok().body(
        Home {
            head_title: "hello",
        }
        .render_once()
        .unwrap(),
    )
}

async fn establish_connection(conf: &Config) -> Result<DbConn, DbErr> {
    Database::connect(&conf.database_url).await
}

#[actix_web::main]
async fn main() {
    let conf = Config::init();
    let db_con = establish_connection(&conf).await.unwrap();
    let provider = DbProvider::new(&db_con);
    let s = provider.add_profile().await;


    let addr = "localhost:8080";
    // let db = web::Data::new(Mutex::new(Db::new()));
    let server = HttpServer::new(move || {
        App::new()
            // .app_data(db.clone())
            .route("/", web::get().to(homepage))
    })
    .bind(addr)
    .unwrap()
    .run();
    println!("Server live at http://{}", addr);
    server.await.unwrap();
}
