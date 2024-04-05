use actix_web::{http::header::ContentType, web, HttpResponse, Responder};

use crate::config::Config;

use super::{common::get_absolute_url, error::HtmlError};

pub async fn robots_txt(config: web::Data<Config>) -> Result<impl Responder, HtmlError> {
    let site_map_url = get_absolute_url(&config, "/sitemap.xml");
    let content = format!(
        "User-agent: *\nSitemap: {}",
        &site_map_url
    );
    let response_builder = HttpResponse::Ok()
        .content_type(ContentType::plaintext())
        .body(content);
    Ok(response_builder)
}
