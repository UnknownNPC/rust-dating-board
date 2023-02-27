use std::error::Error;

use jsonwebtoken_google::Parser;
use serde::{Deserialize, Serialize};

use crate::config::Config;

use super::OAuthUser;

#[derive(Serialize, Deserialize)]
pub struct TokenClaims {
    pub email: String,
    pub name: String,
}

pub async fn get_google_user(
    rawJwtCredentail: &str,
    config: &Config,
) -> Result<OAuthUser, Box<dyn Error>> {

    let parser = Parser::new(&config.oauth_google_client_id);
    let claims = parser.parse::<TokenClaims>(rawJwtCredentail).await?;

    Ok(OAuthUser { 
        email: claims.email,
        name: claims.name
     })
}
