use actix_web::cookie::Cookie;
use actix_web::http::header;
use actix_web::http::StatusCode;
use actix_web::HttpResponse;

// commmon functions
pub fn redirect_to_home_if_not_authorized(is_authorized: bool) -> Result<(), HttpResponse> {
    if !is_authorized {
        println!(
            "[route#...] endpoint for authorized only. Auth status {}. Redirection!",
            is_authorized
        );
        Result::Err(redirect_to_home_page(None, Some("restricted_area"), None))
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
) -> HttpResponse {
    let mut response_builder = HttpResponse::build(StatusCode::FOUND);

    if error.is_some() {
        response_builder.append_header((header::LOCATION, format!("/?error={}", error.unwrap())))
    } else if msg.is_some() {
        response_builder.append_header((header::LOCATION, format!("/?msg={}", msg.unwrap())))
    } else {
        response_builder.append_header((header::LOCATION, "/"))
    };
    if jwt_cookie.is_some() {
        response_builder.cookie(jwt_cookie.unwrap());
    };
    response_builder.finish()
}
