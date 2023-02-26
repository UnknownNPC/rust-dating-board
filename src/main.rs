use actix_web::{web, App, HttpResponse, HttpServer, Responder};


use sea_orm::{Database, DbConn, DbErr};

mod db;

// use std::sync::Mutex;

use sailfish::TemplateOnce;

use crate::db::DbProvider;

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

async fn establish_connection() -> Result<DbConn, DbErr> {
    let database_url = std::env::var("DATABASE_URL").unwrap();
    Database::connect(&database_url).await
}

#[actix_web::main]
async fn main() {
    let db_con = establish_connection().await.unwrap();
    let provider = DbProvider::new(&db_con);


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
