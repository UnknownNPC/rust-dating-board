
mod routes;
mod sign_in;
mod auth;
mod model;
mod html_page;

pub use routes::index as index_endpoint;
pub use routes::google_sign_in as google_sign_in_endpoint;
pub use routes::sign_out as sign_out_endpoint;
pub use routes::add_profile as add_profile_endpoint;
 