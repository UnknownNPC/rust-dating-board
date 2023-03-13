use actix_web::{
    cookie::Cookie,
    http::{header::LOCATION, StatusCode},
    web, HttpRequest, HttpResponse, Responder,
};
use serde::Deserialize;

use crate::{
    config::Config,
    db::DbProvider,
    db::UserModel,
    web_api::{
        auth::{AuthSessionManager, AuthenticationGate},
        routes::constant::{MSG_SIGN_IN_CODE, MSG_SIGN_OUT_CODE},
        sign_in::get_google_user,
    },
};

use super::error::HtmlError;


pub async fn sign_out_endpoint(auth_gate: AuthenticationGate) -> impl Responder {
    let empty_cookie = AuthSessionManager::get_empty_jwt_token();
    if auth_gate.is_authorized {
        println!(
            "[route#sign_out_endpoint] auth user {} is loging out. Token exists.",
            auth_gate.user_id.unwrap()
        );
        homepage(Some(empty_cookie), MSG_SIGN_OUT_CODE)
    } else {
        println!(
            "[route#sign_out_endpoint] auth user {} is loging out. Session expired",
            auth_gate.user_id.unwrap()
        );
        homepage(None, MSG_SIGN_OUT_CODE)
    }
}

pub async fn google_sign_in_endpoint(
    db_provider: web::Data<DbProvider>,
    config: web::Data<Config>,
    callback_payload: web::Form<GoogleSignInFormRequest>,
    request: HttpRequest,
) -> Result<impl Responder, HtmlError> {
    async fn fetch_and_save_user(
        oauth_google_client_id: &str,
        jwt_credentials: &str,
        db_provider: &web::Data<DbProvider>,
    ) -> Result<UserModel, HtmlError> {
        let oauth_user = get_google_user(jwt_credentials, oauth_google_client_id).await?;
        let db_user_opt = db_provider.find_user_by_email(&oauth_user.email).await?;
        match db_user_opt {
            Some(db_user) => {
                println!(
                    "[route#google_sign_in_endpoint] email [{}] exists. Just reusing",
                    &db_user.email
                );
                Ok(db_user)
            }
            None => {
                println!(
                    "[route#google_sign_in_endpoint] email [{}] is new. Creating new user",
                    &oauth_user.email
                );
                db_provider
                    .add_user(None, &oauth_user.name, &oauth_user.email, Some("Google"))
                    .await
                    .map_err(|err| err.into())
            }
        }
    }

    let cookie_gsrf_token = request
        .cookie("g_csrf_token")
        .map(|f| f.value().to_string())
        .unwrap_or_default();

    let is_gsrf_token_matches = &callback_payload.g_csrf_token == &cookie_gsrf_token;

    if callback_payload.credential.is_empty()
        || callback_payload.g_csrf_token.is_empty()
        || !is_gsrf_token_matches
    {
        println!("[route#google_sign_in_endpoint] sign in error: credential [{}], g_csrf_token [{}], gsrf_token_matches [{}]",
        &callback_payload.credential.is_empty(), &callback_payload.g_csrf_token.is_empty(), is_gsrf_token_matches
    );
        return Err(HtmlError::BadParams);
    }

    let user = fetch_and_save_user(
        &config.oauth_google_client_id,
        &callback_payload.credential,
        &db_provider,
    )
    .await?;

    let session_manager = AuthSessionManager::new(&config);
    let jwt_cookie = session_manager
        .get_valid_jwt_token(user.id, &user.name, &user.name)
        .await;
    Ok(homepage(Some(jwt_cookie), MSG_SIGN_IN_CODE))
}

#[derive(Deserialize)]
pub struct GoogleSignInFormRequest {
    pub credential: String,
    pub g_csrf_token: String,
}

pub fn homepage(jwt_cookie_opt: Option<Cookie>, message: &str) -> HttpResponse {
    let path = format!("/?message={}", message);
    let mut response_builder = HttpResponse::build(StatusCode::FOUND);
    
    response_builder.append_header((LOCATION, path));
    if jwt_cookie_opt.is_some() {
        response_builder.cookie(jwt_cookie_opt.unwrap());
    };
    response_builder.finish()
}
