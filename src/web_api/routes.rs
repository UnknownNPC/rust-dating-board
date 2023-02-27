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
    HttpResponse::Ok().body(get_homepage_html_body("OK Page", None))
}

pub async fn google_sign_in(
    _db_provider: web::Data<DbProvider>,
    config: web::Data<Config>,
    callback_payload: web::Form<OAuthCallback>,
    request: HttpRequest,
) -> impl Responder {
    if callback_payload.credential.is_empty() {
        return HttpResponse::Ok().body(get_homepage_html_body(
            "error",
            Some("credential field is empty"),
        ));
    }
    if callback_payload.g_csrf_token.is_empty() {
        return HttpResponse::Ok().body(get_homepage_html_body(
            "error",
            Some("g_csrf_token field is empty"),
        ));
    }

    if Some(callback_payload.g_csrf_token.clone())
        != request
            .cookie("g_csrf_token")
            .map(|f| f.value().to_string())
    {
        return HttpResponse::Ok().body(get_homepage_html_body(
            "error",
            Some("sign in security attack"),
        ));
    }

    let oauth_user_opt = get_google_user(&callback_payload.credential, &config).await;

    match oauth_user_opt {
        Ok(google_user) => HttpResponse::Ok().body(get_homepage_html_body(
            &format!("User was retrieved {:?}", google_user),
            None,
        )),
        Err(err) => HttpResponse::Ok().body(get_homepage_html_body(
            "Error",
            Some(&format!("Failed to retrive user {}", err.to_string())),
        )),
    }
}

fn get_homepage_html_body(title: &str, error_msg: Option<&str>) -> String {
    Home {
        head_title: title,
        error_msg: error_msg.unwrap_or(""),
    }
    .render_once()
    .unwrap()
}
