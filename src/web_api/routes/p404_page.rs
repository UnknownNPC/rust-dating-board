use actix_web::{web, Responder};

use crate::{
    config::Config,
    db::DbProvider,
    web_api::{
        auth::AuthenticationGate,
        routes::{
            common::{HeadContext, NavContext},
            html_render::HtmlPage,
        },
    },
};
use log::info;
use rust_i18n::t;

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
            &Option::None,
            &city_names,
            &config.oauth_google_client_id,
            &config.oauth_google_redirect_url,
        ))
    }

    info!(
        " User auth status: [{}]. 404 page",
        auth_gate.is_authorized,
    );

    let nav_context = get_nav_context(&auth_gate, &config, &db_provider).await?;
    let head_context = HeadContext::new(
        t!("404_page_title").to_string().as_str(),
        t!("404_page_description").to_string().as_str(),
        &config,
        &Option::None,
    );

    Ok(HtmlPage::p404(&head_context, &nav_context))
}
