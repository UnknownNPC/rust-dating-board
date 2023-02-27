use std::error::Error;

use actix_web::{web, HttpRequest, HttpResponse, Responder};

use sailfish::TemplateOnce;
use serde::Deserialize;

use crate::{config::Config, db::DbProvider};

use super::sign_in::get_google_user;

#[derive(Deserialize)]
pub struct OAuthCallback {
    credential: String,
    g_csrf_token: String,
}

#[derive(TemplateOnce)]
#[template(path = "home.stpl")]
struct Home<'a> {
    head_title: &'a str,
    error_msg: &'a str,
}

pub async fn homepage() -> impl Responder {
    get_homepage_html_body("OK Page", None)
}

pub async fn google_sign_in(
    db_provider: web::Data<DbProvider>,
    config: web::Data<Config>,
    callback_payload: web::Form<OAuthCallback>,
    request: HttpRequest,
) -> impl Responder {
    if callback_payload.credential.is_empty() {
        return get_homepage_html_body("error", Some("credential field is empty"));
    }
    if callback_payload.g_csrf_token.is_empty() {
        return get_homepage_html_body("error", Some("g_csrf_token field is empty"));
    }

    if Some(callback_payload.g_csrf_token.clone())
        != request
            .cookie("g_csrf_token")
            .map(|f| f.value().to_string())
    {
        return get_homepage_html_body("error", Some("sign in security attack"));
    }

    let oauth_user_opt = get_google_user(&callback_payload.credential, &config).await;

    match oauth_user_opt {
        Ok(oauth_user) => {
            let db_user_res_opt = db_provider.find_user_by_email(&oauth_user.email).await;
            let db_user = db_user_res_opt.unwrap();


            let db_user = if (db_user.is_some()) {
                println!("Email {} exists. Just reusing", &oauth_user.email);
                db_user.unwrap()
            } else {
                println!("Email {} is new. Creating new user", &oauth_user.email);
               let save_result =  db_provider.add_user(None, &oauth_user.name, &oauth_user.email, Some("Google")).await;
               save_result.unwrap()
            };



            get_homepage_html_body(&format!("OK Page {:?}", db_user), None)

        }
        Err(err) => get_homepage_html_body(
            "Error",
            Some(&format!("Failed to retrive user {}", err.to_string())),
        ),
    }
}

fn get_homepage_html_body(title: &str, error_msg: Option<&str>) -> HttpResponse {
    HttpResponse::Ok().body(
        Home {
            head_title: title,
            error_msg: error_msg.unwrap_or(""),
        }
        .render_once()
        .unwrap(),
    )
}
