
mod routes;
mod sign_in;
mod auth;
mod model;
mod html_page;
mod photo;

pub use routes::index_page;
pub use routes::google_sign_in_endpoint;
pub use routes::sign_out_endpoint;
pub use routes::add_profile_page;
pub use routes::add_profile_photo_endpoint;
