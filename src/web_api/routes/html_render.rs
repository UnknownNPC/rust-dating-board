use actix_web::HttpResponse;
use sailfish::TemplateOnce;

use super::add_page::AddProfilePageContext;

#[derive(TemplateOnce)]
#[template(path = "home.stpl")]
struct Home<'a> {
    error_msg: &'a str,
    user_name: &'a str,
}

#[derive(TemplateOnce)]
#[template(path = "add_profile.stpl")]
struct AddProfile<'a> {
    user_name: &'a str,
    profile_context: &'a AddProfilePageContext,
}

pub struct HtmlPage;

impl HtmlPage {
    pub fn homepage(error_msg: Option<&str>, user_name: Option<&str>) -> HttpResponse {
        let html = HttpResponse::Ok().body(
            Home {
                error_msg: error_msg.unwrap_or(""),
                user_name: user_name.unwrap_or(""),
            }
            .render_once()
            .unwrap(),
        );

        html
    }

    pub fn add_profile(user_name: &str, profile_context: &AddProfilePageContext) -> HttpResponse {
        HttpResponse::Ok().body(
            AddProfile {
                user_name: user_name,
                profile_context,
            }
            .render_once()
            .unwrap(),
        )
    }
}
