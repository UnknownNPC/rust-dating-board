use actix_web::{web, Responder};
use serde::Deserialize;
use uuid::Uuid;

use rust_i18n::t;
use crate::{
    config::Config,
    db::DbProvider,
    web_api::{
        auth::AuthenticationGate,
        routes::{
            common::{HeadContext, NavContext, ProfilePageDataContext},
            html_render::HtmlPage,
            validator::ErrorContext,
        },
    },
};

use super::error::HtmlError;

pub async fn edit_profile_page(
    auth_gate: AuthenticationGate,
    db_provider: web::Data<DbProvider>,
    query: web::Query<EditProfileRequest>,
    config: web::Data<Config>,
) -> Result<impl Responder, HtmlError> {
    if !auth_gate.is_authorized {
        return Err(HtmlError::NotAuthorized);
    }

    println!(
        "[route#add_or_edit_profile_post] User auth status: [{}]. User ID: [{}]",
        auth_gate.is_authorized,
        auth_gate.user_id.unwrap_or_default()
    );

    let taget_profile_id = query.id;

    let profile_opt = db_provider
        .find_active_profile_by_id_and_user_id(&taget_profile_id, auth_gate.user_id.unwrap())
        .await?;
    let profile = profile_opt.ok_or(HtmlError::NotFound)?;

    let profile_photos = db_provider.find_all_profile_photos_for(&profile.id).await?;

    let cities_names = db_provider.find_city_names().await?;

    let data_contex = ProfilePageDataContext::new(
        &config.all_photos_folder_name,
        &Some(profile),
        &profile_photos,
        true,
    );

    let nav_context = NavContext::new(
        &auth_gate.user_name.unwrap(),
        "",
        &config.captcha_google_id,
        false,
        false,
        &cities_names,
        &config.oauth_google_client_id,
            &config.oauth_google_redirect_url,
    );
    let error_context = ErrorContext::empty();
    let head_context = HeadContext::new(
        t!("edit_profile_page_title").to_string().as_str(),
        t!("edit_profile_page_description").to_string().as_str(),
        &config,
        &Option::None,
    );

    Ok(HtmlPage::add_or_edit_profile(
        &head_context,
        &nav_context,
        &data_contex,
        &error_context,
    ))
}

#[derive(Deserialize)]
pub struct EditProfileRequest {
    pub id: Uuid,
}
