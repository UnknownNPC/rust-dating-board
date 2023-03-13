use std::{error::Error, fmt::Display, io};

use actix_web::{
    error,
    http::{
        header::{ContentType, LOCATION},
        StatusCode,
    },
    HttpResponse,
};
use jsonwebtoken_google::ParserError;
use sea_orm::DbErr;

use crate::web_api::recaptcha::CaptchaError;

use super::constant::{
    MSG_BAD_REQUEST_ERROR_CODE, MSG_BOT_DETECTED_ERROR_CODE, MSG_SERVER_ERROR_CODE,
    MSG_UNAUTHORIZED_ERROR_CODE,
};

impl Error for WebApiError {}

#[derive(Debug)]
pub enum WebApiError {
    WebServerError,
    NotAuthorized,
    BadParams,
    BotDetection,
}

impl Display for WebApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error type: {}", &self)
    }
}

impl From<DbErr> for WebApiError {
    fn from(err: DbErr) -> Self {
        println!("[WebApiError] db error happens: [{}]", &err);
        WebApiError::WebServerError
    }
}

impl From<CaptchaError> for WebApiError {
    fn from(_err: CaptchaError) -> Self {
        WebApiError::BadParams
    }
}

impl From<ParserError> for WebApiError {
    fn from(_err: ParserError) -> Self {
        WebApiError::WebServerError
    }
}

impl From<io::Error> for WebApiError {
    fn from(_err: io::Error) -> Self {
        WebApiError::WebServerError
    }
}

impl error::ResponseError for WebApiError {
    fn error_response(&self) -> HttpResponse {
        match *self {
            WebApiError::WebServerError => homepage(MSG_SERVER_ERROR_CODE),
            WebApiError::NotAuthorized => homepage(MSG_UNAUTHORIZED_ERROR_CODE),
            WebApiError::BadParams => homepage(MSG_BAD_REQUEST_ERROR_CODE),
            WebApiError::BotDetection => homepage(MSG_BOT_DETECTED_ERROR_CODE),
        }
    }


    fn status_code(&self) -> StatusCode {
        match *self {
            WebApiError::WebServerError => StatusCode::INTERNAL_SERVER_ERROR,
            WebApiError::NotAuthorized => StatusCode::UNAUTHORIZED,
            WebApiError::BadParams => StatusCode::BAD_REQUEST,
            WebApiError::BotDetection => StatusCode::FORBIDDEN,
        }
    }
}

pub fn homepage(message: &str) -> HttpResponse {
    let path = format!("/?message={}", message);
    HttpResponse::build(StatusCode::FOUND)
        .insert_header((LOCATION, path))
        .insert_header(ContentType::html())
        .finish()
}
