use actix_web::cookie::time::Duration as ActixWebDuration;
use actix_web::cookie::Cookie;
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};

use crate::config::Config;

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String,
    pub iat: usize,
    pub exp: usize,
}

pub struct SessionManager<'a> {
    config: &'a Config,
}

impl<'cfg> SessionManager<'cfg> {

    pub fn new(config: &'cfg Config) -> Self {
        Self { config }
    }

    pub fn get_empty_jwt_token() -> Cookie<'cfg> {
        Cookie::build("token", "")
            .path("/")
            .max_age(ActixWebDuration::new(-1, 0))
            .http_only(true)
            .finish()
    }

    pub async fn get_valid_jwt_token(&self, user_id: i64) -> Cookie {
        let jwt_secret = &self.config.jwt_secret;
        let now = Utc::now();
        let iat = now.timestamp() as usize;
        let exp = (now + Duration::minutes(self.config.jwt_max_age)).timestamp() as usize;
        let claims: TokenClaims = TokenClaims {
            sub: user_id.to_string(),
            exp,
            iat,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(jwt_secret.as_ref()),
        )
        .unwrap();

        Cookie::build("token", token)
            .path("/")
            .max_age(ActixWebDuration::new(60 * self.config.jwt_max_age, 0))
            .http_only(true)
            .finish()
    }
}
