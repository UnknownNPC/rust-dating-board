
mod routes;
mod sign_in;
mod auth;

pub use routes::homepage as homepage_endpoint;
pub use routes::google_sign_in as google_sign_in_endpoint;