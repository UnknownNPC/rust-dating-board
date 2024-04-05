use actix_web::{web, Responder};
use chrono::Utc;

use crate::{config::Config, db::DbProvider, web_api::routes::html_render::HtmlPage};

use super::{common::get_absolute_url, error::HtmlError};

pub static DATE_FORMAT: &'static str = "%Y-%m-%d";
pub static INDEX_URL_PRIORITY: &'static str = "1.0";
pub static CITY_URL_PRIORITY: &'static str = "0.9";
pub static PROFILE_URL_PRIORITY: &'static str = "0.7";
pub static INDEX_URL_UPDATE_FREQ: &'static str = "daily";
pub static CITY_URL_UPDATE_FREQ: &'static str = "daily";
pub static PROFILE_URL_UPDATE_FREQ: &'static str = "weekly";

pub struct UrlContext {
    pub loc: String,
    pub lastmod: String,
    pub changefreq: String,
    pub priority: String,
}

impl UrlContext {
    pub fn new(loc: &str, lastmod: &str, changefreq: &str, priority: &str) -> Self {
        UrlContext {
            loc: loc.to_owned(),
            lastmod: lastmod.to_owned(),
            changefreq: changefreq.to_owned(),
            priority: priority.to_owned(),
        }
    }
}

pub struct SitemapContext {
    pub urls: Vec<UrlContext>,
}

pub async fn sitemap(
    config: web::Data<Config>,
    db_provider: web::Data<DbProvider>,
) -> Result<impl Responder, HtmlError> {

    let index = get_absolute_url(&config, "/");

    // cities
    let latest_profiles_per_city = db_provider
        .find_latest_active_profile_from_every_city()
        .await?;
    let mut city_sitemaps = latest_profiles_per_city
        .iter()
        .map(|f| {
            let lastmod = f.updated_at.format(DATE_FORMAT).to_string();
            UrlContext::new(
                format!("{}{}", &index, f.city).as_str(),
                lastmod.as_str(),
                CITY_URL_UPDATE_FREQ,
                CITY_URL_PRIORITY,
            )
        })
        .collect::<Vec<UrlContext>>();

    //index
    let mut latest_profiles_per_city_copy = latest_profiles_per_city.clone();
    latest_profiles_per_city_copy.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    let latest_city_with_profile = latest_profiles_per_city_copy.first();
    let latest_date = latest_city_with_profile
        .map(|f| f.updated_at)
        .unwrap_or(Utc::now().naive_utc());
    let index_url_context = UrlContext::new(
        format!("{}{}", &index, "").as_str(),
        latest_date.format(DATE_FORMAT).to_string().as_str(),
        INDEX_URL_UPDATE_FREQ,
        INDEX_URL_PRIORITY,
    );

    // profiles
    let profiles = db_provider
        .profiles_pagination(100, &Option::None, &Option::None)
        .await?;
    let mut profile_sitemaps = profiles
        .1
        .iter()
        .map(|profile| {
            let url = format!("view_profile?id={}", profile.id.to_string());
            let lastmod = profile.updated_at.format(DATE_FORMAT).to_string();
            UrlContext::new(
                format!("{}{}", &index, url).as_str(),
                lastmod.as_str(),
                PROFILE_URL_UPDATE_FREQ,
                PROFILE_URL_PRIORITY,
            )
        })
        .collect::<Vec<UrlContext>>();

    let mut result: Vec<UrlContext> = vec![];
    result.append(city_sitemaps.as_mut());
    result.push(index_url_context);
    result.append(profile_sitemaps.as_mut());

    let context = SitemapContext { urls: result };

    Ok(HtmlPage::sitemap(&context))
}
