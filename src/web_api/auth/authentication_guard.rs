use std::future::{ready, Ready};

use actix_web::{
    dev::Payload,
    error::{Error as ActixWebError, ErrorUnauthorized},
    web, FromRequest, HttpRequest,
};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};

use crate::config::Config;

use super::session_manager::TokenClaims;

pub struct AuthenticationGuard {
    pub user_id: i64,
}

impl FromRequest for AuthenticationGuard {
    type Error = ActixWebError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let token = req.cookie("token").map(|c| c.value().to_string());

        if token.is_none() {
            println!("Auth guard didn't find token. Rejecting");
            return ready(Err(ErrorUnauthorized("Auth token doesn't exist")));
        }

        let config = req.app_data::<web::Data<Config>>().unwrap();

        let jwt_secret = config.jwt_secret.to_owned();
        let decode = decode::<TokenClaims>(
            token.unwrap().as_str(),
            &DecodingKey::from_secret(jwt_secret.as_ref()),
            &Validation::new(Algorithm::HS256),
        );

        match decode {
            Ok(token) => {
                let user_id = token.claims.sub.parse::<i64>().unwrap();
                println!("Auth guard found token with id {}", user_id);
                ready(Ok(AuthenticationGuard { user_id: user_id }))
            }
            Err(_) => {
                println!("Auth guard found token but wasn't able to verify it. I guess it was hoooker attack :3");
                ready(Err(ErrorUnauthorized("Invalid token. Are u haker? :3")))
            }
        }
    }
}
