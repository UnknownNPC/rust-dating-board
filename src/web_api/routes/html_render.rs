use actix_web::HttpResponse;
use sailfish::TemplateOnce;

use super::home_page::HomePageDataContext;
use super::common::{ActionContext, NavContext, ProfilePageDataContext};

#[derive(TemplateOnce)]
#[template(path = "home.stpl")]
struct Home<'a> {
    nav_context: &'a NavContext,
    action_context: &'a ActionContext<'a>,
    data_context: &'a HomePageDataContext,
}

#[derive(TemplateOnce)]
#[template(path = "add_profile.stpl")]
struct AddProfile<'a> {
    nav_context: &'a NavContext,
    data_context: &'a ProfilePageDataContext,
}

pub struct HtmlPage;

impl HtmlPage {
    pub fn homepage(
        nav_context: &NavContext,
        action_context: &ActionContext,
        data_context: &HomePageDataContext,
    ) -> HttpResponse {
        let html = HttpResponse::Ok().body(
            Home {
                nav_context,
                action_context,
                data_context,
            }
            .render_once()
            .unwrap(),
        );

        html
    }

    pub fn add_profile(
        nav_context: &NavContext,
        data_context: &ProfilePageDataContext,
    ) -> HttpResponse {
        HttpResponse::Ok().body(
            AddProfile {
                nav_context,
                data_context,
            }
            .render_once()
            .unwrap(),
        )
    }
}
