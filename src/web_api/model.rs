use serde::Deserialize;

#[derive(Deserialize)]
pub struct GoogleSignInPost {
    pub credential: String,
    pub g_csrf_token: String,
}

#[derive(Deserialize)]
pub struct HomeQuery {
    pub error: Option<String>,
}
