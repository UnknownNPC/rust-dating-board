use actix_web::HttpResponse;
use sailfish::TemplateOnce;

use super::common::{NavContext, ProfilePageDataContext};
use super::home_page::HomePageDataContext;
use super::validator::ErrorContext;
use super::view_profile_page::ViewProfileResponse;

#[derive(TemplateOnce)]
#[template(path = "p404.stpl")]
struct P404<'a> {
    nav_context: &'a NavContext,
}

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
    error_context: &'a ErrorContext
}

#[derive(TemplateOnce)]
#[template(path = "view_profile.stpl")]
struct ViewProfile<'a> {
    nav_context: &'a NavContext,
    data_context: &'a ViewProfileResponse,
}

pub struct HtmlPage;

impl HtmlPage {
    pub fn homepage(nav_context: &NavContext, data_context: &HomePageDataContext) -> HttpResponse {
        HttpResponse::Ok().body(
            Home {
                nav_context,
                data_context,
            }
            .render_once()
            .unwrap(),
        )
    }

    pub fn add_or_edit_profile(
        nav_context: &NavContext,
        data_context: &ProfilePageDataContext,
        error_context: &ErrorContext
    ) -> HttpResponse {
        HttpResponse::Ok().body(
            AddOrEditProfile {
                nav_context,
                data_context,
                error_context
            }
            .render_once()
            .unwrap(),
        )
    }

    pub fn view_profile(
        nav_context: &NavContext,
        data_context: &ViewProfileResponse,
    ) -> HttpResponse {
        HttpResponse::Ok().body(
            ViewProfile {
                nav_context,
                data_context,
            }
            .render_once()
            .unwrap(),
        )
    }

    pub fn p404(nav_context: &NavContext) -> HttpResponse {
        HttpResponse::NotFound().body(P404 { nav_context }.render_once().unwrap())
    }
}
