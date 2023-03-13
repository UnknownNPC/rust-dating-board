use std::{error::Error, fmt::Display, io};

use actix_web::{error, http::StatusCode, web, HttpResponse};
use sea_orm::DbErr;
use serde::Serialize;

use crate::web_api::routes::constant::{
    MSG_BAD_REQUEST_ERROR_CODE, MSG_SERVER_ERROR_CODE, MSG_UNAUTHORIZED_ERROR_CODE,
};

impl Error for JsonError {}

#[derive(Serialize, Debug)]
struct JsonErrorPayload<'a> {
    error: &'a str,
}

#[derive(Debug)]
pub enum JsonError {
    ServerError,
    NotAuthorized,
    BadParams,
}

impl Display for JsonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error type: [{}]", &self)
    }
}

impl From<DbErr> for JsonError {
    fn from(err: DbErr) -> Self {
        println!("[DbErr] db error happens: [{}]", &err);
        JsonError::ServerError
    }
}

impl From<io::Error> for JsonError {
    fn from(err: io::Error) -> Self {
        println!("[io::Error] io exception: [{}]", &err);
        JsonError::ServerError
    }
}

impl error::ResponseError for JsonError {
    fn error_response(&self) -> HttpResponse {
        match *self {
            JsonError::ServerError => error_json(MSG_SERVER_ERROR_CODE, &self.status_code()),
            JsonError::NotAuthorized => {
                error_json(MSG_UNAUTHORIZED_ERROR_CODE, &self.status_code())
            }
            JsonError::BadParams => error_json(MSG_BAD_REQUEST_ERROR_CODE, &self.status_code()),
        }
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            JsonError::ServerError => StatusCode::INTERNAL_SERVER_ERROR,
            JsonError::NotAuthorized => StatusCode::UNAUTHORIZED,
            JsonError::BadParams => StatusCode::BAD_REQUEST,
        }
    }
}

fn error_json(msg: &str, status: &StatusCode) -> HttpResponse {
    let response = JsonErrorPayload { error: msg };
    let data = web::Json(response);

    HttpResponse::build(status.to_owned())
        .content_type("application/json")
        .json(data.into_inner())
}
