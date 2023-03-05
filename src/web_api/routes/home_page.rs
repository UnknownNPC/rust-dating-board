use actix_web::{web, Responder};
use futures::future::OptionFuture;
use sea_orm::ColIdx;
use serde::Deserialize;

use crate::{
    db::DbProvider,
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
) -> impl Responder {
    println!(
        "[route#index_page] Inside the index page. User auth status {}",
        auth_gate.is_authorized
    );

    let user_opt = OptionFuture::from(auth_gate.user_id.map(|id| db_provider.find_user_by_id(id)))
        .await
        .unwrap_or(Ok(None))
        .unwrap();

    let user_name_opt = user_opt.map(|f| f.name);
    let nav_context = NavContext::new(user_name_opt.as_deref().unwrap_or(""));

    //todo err code 2 msg
    let action_context = ActionContext::new(query.error.as_deref().unwrap_or(""));

    let data_context = HomePageDataContext { profiles: vec![] };

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
}
