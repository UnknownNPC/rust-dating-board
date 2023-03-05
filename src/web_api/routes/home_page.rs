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

use crate::constant::DEFAULT_PHOTO_PLACEHOLDER;

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
    ) -> HomePageDataContext {
        let all_profiles = db_provider
            .find_all_profiles(PROFILES_ON_PAGE.to_owned())
            .await
            .unwrap();
        let all_profiles_ids = all_profiles.iter().map(|profile| profile.id).collect();
        let profile_id_and_profile_photo_map = db_provider
            .find_first_profile_photos_for(&all_profiles_ids)
            .await
            .unwrap();

        let context_profiles: Vec<HomePageProfileDataContext> = all_profiles
            .iter()
            .map(|profile| {
                let profile_photo_opt = profile_id_and_profile_photo_map.get(&profile.id).unwrap();
                HomePageProfileDataContext::new(&profile, profile_photo_opt, config)
            })
            .collect();

        HomePageDataContext {
            profiles: context_profiles,
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
    let data_context = get_data_context(&db_provider, &config).await;

    HtmlPage::homepage(&nav_context, &action_context, &data_context)
}

#[derive(Deserialize)]
pub enum HomeFilterRequest {
    #[serde(rename = "my-profiles")]
    MyProfiles,
}

#[derive(Deserialize)]
pub struct HomeQueryRequest {
    pub error: Option<String>,
    pub filter: Option<HomeFilterRequest>,
}

pub struct HomePageDataContext<'a> {
    profiles: Vec<HomePageProfileDataContext<'a>>,
}

pub struct HomePageProfileDataContext<'a> {
    name: &'a str,
    short_description: &'a str,
    photo_url: &'a str,
    date_create: &'a str,
}

impl<'a> HomePageProfileDataContext<'a> {
    fn new(
        profile: &ProfileModel,
        profile_photo_opt: &Option<ProfilePhotoModel>,
        config: &web::Data<Config>,
    ) -> Self {
        let short_description: String = profile.description.chars().take(20).collect();
        let photo_url = profile_photo_opt
            .map(|profile_photo| {
                (config.all_photos_folder_name
                    + "/"
                    + profile.id.to_string().as_str()
                    + "/"
                    + profile_photo.file_name.as_str())
                .as_str()
            })
            .unwrap_or(DEFAULT_PHOTO_PLACEHOLDER);

        let date_create: &str = profile
            .created_at
            .format("%Y-%m-%dT%H:%M")
            .to_string()
            .as_str();
        HomePageProfileDataContext {
            name: profile.name.as_str(),
            short_description: short_description.as_str(),
            photo_url: photo_url,
            date_create,
        }
    }
}
