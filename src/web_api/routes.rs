use std::error::Error;

use actix_web::{
    cookie::Cookie,
    http::{header, StatusCode},
    web, HttpRequest, HttpResponse, Responder,
};

use crate::{
    config::Config,
    db::DbProvider,
    db::UserModel,
    web_api::{auth::AuthSessionManager, html_page::HtmlPage},
};

use super::{auth::AuthenticationGate, model::*, sign_in::get_google_user};

pub async fn index(
    db_provider: web::Data<DbProvider>,
    auth_gate: AuthenticationGate,
    query: web::Query<HomeQuery>,
) -> impl Responder {
    println!(
        "Inside the homepage endpoint. User auth status {}",
        auth_gate.is_authorized
    );

    let user = if auth_gate.is_authorized {
        db_provider
            .find_user_by_id(auth_gate.user_id.unwrap())
            .await
            .unwrap()
    } else {
        None
    };

    let user_name = user.map(|f| f.name);

    HtmlPage::homepage("Home page", query.error.as_deref(), user_name.as_deref())
}

pub async fn add_profile(
    db_provider: web::Data<DbProvider>,
    auth_gate: AuthenticationGate,
    query: web::Query<HomeQuery>,
) -> impl Responder {
    println!(
        "Inside the add_profile endpoint. User auth status {}",
        auth_gate.is_authorized
    );

    if !auth_gate.is_authorized {
        return redirect_to_home_page(None, Some("restricted_area"));
    }

    let user = db_provider
        .find_user_by_id(auth_gate.user_id.unwrap())
        .await
        .unwrap()
        .unwrap();

    HtmlPage::add_profile("Add new profile", &user.name)
}

pub async fn sign_out(auth_gate: AuthenticationGate) -> impl Responder {
    let empty_cookie = AuthSessionManager::get_empty_jwt_token();
    if auth_gate.is_authorized {
        println!("Auth user {} is loging out", auth_gate.user_id.unwrap());
        redirect_to_home_page(Some(empty_cookie), None)
    } else {
        redirect_to_home_page(None, None)
    }
}

pub async fn google_sign_in(
    db_provider: web::Data<DbProvider>,
    config: web::Data<Config>,
    callback_payload: web::Form<GoogleSignInPost>,
    request: HttpRequest,
) -> impl Responder {
    async fn fetch_and_save_user(
        db_provider: &web::Data<DbProvider>,
        callback_payload: &web::Form<GoogleSignInPost>,
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

    if callback_payload.credential.is_empty() {
        return redirect_to_home_page(None, Some("lost_credential"));
    }
    if callback_payload.g_csrf_token.is_empty() {
        return redirect_to_home_page(None, Some("lost_g_csrf_token"));
    }

    if Some(callback_payload.g_csrf_token.clone())
        != request
            .cookie("g_csrf_token")
            .map(|f| f.value().to_string())
    {
        return redirect_to_home_page(None, Some("invalid_g_csrf_token"));
    }

    let user_res = fetch_and_save_user(&db_provider, &callback_payload, &config).await;

    match user_res {
        Ok(user) => {
            let session_manager = AuthSessionManager::new(&config);
            let jwt_cookie = session_manager.get_valid_jwt_token(user.id).await;
            redirect_to_home_page(Some(jwt_cookie), None)
        }
        Err(err) => {
            println!("Error happened during user fetch: {}", err);
            redirect_to_home_page(None, Some("invalid_user"))
        }
    }
}

fn redirect_to_home_page(jwt_cookie: Option<Cookie>, error: Option<&str>) -> HttpResponse {
    let mut response_builder = HttpResponse::build(StatusCode::FOUND);

    if error.is_some() {
        response_builder.append_header((header::LOCATION, format!("/?error={}", error.unwrap())))
    } else {
        response_builder.append_header((header::LOCATION, "/"))
    };
    if jwt_cookie.is_some() {
        response_builder.cookie(jwt_cookie.unwrap());
    };
    response_builder.finish()
}
