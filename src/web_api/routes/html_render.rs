use actix_web::HttpResponse;
use sailfish::TemplateOnce;

use super::home_page::HomePageDataContext;
use super::common::{NavContext, ProfilePageDataContext};

#[derive(TemplateOnce)]
#[template(path = "home.stpl")]
struct Home<'a> {
    nav_context: &'a NavContext,
    data_context: &'a HomePageDataContext,
}

#[derive(TemplateOnce)]
#[template(path = "add_or_edit_profile.stpl")]
struct AddOrEditProfile<'a> {
    nav_context: &'a NavContext,
    data_context: &'a ProfilePageDataContext,
}

pub struct HtmlPage;

impl HtmlPage {
    pub fn homepage(
        nav_context: &NavContext,
        data_context: &HomePageDataContext,
    ) -> HttpResponse {
        let html = HttpResponse::Ok().body(
            Home {
                nav_context,
                data_context,
            }
            .render_once()
            .unwrap(),
        );

        html
    }

    pub fn add_or_edit_profile(
        nav_context: &NavContext,
        data_context: &ProfilePageDataContext,
    ) -> HttpResponse {
        HttpResponse::Ok().body(
            AddOrEditProfile {
                nav_context,
                data_context,
            }
            .render_once()
            .unwrap(),
        )
    }
}
