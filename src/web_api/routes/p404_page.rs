use actix_web::{web, Responder};

use crate::{
    config::Config,
    db::DbProvider,
    web_api::{
        auth::AuthenticationGate,
        routes::{common::NavContext, html_render::HtmlPage},
    },
};

use super::error::HtmlError;

pub async fn p404_page(
    auth_gate: AuthenticationGate,
    config: web::Data<Config>,
    db_provider: web::Data<DbProvider>,
) -> Result<impl Responder, HtmlError> {
    async fn get_nav_context(
        auth_gate: &AuthenticationGate,
        config: &web::Data<Config>,
        db_provider: &web::Data<DbProvider>,
    ) -> Result<NavContext, HtmlError> {
        let city_names = db_provider.find_city_names().await?;
        let user_name = auth_gate
            .user_name
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or_default();

        Ok(NavContext::new(
            user_name,
            "",
            &config.captcha_google_id,
            false,
            false,
            &city_names,
            &config.oauth_google_client_id,
            &config.oauth_google_redirect_url,
        ))
    }

    println!(
        "[route#404] User auth status: [{}]. 404 page",
        auth_gate.is_authorized,
    );

    let nav_context = get_nav_context(&auth_gate, &config, &db_provider).await?;

    Ok(HtmlPage::p404(&nav_context))
}
