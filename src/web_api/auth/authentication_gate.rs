use std::future::{ready, Ready};

use actix_web::{dev::Payload, error::Error as ActixWebError, web, FromRequest, HttpRequest};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};

use crate::config::Config;

use super::session_manager::TokenClaims;

pub struct AuthenticationGate {
    pub is_authorized: bool,
    pub user_id: Option<i64>,
    pub user_name: Option<String>,
    pub user_email: Option<String>,
}

impl AuthenticationGate {
    fn empty() -> Self {
        AuthenticationGate {
            is_authorized: false,
            user_id: None,
            user_name: None,
            user_email: None,
        }
    }
}

impl FromRequest for AuthenticationGate {
    type Error = ActixWebError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let token = req.cookie("token").map(|c| c.value().to_string());

        if token.is_none() {
            println!("[authentication_gate] Token doesn't exist. Exit");
            return ready(Ok(AuthenticationGate::empty()));
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
                let id = token.claims.sub.parse::<i64>().unwrap();
                let name = token.claims.name;
                let email = token.claims.email;
                println!(
                    "[authentication_gate] JWT token user id [{}], name: [{}], email: [{}]",
                    &id, &name, &email
                );
                ready(Ok(AuthenticationGate {
                    is_authorized: true,
                    user_id: Some(id),
                    user_name: Some(name),
                    user_email: Some(email),
                }))
            }
            Err(_) => {
                println!("[authentication_gate] Found token but wasn't able to verify it. I guess it was hoooker attack :3");
                ready(Ok(AuthenticationGate::empty()))
            }
        }
    }
}
