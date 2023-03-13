use actix_web::{web, Responder};
use serde::Deserialize;

use crate::{
    config::Config,
    db::DbProvider,
    web_api::{
        auth::AuthenticationGate,
        routes::{
            common::{get_photo_url, NavContext},
            constant::{HOME_DATE_FORMAT, NO_PHOTO_URL},
            html_render::HtmlPage,
        },
    },
};

use super::{edit_profile_page::EditProfileRequest, error::HtmlError};

pub async fn view_profile_page(
    db_provider: web::Data<DbProvider>,
    auth_gate: AuthenticationGate,
    config: web::Data<Config>,
    query: web::Query<EditProfileRequest>,
) -> Result<impl Responder, HtmlError> {
    async fn resolve_view_profile(
        profile_id: i64,
        db_provider: &web::Data<DbProvider>,
        config: &web::Data<Config>,
    ) -> Result<ViewProfileResponse, HtmlError> {
        let profile_opt = db_provider.find_active_profile_by(profile_id).await?;
        let profile = profile_opt.ok_or(HtmlError::NotFound)?;
        let profile_photos = db_provider.find_all_profile_photos_for(profile_id).await?;

        let photo_urls: Vec<String> = profile_photos
            .iter()
            .map(|profile_photo| {
                get_photo_url(profile_photo, config.all_photos_folder_name.as_str())
            })
            .collect();
        let photo_urls_or_placeholder = match photo_urls.is_empty() {
            true => vec![NO_PHOTO_URL.to_string()],
            false => photo_urls,
        };

        Ok(ViewProfileResponse {
            id: profile.id,
            name: profile.name,
            phone_num: profile.phone_number,
            height: profile.height as i64,
            city: profile.city,
            description: profile.description,
            photo_urls: photo_urls_or_placeholder,
            date_create: profile.created_at.format(HOME_DATE_FORMAT).to_string(),
        })
    }

    async fn resolve_nav_context(
        db_provider: &web::Data<DbProvider>,
        auth_gate: &AuthenticationGate,
        config: &web::Data<Config>,
    ) -> Result<NavContext, HtmlError> {
        let name = auth_gate
            .user_name
            .as_ref()
            .map(|f| f.as_str())
            .unwrap_or_default();
        let cities_names = db_provider.find_city_names().await?;

        Ok(NavContext::new(
            name,
            "",
            &config.captcha_google_id,
            false,
            false,
            &cities_names,
        ))
    }

    println!(
        "[route#view_profile_page] User auth status: [{}]. User ID: [{}]",
        auth_gate.is_authorized,
        auth_gate.user_id.unwrap_or_default()
    );

    let nav_context = resolve_nav_context(&db_provider, &auth_gate, &config).await?;
    let data_context = resolve_view_profile(query.id, &db_provider, &config).await?;

    Ok(HtmlPage::view_profile(&nav_context, &data_context))
}

#[derive(Deserialize)]
pub struct ViewProfileRequest {
    pub id: i64,
}

pub struct ViewProfileResponse {
    pub id: i64,
    pub name: String,
    pub phone_num: String,
    pub height: i64,
    pub city: String,
    pub description: String,
    pub photo_urls: Vec<String>,
    pub date_create: String,
}