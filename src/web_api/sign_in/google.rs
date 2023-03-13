use jsonwebtoken_google::{Parser, ParserError};
use serde::{Deserialize, Serialize};

use super::OAuthUser;

#[derive(Serialize, Deserialize)]
pub struct TokenClaims {
    pub email: String,
    pub name: String,
}

pub async fn get_google_user(
    raw_jwt_credentail: &str,
    oauth_google_client_id: &str,
) -> Result<OAuthUser, ParserError> {
    let parser = Parser::new(oauth_google_client_id);
    let claims = parser.parse::<TokenClaims>(raw_jwt_credentail).await?;

    Ok(OAuthUser {
        email: claims.email,
        name: claims.name,
    })
}
