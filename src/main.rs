use std::env;

use actix_files::Files;
use actix_web::{web, App, HttpServer};
use sea_orm::{Database, DbConn, DbErr};
use std::fs;

mod config;
mod constant;
mod db;
mod web_api;

use crate::{config::Config, db::DbProvider};

async fn establish_connection(conf: &Config) -> Result<DbConn, DbErr> {
    Database::connect(&conf.database_url).await
}

#[actix_web::main]
async fn main() {
    let conf = Config::init();
    let db_con = establish_connection(&conf).await.unwrap();
    let provider = DbProvider::new(db_con);

    let addr = "localhost:8000";
    let all_photos_os_folder = all_photos_folder_path(&conf);

    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(provider.clone()))
            .app_data(web::Data::new(conf.clone()))
            .route("/", web::get().to(web_api::index_page))
            .route("/add_profile", web::get().to(web_api::add_profile_get))
            .route("/add_profile", web::post().to(web_api::add_profile_post))
            .route("/edit_profile", web::get().to(web_api::edit_profile_page))
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
                web::resource("/sign_in/google")
                    .route(web::post().to(web_api::google_sign_in_endpoint)),
            )
            .service(web::resource("/sign_out").route(web::get().to(web_api::sign_out_endpoint)))
            // static services
            .service(Files::new("/static", "static").show_files_listing())
            .service(Files::new("/photos", &all_photos_os_folder).show_files_listing())
    })
    .bind(addr)
    .unwrap()
    .run();
    println!("Server live at http://{}", addr);
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
