mod add_profile_page;
mod authorization_endpoint;
mod home_page;
mod html_render;
mod common;
mod manage_profile_service;
mod constants;


pub use add_profile_page::add_profile_page;
pub use add_profile_page::add_profile_photo_endpoint;
pub use add_profile_page::add_profile_post;
pub use add_profile_page::delete_profile_photo_endpoint;
pub use authorization_endpoint::google_sign_in_endpoint;
pub use authorization_endpoint::sign_out_endpoint;
pub use home_page::index_page;
pub use manage_profile_service::delete_profile_endpoint;
pub use manage_profile_service::edit_profile_page;
