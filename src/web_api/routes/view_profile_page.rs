use actix_web::{web, Responder};
use futures::future::OptionFuture;
use serde::Deserialize;

use crate::{
    config::Config,
    db::DbProvider,
    web_api::{
        auth::AuthenticationGate,
        routes::{
            common::{get_photo_url, redirect_to_home_page, NavContext},
            constants::{HOME_DATE_FORMAT, NOT_FOUND, NO_PHOTO_URL},
            html_render::HtmlPage,
        },
    },
};

use super::edit_profile_page::EditProfileRequest;

pub async fn view_profile_page(
    db_provider: web::Data<DbProvider>,
    auth_gate: AuthenticationGate,
    config: web::Data<Config>,
    query: web::Query<EditProfileRequest>,
) -> impl Responder {
    async fn resolve_view_profile(
        profile_id: i64,
        db_provider: &web::Data<DbProvider>,
        config: &web::Data<Config>,
    ) -> Option<ViewProfileResponse> {
        let profile_opt = db_provider
            .find_active_profile_by(profile_id)
            .await
            .unwrap();

        let profile_photos = db_provider
            .find_all_profile_photos_for(profile_id)
            .await
            .unwrap();

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

        profile_opt.map(|profile| ViewProfileResponse {
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
    ) -> NavContext {
        let user_opt = OptionFuture::from(
            auth_gate
                .user_id
                .map(|id| db_provider.find_active_profile_by(id)),
        )
        .await
        .map(|f| f.unwrap())
        .flatten();

        let cities_models = db_provider.find_cities_on().await.unwrap();
        let cities_names = cities_models.iter().map(|city| city.name.clone()).collect();

        let user_name = user_opt.map(|user| user.name).unwrap_or_default();

        NavContext::new(user_name, cities_names, String::from(""), false)
    }

    println!("[routes#view_profile_get] User opens profile #{}", query.id);

    let nav_context = resolve_nav_context(&db_provider, &auth_gate).await;
    let data_context_opt = resolve_view_profile(query.id, &db_provider, &config).await;

    match data_context_opt {
        Some(data_context) => HtmlPage::view_profile(&nav_context, &data_context),
        None => redirect_to_home_page(None, Some(NOT_FOUND), None, false),
    }
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
