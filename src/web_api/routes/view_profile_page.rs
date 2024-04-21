use actix_web::{web, Responder};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    config::Config,
    db::DbProvider,
    web_api::{
        auth::AuthenticationGate,
        routes::{
            common::{get_relative_photo_url, HeadContext, NavContext},
            constant::{HOME_DATE_FORMAT, NO_PHOTO_URL},
            html_render::HtmlPage,
        },
    },
};
use rust_i18n::t;

use super::{
    bot_detector_gate::BotDetector, edit_profile_page::EditProfileRequest, error::HtmlError,
};

pub async fn view_profile_page(
    db_provider: web::Data<DbProvider>,
    auth_gate: AuthenticationGate,
    bot_detector: BotDetector,
    config: web::Data<Config>,
    query: web::Query<EditProfileRequest>,
) -> Result<impl Responder, HtmlError> {
    async fn resolve_view_profile(
        profile_id: &Uuid,
        db_provider: &web::Data<DbProvider>,
        config: &web::Data<Config>,
        auth_gate: &AuthenticationGate,
        bot_detector: &BotDetector,
    ) -> Result<ViewProfileResponse, HtmlError> {
        let profile_opt = db_provider.find_active_profile_by(&profile_id).await?;
        let profile = profile_opt.ok_or(HtmlError::NotFound)?;
        let profile_photos = db_provider.find_all_profile_photos_for(profile_id).await?;

        let photo_urls: Vec<String> = profile_photos
            .iter()
            .map(|profile_photo| {
                get_relative_photo_url(profile_photo, config.all_photos_folder_name.as_str())
            })
            .collect();
        let photo_urls_or_placeholder = match photo_urls.is_empty() {
            true => vec![NO_PHOTO_URL.to_string()],
            false => photo_urls,
        };

        let is_user_profile_author = auth_gate
            .user_id
            .as_ref()
            .map(|auth_user_id| &profile.user_id == auth_user_id)
            .unwrap_or_default();

        //increase view counter
        // if not is_user_profile_author == regular page or search request
        if is_user_profile_author || bot_detector.is_bot {
            println!(
                "Is user profile owner [{}] or bot [{}]. Do not increase view counter",
                is_user_profile_author, bot_detector.is_bot
            )
        } else {
            db_provider
                .increase_view_for_profiles(&vec![profile.id])
                .await?;
        }

        Ok(ViewProfileResponse {
            id: profile.id,
            name: profile.name,
            phone_num: profile.phone_number,
            height: profile.height as i64,
            weight: profile.weight as i64,
            city: profile.city,
            description: profile.description,
            photo_urls: photo_urls_or_placeholder,
            date_create: profile.created_at.format(HOME_DATE_FORMAT).to_string(),
            is_user_profile_author,
            view_count: profile.view_count,
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
            &Option::None,
            &cities_names,
            &config.oauth_google_client_id,
            &config.oauth_google_redirect_url,
        ))
    }

    println!(
        "[route#view_profile_page] Profile ID [{}]. User auth status: [{}]. User ID: [{}]. Is bot: [{}]",
        &query.id,
        auth_gate.is_authorized,
        auth_gate.user_id.unwrap_or_default(),
        bot_detector.is_bot
    );

    let nav_context = resolve_nav_context(&db_provider, &auth_gate, &config).await?;
    let data_context =
        resolve_view_profile(&query.id, &db_provider, &config, &auth_gate, &bot_detector).await?;

    let page_title = format!(
        "{} {} – 0{}",
        t!("view_profile_page_title"),
        &data_context.name,
        &data_context.phone_num
    );
    let profile_photos = db_provider.find_all_profile_photos_for(&query.id).await?;
    let page_description: String = data_context.description.clone().chars().take(100).collect();
    let head_context = HeadContext::new(
        &page_title,
        &page_description,
        &config,
        &profile_photos.first().cloned(),
    );

    Ok(HtmlPage::view_profile(
        &head_context,
        &nav_context,
        &data_context,
    ))
}

#[derive(Deserialize)]
pub struct ViewProfileRequest {
    pub id: i64,
}

pub struct ViewProfileResponse {
    pub id: Uuid,
    pub name: String,
    pub phone_num: String,
    pub height: i64,
    pub weight: i64,
    pub city: String,
    pub description: String,
    pub photo_urls: Vec<String>,
    pub date_create: String,
    pub is_user_profile_author: bool,
    pub view_count: i64,
}
