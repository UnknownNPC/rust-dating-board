use actix_web::{web, Responder};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    config::Config,
    db::{DbProvider, ProfileModel, ProfilePhotoModel},
    web_api::{
        auth::AuthenticationGate,
        routes::{
            common::{HeadContext, NavContext},
            constant::PROFILES_ON_PAGE,
            html_render::HtmlPage,
        },
    },
};
use log::info;
use rust_i18n::t;

use super::{
    bot_detector_gate::BotDetector, common::get_relative_photo_url, constant::HOME_DATE_FORMAT,
    error::HtmlError,
};

pub async fn index_page(
    db_provider: web::Data<DbProvider>,
    auth_gate: AuthenticationGate,
    bot_detector: BotDetector,
    query: web::Query<QueryRequest>,
    config: web::Data<Config>,
) -> Result<impl Responder, HtmlError> {
    async fn get_nav_context(
        auth_gate: &AuthenticationGate,
        query: &web::Query<QueryRequest>,
        config: &web::Data<Config>,
        db_provider: &web::Data<DbProvider>,
    ) -> Result<NavContext, HtmlError> {
        let city_names = db_provider.find_city_names().await?;
        let user_name = auth_gate
            .user_name
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or_default();
        let current_city = query
            .filter_city
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or_default();

        let is_user_profiles = auth_gate.is_authorized && query.show_my.unwrap_or_default();

        Ok(NavContext::new(
            user_name,
            current_city,
            &config.captcha_google_id,
            is_user_profiles,
            &query.search,
            &city_names,
            &config.oauth_google_client_id,
            &config.oauth_google_redirect_url,
        ))
    }

    async fn get_data_context(
        db_provider: &web::Data<DbProvider>,
        config: &web::Data<Config>,
        query: &web::Query<QueryRequest>,
        auth_gate: &AuthenticationGate,
    ) -> Result<HomePageDataContext, HtmlError> {
        let is_user_profiles = auth_gate.is_authorized && query.show_my.unwrap_or_default();
        let is_search = query.search.is_some();
        let search_result = 20;

        let all_profiles = if is_search {
            let profiles = db_provider
                .search_profiles(query.search.as_ref().unwrap(), search_result)
                .await?;
            (0, profiles)
        } else if is_user_profiles {
            let profiles = db_provider
                .all_user_profiles(auth_gate.user_id.unwrap())
                .await?;
            (0, profiles)
        } else {
            // regular page
            db_provider
                .profiles_pagination(PROFILES_ON_PAGE.to_owned(), &query.page, &query.filter_city)
                .await?
        };
        let all_profiles_ids = all_profiles.1.iter().map(|profile| profile.id).collect();
        let profile_id_and_profile_photo_map = db_provider
            .find_first_profile_photos_for(&all_profiles_ids)
            .await?;

        let context_profiles: Vec<HomePageProfileDataContext> = all_profiles
            .1
            .iter()
            .map(|profile| {
                let profile_photo_opt = profile_id_and_profile_photo_map.get(&profile.id).unwrap();
                HomePageProfileDataContext::new(&profile, profile_photo_opt, config)
            })
            .collect();

        let total_pages = all_profiles.0;
        let curret_page = query.page.unwrap_or(1);
        let has_next = curret_page < total_pages;
        let has_previous = curret_page > 1;

        Ok(HomePageDataContext {
            profiles: context_profiles,
            pagination: Pagination {
                has_next,
                has_previous,
                current: curret_page,
                total: total_pages,
            },
            search_text: query.search.clone(),
            message_code: query.message.clone(),
        })
    }

    async fn get_head_context(
        db_provider: &web::Data<DbProvider>,
        config: &web::Data<Config>,
        search: &Option<String>,
    ) -> Result<HeadContext, HtmlError> {
        let photo = &db_provider.find_any_active_profile_photo().await?;
        let is_search = search.is_some();
        let title = if is_search {
            format!(
                "{} {}",
                t!("search_title").to_string().as_str(),
                search.as_ref().unwrap()
            )
        } else {
            t!("main_page_title").to_string()
        };

        Ok(HeadContext::new(
            title.as_str(),
            t!("main_page_description").to_string().as_str(),
            &config,
            photo,
        ))
    }

    info!(
        "User auth status: [{}], Is Bot [{}]. Index page",
        auth_gate.is_authorized, bot_detector.is_bot
    );

    let nav_context = get_nav_context(&auth_gate, &query, &config, &db_provider).await?;
    let data_context = get_data_context(&db_provider, &config, &query, &auth_gate).await?;
    let head_context = get_head_context(&db_provider, &config, &query.search).await?;
    Ok(HtmlPage::homepage(
        &head_context,
        &nav_context,
        &data_context,
    ))
}

pub struct Pagination {
    pub has_next: bool,
    pub has_previous: bool,
    pub current: u64,
    pub total: u64,
}

pub struct HomePageDataContext {
    pub message_code: Option<String>,
    pub search_text: Option<String>,
    pub profiles: Vec<HomePageProfileDataContext>,
    pub pagination: Pagination,
}

#[derive(Clone)]
pub struct HomePageProfileDataContext {
    pub id: Uuid,
    pub name: String,
    pub city: String,
    pub short_description: String,
    pub photo_url_opt: Option<String>,
    pub date_create: String,
    pub view_count: i64,
}

impl HomePageProfileDataContext {
    fn new(
        profile: &ProfileModel,
        profile_photo_opt: &Option<ProfilePhotoModel>,
        config: &web::Data<Config>,
    ) -> Self {
        let short_description: String = profile.description.chars().take(50).collect();
        let photo_url_opt = profile_photo_opt.as_ref().map(|profile_photo| {
            get_relative_photo_url(profile_photo, &config.all_photos_folder_name)
        });

        let date_create = profile.created_at.format(HOME_DATE_FORMAT).to_string();
        HomePageProfileDataContext {
            id: profile.id,
            name: profile.name.clone(),
            city: profile.city.clone(),
            short_description: short_description,
            photo_url_opt: photo_url_opt,
            date_create,
            view_count: profile.view_count,
        }
    }
}

#[derive(Deserialize)]
pub struct QueryRequest {
    pub message: Option<String>,
    pub show_my: Option<bool>,
    pub search: Option<String>,
    pub filter_city: Option<String>,
    pub page: Option<u64>,
}
