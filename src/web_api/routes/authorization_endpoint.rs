use std::error::Error;

use actix_web::{web, HttpRequest, Responder};
use serde::Deserialize;

use crate::{
    config::Config,
    db::DbProvider,
    db::UserModel,
    web_api::{
        auth::{AuthSessionManager, AuthenticationGate},
        routes::{
            constants::{INVALID_G_CSRF, INVALID_USER, LOST_CREDENTIAL, LOST_G_CSRF},
            util::redirect_to_home_page,
        },
        sign_in::get_google_user,
    },
};

pub async fn sign_out_endpoint(auth_gate: AuthenticationGate) -> impl Responder {
    let empty_cookie = AuthSessionManager::get_empty_jwt_token();
    if auth_gate.is_authorized {
        println!(
            "[route#sign_out_endpoint] auth user {} is loging out",
            auth_gate.user_id.unwrap()
        );
        redirect_to_home_page(Some(empty_cookie), None, None, false)
    } else {
        redirect_to_home_page(None, None, None, false)
    }
}

pub async fn google_sign_in_endpoint(
    db_provider: web::Data<DbProvider>,
    config: web::Data<Config>,
    callback_payload: web::Form<GoogleSignInFormRequest>,
    request: HttpRequest,
) -> impl Responder {
    async fn fetch_and_save_user(
        db_provider: &web::Data<DbProvider>,
        callback_payload: &web::Form<GoogleSignInFormRequest>,
        config: &web::Data<Config>,
    ) -> Result<UserModel, Box<dyn Error>> {
        let oauth_user = get_google_user(&callback_payload.credential, &config).await?;
        let db_user_opt = db_provider.find_user_by_email(&oauth_user.email).await?;

        let user = if let Some(db_user) = db_user_opt {
            println!(
                "[route#google_sign_in_endpoint] email {} exists. Just reusing",
                &oauth_user.email
            );
            db_user
        } else {
            println!(
                "[route#google_sign_in_endpoint] email {} is new. Creating new user",
                &oauth_user.email
            );
            let new_user_model = db_provider
                .add_user(None, &oauth_user.name, &oauth_user.email, Some("Google"))
                .await?;
            new_user_model
        };

        Ok(user)
    }

    if callback_payload.credential.is_empty() {
        return redirect_to_home_page(None, Some(LOST_CREDENTIAL), None, false);
    }
    if callback_payload.g_csrf_token.is_empty() {
        return redirect_to_home_page(None, Some(LOST_G_CSRF), None, false);
    }

    if Some(callback_payload.g_csrf_token.clone())
        != request
            .cookie("g_csrf_token")
            .map(|f| f.value().to_string())
    {
        return redirect_to_home_page(None, Some(INVALID_G_CSRF), None, false);
    }

    let user_res = fetch_and_save_user(&db_provider, &callback_payload, &config).await;

    match user_res {
        Ok(user) => {
            let session_manager = AuthSessionManager::new(&config);
            let jwt_cookie = session_manager.get_valid_jwt_token(user.id).await;
            redirect_to_home_page(Some(jwt_cookie), None, None, false)
        }
        Err(err) => {
            println!(
                "[route#google_sign_in_endpoint] error happened during user fetch: {}",
                err
            );
            redirect_to_home_page(None, Some(INVALID_USER), None, false)
        }
    }
}

#[derive(Deserialize)]
pub struct GoogleSignInFormRequest {
    pub credential: String,
    pub g_csrf_token: String,
}
