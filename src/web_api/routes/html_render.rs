use actix_web::http::header::ContentType;
use actix_web::HttpResponse;
use sailfish::TemplateOnce;

use super::common::{HeadContext, NavContext, ProfilePageDataContext};
use super::home_page::HomePageDataContext;
use super::sitemap_page::SitemapContext;
use super::validator::ErrorContext;
use super::view_profile_page::ViewProfilePageDataContext;

#[derive(TemplateOnce)]
#[template(path = "p404.stpl")]
struct P404<'a> {
    head_context: &'a HeadContext,
    nav_context: &'a NavContext,
}

#[derive(TemplateOnce)]
#[template(path = "home.stpl")]
struct Home<'a> {
    head_context: &'a HeadContext,
    nav_context: &'a NavContext,
    data_context: &'a HomePageDataContext,
}

#[derive(TemplateOnce)]
#[template(path = "add_or_edit_profile.stpl")]
struct AddOrEditProfile<'a> {
    head_context: &'a HeadContext,
    nav_context: &'a NavContext,
    data_context: &'a ProfilePageDataContext,
    error_context: &'a ErrorContext,
}

#[derive(TemplateOnce)]
#[template(path = "view_profile.stpl")]
struct ViewProfile<'a> {
    head_context: &'a HeadContext,
    nav_context: &'a NavContext,
    data_context: &'a ViewProfilePageDataContext,
    error_context: &'a ErrorContext,
}

#[derive(TemplateOnce)]
#[template(path = "sitemap.stpl")]
struct Sitemap<'a> {
    context: &'a SitemapContext,
}

pub struct HtmlPage;

impl HtmlPage {
    pub fn homepage(
        head_context: &HeadContext,
        nav_context: &NavContext,
        data_context: &HomePageDataContext,
    ) -> HttpResponse {
        HttpResponse::Ok().body(
            Home {
                head_context,
                nav_context,
                data_context,
            }
            .render_once()
            .unwrap(),
        )
    }

    pub fn add_or_edit_profile(
        head_context: &HeadContext,
        nav_context: &NavContext,
        data_context: &ProfilePageDataContext,
        error_context: &ErrorContext,
    ) -> HttpResponse {
        HttpResponse::Ok().body(
            AddOrEditProfile {
                head_context,
                nav_context,
                data_context,
                error_context,
            }
            .render_once()
            .unwrap(),
        )
    }

    pub fn view_profile(
        head_context: &HeadContext,
        nav_context: &NavContext,
        data_context: &ViewProfilePageDataContext,
        error_context: &ErrorContext,
    ) -> HttpResponse {
        HttpResponse::Ok().body(
            ViewProfile {
                head_context,
                nav_context,
                data_context,
                error_context,
            }
            .render_once()
            .unwrap(),
        )
    }

    pub fn p404(head_context: &HeadContext, nav_context: &NavContext) -> HttpResponse {
        HttpResponse::NotFound().body(
            P404 {
                head_context,
                nav_context,
            }
            .render_once()
            .unwrap(),
        )
    }

    pub fn sitemap(context: &SitemapContext) -> HttpResponse {
        HttpResponse::Ok()
            .content_type(ContentType::xml())
            .body(Sitemap { context }.render_once().unwrap())
    }
}
