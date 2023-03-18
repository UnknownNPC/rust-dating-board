use std::{error::Error, fmt::Display, io};

use actix_web::{
    error,
    http::{header::LOCATION, StatusCode},
    HttpResponse,
};
use jsonwebtoken_google::ParserError;
use sea_orm::DbErr;

use crate::web_api::{
    recaptcha::CaptchaError,
    routes::constant::{
        MSG_BAD_REQUEST_ERROR_CODE, MSG_BOT_DETECTED_ERROR_CODE, MSG_SERVER_ERROR_CODE,
        MSG_UNAUTHORIZED_ERROR_CODE,
    },
};

impl Error for HtmlError {}

#[derive(Debug)]
pub enum HtmlError {
    ServerError,
    NotAuthorized,
    BadParams,
    NotFound,
    BotDetection,
}

impl Display for HtmlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error type: {}", &self)
    }
}

impl From<DbErr> for HtmlError {
    fn from(err: DbErr) -> Self {
        println!("[DbErr] db error happens: [{}]", &err);
        HtmlError::ServerError
    }
}

impl From<CaptchaError> for HtmlError {
    fn from(err: CaptchaError) -> Self {
        println!("[CaptchaError] captcha exception: [{}]", &err);
        HtmlError::BadParams
    }
}

impl From<ParserError> for HtmlError {
    fn from(err: ParserError) -> Self {
        println!("[ParserError] parse exception: [{}]", &err);
        HtmlError::ServerError
    }
}

impl From<io::Error> for HtmlError {
    fn from(err: io::Error) -> Self {
        println!("[io::Error] io exception: [{}]", &err);
        HtmlError::ServerError
    }
}

impl error::ResponseError for HtmlError {
    fn error_response(&self) -> HttpResponse {
        match *self {
            HtmlError::ServerError => homepage(MSG_SERVER_ERROR_CODE),
            HtmlError::NotAuthorized => homepage(MSG_UNAUTHORIZED_ERROR_CODE),
            HtmlError::BadParams => homepage(MSG_BAD_REQUEST_ERROR_CODE),
            HtmlError::BotDetection => homepage(MSG_BOT_DETECTED_ERROR_CODE),
            HtmlError::NotFound => page_404(),
        }
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            HtmlError::ServerError => StatusCode::INTERNAL_SERVER_ERROR,
            HtmlError::NotAuthorized => StatusCode::UNAUTHORIZED,
            HtmlError::BadParams => StatusCode::BAD_REQUEST,
            HtmlError::BotDetection => StatusCode::FORBIDDEN,
            HtmlError::NotFound => StatusCode::NOT_FOUND,
        }
    }
}

pub fn homepage(message: &str) -> HttpResponse {
    let path = format!("/?message={}", message);
    HttpResponse::Found()
        .insert_header((LOCATION, path))
        .finish()
}

pub fn page_404() -> HttpResponse {
    HttpResponse::Found()
        .insert_header((LOCATION, "/404"))
        .finish()
}
