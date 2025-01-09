use dotenv::dotenv;

#[derive(Debug, Clone)]
pub struct Config {
    pub site_protocol: String,
    pub site_url: String,
    pub site_port: i64,

    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_max_age: i64,

    pub oauth_google_client_id: String,
    pub oauth_google_client_secret: String,
    pub oauth_google_redirect_url: String,

    pub all_photos_folder_name: String,

    pub captcha_google_id: String,
    pub captcha_google_secret: String,
    pub captcha_google_score: f64,
}

impl Config {
    pub fn init() -> Config {
        dotenv().ok();

        let site_protocol = std::env::var("SITE_PROTOCOL").expect("SITE_PROTOCOL must be set");
        let site_url = std::env::var("SITE_URL").expect("SITE_URL must be set");
        let site_port = std::env::var("SITE_PORT").expect("SITE_PORT must be set");

        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
        let jwt_maxage = std::env::var("JWT_MAXAGE").expect("JWT_MAXAGE must be set");

        let oauth_google_client_id =
            std::env::var("OAUTH_GOOGLE_CLIENT_ID").expect("DATABASE_URL must be set");
        let oauth_google_client_secret = std::env::var("OAUTH_GOOGLE_CLIENT_SECRET")
            .expect("OAUTH_GOOGLE_CLIENT_SECRET must be set");
        let oauth_google_redirect_url = std::env::var("OAUTH_GOOGLE_REDIRECT_URL")
            .expect("OAUTH_GOOGLE_REDIRECT_URL must be set");

        let all_photos_folder_name =
            std::env::var("ALL_PHOTOS_FOLDER_NAME").expect("ALL_PHOTOS_FOLDER_NAME must be set");

        let captcha_google_id =
            std::env::var("CAPTCHA_GOOGLE_ID").expect("CAPTCHA_GOOGLE_ID must be set");
        let captcha_google_secret =
            std::env::var("CAPTCHA_GOOGLE_SECRET").expect("CAPTCHA_GOOGLE_SECRET must be set");
        let captcha_google_score =
            std::env::var("CAPTCHA_GOOGLE_SCORE").expect("CAPTCHA_GOOGLE_SCORE must be set");

        Config {
            site_protocol,
            site_url,
            site_port: site_port.parse::<i64>().unwrap(),
            database_url,
            jwt_secret,
            jwt_max_age: jwt_maxage.parse::<i64>().unwrap(),
            oauth_google_client_id,
            oauth_google_client_secret,
            oauth_google_redirect_url,
            all_photos_folder_name,
            captcha_google_id,
            captcha_google_secret,
            captcha_google_score: captcha_google_score.parse::<f64>().unwrap(),
        }
    }
}
