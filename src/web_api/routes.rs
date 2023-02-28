use std::error::Error;

use actix_web::{cookie::Cookie, web, HttpRequest, HttpResponse, Responder};

use sailfish::TemplateOnce;
use serde::Deserialize;

use crate::{config::Config, db::DbProvider, db::UserModel, web_api::auth::AuthSessionManager};

use super::{auth::AuthenticationGate, sign_in::get_google_user};

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

pub async fn homepage(auth_gate: AuthenticationGate) -> impl Responder {
    println!(
        "Inside the homepage endpoint. User auth status {}",
        auth_gate.is_authorized
    );
    get_homepage_html_body("OK Page", None, None)
}

pub async fn google_sign_in(
    db_provider: web::Data<DbProvider>,
    config: web::Data<Config>,
    callback_payload: web::Form<OAuthCallback>,
    request: HttpRequest,
) -> impl Responder {
    if callback_payload.credential.is_empty() {
        return get_homepage_html_body("error", Some("credential field is empty"), None);
    }
    if callback_payload.g_csrf_token.is_empty() {
        return get_homepage_html_body("error", Some("g_csrf_token field is empty"), None);
    }

    if Some(callback_payload.g_csrf_token.clone())
        != request
            .cookie("g_csrf_token")
            .map(|f| f.value().to_string())
    {
        return get_homepage_html_body("error", Some("sign in security attack"), None);
    }

    async fn fetch_and_save_user(
        db_provider: &web::Data<DbProvider>,
        callback_payload: &web::Form<OAuthCallback>,
        config: &web::Data<Config>,
    ) -> Result<UserModel, Box<dyn Error>> {
        let oauth_user = get_google_user(&callback_payload.credential, &config).await?;
        let db_user_opt = db_provider.find_user_by_email(&oauth_user.email).await?;

        let user = if db_user_opt.is_some() {
            println!("Email {} exists. Just reusing", &oauth_user.email);
            db_user_opt.unwrap()
        } else {
            println!("Email {} is new. Creating new user", &oauth_user.email);
            let new_user_model = db_provider
                .add_user(None, &oauth_user.name, &oauth_user.email, Some("Google"))
                .await?;
            new_user_model
        };

        Ok(user)
    }

    let user_res = fetch_and_save_user(&db_provider, &callback_payload, &config).await;

    match user_res {
        Ok(user) => {
            let session_manager = AuthSessionManager::new(&config);
            let jwt_cookie = session_manager.get_jwt_token(user.id).await;

            get_homepage_html_body(&format!("OK Page {:?}", user), None, Some(jwt_cookie))
        }
        Err(err) => get_homepage_html_body(
            "Error",
            Some(&format!("Failed to retrive user {}", err.to_string())),
            None,
        ),
    }
}

fn get_homepage_html_body(
    title: &str,
    error_msg: Option<&str>,
    jwt_cookie_opt: Option<Cookie>,
) -> HttpResponse {
    let mut html = HttpResponse::Ok().body(
        Home {
            head_title: title,
            error_msg: error_msg.unwrap_or(""),
        }
        .render_once()
        .unwrap(),
    );

    jwt_cookie_opt.map(|f| html.add_cookie(&f));

    html
}
