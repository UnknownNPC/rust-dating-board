use std::env;

use actix_files::Files;
use actix_web::{middleware, web, App, HttpServer};
use env_logger::Builder;
use sea_orm::{Database, DbConn, DbErr};
use std::fs;

mod config;
mod db;
mod web_api;

use log::info;

rust_i18n::i18n!("locales");

use crate::{config::Config, db::DbProvider};

async fn establish_connection(conf: &Config) -> Result<DbConn, DbErr> {
    Database::connect(&conf.database_url).await
}

#[actix_web::main]
async fn main() {
    Builder::new()
        .filter_level(log::LevelFilter::Info)
        .filter_module("sqlx::query", log::LevelFilter::Off)
        .init();

    rust_i18n::set_locale("uk");

    let conf = Config::init();
    let db_con = establish_connection(&conf).await.unwrap();
    let provider = DbProvider::new(db_con);

    let port = std::env::var("PORT").unwrap_or("8080".to_string());
    let addr = format!("localhost:{}", &port);

    let all_photos_os_folder = all_photos_folder_path(&conf);

    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(provider.clone()))
            .app_data(web::Data::new(conf.clone()))
            .route("/", web::get().to(web_api::index_page))
            .route("/404", web::get().to(web_api::p404_page))
            .route("/add_profile", web::get().to(web_api::add_profile_page))
            .route("/edit_profile", web::get().to(web_api::edit_profile_page))
            .route("/view_profile", web::get().to(web_api::view_profile_page))
            .route("/sitemap.xml", web::get().to(web_api::sitemap))
            .route(
                "/add_or_edit_profile",
                web::post().to(web_api::add_or_edit_profile_post),
            )
            .route("/comment/add", web::post().to(web_api::add_comment))
            .route("/robots.txt", web::get().to(web_api::robots_txt))
            .service(
                web::resource("/profile/delete")
                    .route(web::post().to(web_api::delete_profile_endpoint)),
            )
            .service(
                web::resource("/profile_photo/upload")
                    .route(web::post().to(web_api::add_profile_photo_endpoint)),
            )
            .service(
                web::resource("/profile_photo/delete")
                    .route(web::post().to(web_api::delete_profile_photo_endpoint)),
            )
            .service(
                web::resource("/comment/delete")
                    .route(web::post().to(web_api::delete_comment_endpoint)),
            )
            .service(
                web::resource("/sign_in/google")
                    .route(web::post().to(web_api::google_sign_in_endpoint)),
            )
            .service(web::resource("/sign_out").route(web::get().to(web_api::sign_out_endpoint)))
            // static services
            .service(
                web::scope("")
                    .wrap(middleware::DefaultHeaders::new().add(("Cache-Control", "max-age=86400")))
                    .service(Files::new("/static", "static").index_file("not_found"))
                    .service(Files::new("/photos", &all_photos_os_folder).index_file("not_found")),
            )
            .default_service(web::route().to(web_api::p404_page))
    })
    .bind(&addr)
    .unwrap()
    .run();
    info!("Server live at http://{}", &addr);
    server.await.unwrap();
}

fn all_photos_folder_path(config: &Config) -> String {
    let mut new_file_path = env::current_exe().unwrap();
    // remove binary name
    new_file_path.pop();
    // add global_folder
    new_file_path.push(&config.all_photos_folder_name);
    if !new_file_path.exists() {
        fs::create_dir_all(&new_file_path).unwrap();
    }

    new_file_path.to_str().unwrap().to_owned()
}
