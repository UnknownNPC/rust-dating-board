use dotenv::dotenv;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expires_in: String,
    pub jwt_maxage: i32,
    pub oauth_google_client_id: String,
    pub oauth_google_client_secret: String,
    pub oauth_google_redirect_url: String,
}

impl Config {
    pub fn init() -> Config {

        dotenv().ok();

        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
        let jwt_expires_in = std::env::var("JWT_EXPIRED_IN").expect("JWT_EXPIRED_IN must be set");
        let jwt_maxage = std::env::var("JWT_MAXAGE").expect("JWT_MAXAGE must be set");

        let oauth_google_client_id=std::env::var("OAUTH_GOOGLE_CLIENT_ID").expect("DATABASE_URL must be set");
        let oauth_google_client_secret=std::env::var("OAUTH_GOOGLE_CLIENT_SECRET").expect("OAUTH_GOOGLE_CLIENT_SECRET must be set");
        let oauth_google_redirect_url=std::env::var("OAUTH_GOOGLE_REDIRECT_URL").expect("OAUTH_GOOGLE_REDIRECT_URL must be set");

        Config {
            database_url,
            jwt_secret,
            jwt_expires_in,
            jwt_maxage: jwt_maxage.parse::<i32>().unwrap(),
            oauth_google_client_id,
            oauth_google_client_secret,
            oauth_google_redirect_url
        }
    }
}
