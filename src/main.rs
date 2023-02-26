use actix_web::{web, App, HttpResponse, HttpServer, Responder};



mod db;

// use std::sync::Mutex;

use sailfish::TemplateOnce;

use crate::db::service::{Service, self};




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

#[actix_web::main]
async fn main() {

    service::Service::addProfile().await;

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
