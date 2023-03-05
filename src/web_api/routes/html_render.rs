use actix_web::HttpResponse;
use sailfish::TemplateOnce;

use super::add_page::AddProfilePageDataContext;
use super::home_page::HomePageDataContext;
use super::util::{ActionContext, NavContext};

#[derive(TemplateOnce)]
#[template(path = "home.stpl")]
struct Home<'a> {
    nav_context: &'a NavContext<'a>,
    action_context: &'a ActionContext<'a>,
    data_context: &'a HomePageDataContext,
}

#[derive(TemplateOnce)]
#[template(path = "add_profile.stpl")]
struct AddProfile<'a> {
    nav_context: &'a NavContext<'a>,
    data_context: &'a AddProfilePageDataContext,
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
        data_context: &AddProfilePageDataContext,
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
