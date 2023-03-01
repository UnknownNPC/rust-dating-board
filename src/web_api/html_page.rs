use actix_web::HttpResponse;
use sailfish::TemplateOnce;

#[derive(TemplateOnce)]
#[template(path = "home.stpl")]
struct Home<'a> {
    head_title: &'a str,
    error_msg: &'a str,

    is_authorized: bool,
    user_name: &'a str,
}

pub struct HtmlPage;

impl HtmlPage {
    pub fn homepage(
        title: &str,
        error_msg: Option<&str>,
        is_authorized: bool,
        user_name: Option<&str>,
    ) -> HttpResponse {
        let html = HttpResponse::Ok().body(
            Home {
                head_title: title,
                error_msg: error_msg.unwrap_or(""),
                is_authorized,
                user_name: user_name.unwrap_or(""),
            }
            .render_once()
            .unwrap(),
        );

        html
    }
}
