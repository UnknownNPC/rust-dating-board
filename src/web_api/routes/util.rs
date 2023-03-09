use actix_web::cookie::Cookie;
use actix_web::http::header;
use actix_web::http::StatusCode;
use actix_web::HttpResponse;
use serde::Deserialize;

//common models

#[derive(Deserialize)]
pub enum QueryFilterTypeRequest {
    #[serde(rename = "my")]
    My,
}

#[derive(Deserialize)]
pub struct QueryRequest {
    pub error: Option<String>,
    pub filter_type: Option<QueryFilterTypeRequest>,
    pub filter_city: Option<String>,
    pub page: Option<u64>,
}

pub struct NavContext {
    pub name: String,
    pub all_cities: Vec<String>,
    pub current_city: String,
    pub is_user_profiles: bool,
}

impl NavContext {
    pub fn new(
        name: String,
        cities: Vec<String>,
        current_city: String,
        is_user_profiles: bool,
    ) -> Self {
        NavContext {
            name,
            all_cities: cities,
            current_city,
            is_user_profiles,
        }
    }
}

pub struct ActionContext<'a> {
    pub error_msg: &'a str,
}

impl<'a> ActionContext<'a> {
    pub fn new(error_msg: &'a str) -> Self {
        ActionContext { error_msg }
    }
}

// commmon functions
pub fn redirect_to_home_if_not_authorized(is_authorized: bool) -> Result<(), HttpResponse> {
    if !is_authorized {
        println!(
            "[route#...] endpoint for authorized only. Auth status {}. Redirection!",
            is_authorized
        );
        Result::Err(redirect_to_home_page(
            None,
            Some("restricted_area"),
            None,
            false,
        ))
    } else {
        println!(
            "[route#...] endpoint for authorized only. Auth status {}. OK!",
            is_authorized
        );
        Ok(())
    }
}

pub fn redirect_to_home_page(
    jwt_cookie: Option<Cookie>,
    error: Option<&str>,
    msg: Option<&str>,
    to_user_profiles: bool,
) -> HttpResponse {
    let mut response_builder = HttpResponse::build(StatusCode::FOUND);

    if error.is_some() {
        response_builder.append_header((header::LOCATION, format!("/?error={}", error.unwrap())))
    } else if msg.is_some() {
        response_builder.append_header((header::LOCATION, format!("/?msg={}", msg.unwrap())))
    } else if to_user_profiles {
        response_builder.append_header((header::LOCATION, "/?filter_type=my"))
    } else {
        response_builder.append_header((header::LOCATION, "/"))
    };
    if jwt_cookie.is_some() {
        response_builder.cookie(jwt_cookie.unwrap());
    };
    response_builder.finish()
}
