use actix_web::{web, Responder};
use futures::future::OptionFuture;
use serde::Deserialize;

use crate::{
    db::DbProvider,
    web_api::{auth::AuthenticationGate, routes::html_render::HtmlPage},
};

pub async fn index_page(
    db_provider: web::Data<DbProvider>,
    auth_gate: AuthenticationGate,
    query: web::Query<HomeQueryRequest>,
) -> impl Responder {
    println!(
        "[route#index_page] Inside the index page. User auth status {}",
        auth_gate.is_authorized
    );

    let user_opt = OptionFuture::from(auth_gate.user_id.map(|id| db_provider.find_user_by_id(id)))
        .await
        .unwrap_or(Ok(None))
        .unwrap();

    let user_name = user_opt.map(|f| f.name);

    HtmlPage::homepage(query.error.as_deref(), user_name.as_deref())
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
