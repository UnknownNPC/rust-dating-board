use core::fmt;
use std::error::Error;

use awc::{http::header, Client};
use mime::APPLICATION_JSON;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct CaptchaError {
    message: String,
}

impl fmt::Display for CaptchaError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for CaptchaError {}

#[derive(Serialize)]
struct Request {
    secret: String,
    response: String,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct Response {
    success: bool,
    score: Option<f64>,
    action: Option<String>,
    #[serde(rename = "error-codes")]
    error_codes: Option<Vec<String>>,
}

type Score = f64;

#[derive(Clone)]
pub struct Recaptcha {}

impl Recaptcha {
    pub async fn verify(secret: &str, token: &str) -> Result<Score, CaptchaError> {
        let http_client = Client::new();

        let url = format!(
            "https://www.google.com/recaptcha/api/siteverify?secret={}&response={}",
            secret, token
        );

        let mut raw_response = http_client
            .post(url)
            .insert_header((header::CONTENT_TYPE, APPLICATION_JSON))
            .send()
            .await
            .map_err(|err| CaptchaError {
                message: err.to_string()
            })?;

        let response_json_res = raw_response.json::<Response>().await;

        response_json_res
            .map(|response| {
                println!("[Recaptcha#verify] recaptcha RAW response {:?}", response);

                response.score.unwrap_or_default()
            })
            .map_err(|err| CaptchaError {
                message: err.to_string(),
            })
    }
}
