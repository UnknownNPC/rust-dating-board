use actix_web::{web, Responder};
use futures::future::OptionFuture;
use serde::Deserialize;

use crate::{
    config::Config,
    constant::PROFILES_ON_PAGE,
    db::{DbProvider, ProfileModel, ProfilePhotoModel},
    web_api::{
        auth::AuthenticationGate,
        routes::{
            html_render::HtmlPage,
            util::{ActionContext, NavContext},
        },
    },
};

pub async fn index_page(
    db_provider: web::Data<DbProvider>,
    auth_gate: AuthenticationGate,
    query: web::Query<HomeQueryRequest>,
    config: web::Data<Config>,
) -> impl Responder {
    fn get_nav_context(user_name_opt: &Option<String>) -> NavContext {
        NavContext::new(user_name_opt.as_deref().unwrap_or(""))
    }

    fn get_action_context(error: &Option<String>) -> ActionContext {
        ActionContext::new(error.as_deref().unwrap_or(""))
    }

    async fn get_data_context(
        db_provider: &web::Data<DbProvider>,
        config: &web::Data<Config>,
        query: &web::Query<HomeQueryRequest>,
        auth_gate: &AuthenticationGate,
    ) -> HomePageDataContext {
        let show_users_profiles = query
            .filter_type
            .as_ref()
            .map(|f| matches!(f, HomeFilterTypeRequest::My))
            .unwrap_or(false);

        let can_show_user_profiles = auth_gate.is_authorized && show_users_profiles;

        let all_profiles = if can_show_user_profiles {
            db_provider
                .all_user_profiles(auth_gate.user_id.unwrap())
                .await
                .map(|res| (0, res))
                .unwrap()
        } else {
            // regular page
            db_provider
                .profiles_pagination(PROFILES_ON_PAGE.to_owned(), &query.page)
                .await
                .unwrap()
        };
        let all_profiles_ids = all_profiles.1.iter().map(|profile| profile.id).collect();
        let profile_id_and_profile_photo_map = db_provider
            .find_first_profile_photos_for(&all_profiles_ids)
            .await
            .unwrap();

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

        HomePageDataContext {
            profiles: context_profiles,
            pagination: Pagination {
                has_next,
                has_previous,
                current: curret_page,
                total: total_pages,
            },
            is_user_profiles: can_show_user_profiles,
        }
    }

    println!(
        "[route#index_page] Inside the index page. User auth status {}",
        auth_gate.is_authorized
    );

    let user_opt = OptionFuture::from(auth_gate.user_id.map(|id| db_provider.find_user_by_id(id)))
        .await
        .unwrap_or(Ok(None))
        .unwrap();

    let user_name_opt = user_opt.map(|f| f.name);
    let nav_context = get_nav_context(&user_name_opt);
    let action_context = get_action_context(&query.error);
    let data_context = get_data_context(&db_provider, &config, &query, &auth_gate).await;

    HtmlPage::homepage(&nav_context, &action_context, &data_context)
}

#[derive(Deserialize)]
pub enum HomeFilterTypeRequest {
    #[serde(rename = "my")]
    My,
}

#[derive(Deserialize)]
pub struct HomeQueryRequest {
    pub error: Option<String>,
    pub filter_type: Option<HomeFilterTypeRequest>,
    pub filter_city: Option<String>,
    pub page: Option<u64>,
}

pub struct Pagination {
    pub has_next: bool,
    pub has_previous: bool,
    pub current: u64,
    pub total: u64,
}

pub struct HomePageDataContext {
    pub profiles: Vec<HomePageProfileDataContext>,
    pub pagination: Pagination,
    pub is_user_profiles: bool,
}

#[derive(Clone)]
pub struct HomePageProfileDataContext {
    pub name: String,
    pub short_description: String,
    pub photo_url_opt: Option<String>,
    pub date_create: String,
}

impl HomePageProfileDataContext {
    fn new(
        profile: &ProfileModel,
        profile_photo_opt: &Option<ProfilePhotoModel>,
        config: &web::Data<Config>,
    ) -> Self {
        let short_description: String = profile.description.chars().take(20).collect();

        let photo_url_opt = profile_photo_opt.as_ref().map(|profile_photo| {
            config.all_photos_folder_name.clone()
                + "/"
                + profile.id.to_string().as_str()
                + "/"
                + profile_photo.file_name.as_str()
        });

        let date_create = profile.created_at.format("%Y-%m-%d %H:%M").to_string();
        HomePageProfileDataContext {
            name: profile.name.clone(),
            short_description: short_description,
            photo_url_opt: photo_url_opt,
            date_create,
        }
    }
}
