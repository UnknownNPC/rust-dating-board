mod add_profile_page;
mod edit_profile_page;
mod authorization_endpoint;
mod home_page;
mod html_render;
mod common;
mod constants;
mod profile_endpoints;



pub use home_page::index_page;

pub use add_profile_page::add_profile_get;
pub use edit_profile_page::edit_profile_page;
pub use add_profile_page::add_or_edit_profile_post;


pub use profile_endpoints::add_profile_photo_endpoint;
pub use profile_endpoints::delete_profile_photo_endpoint;
pub use profile_endpoints::delete_profile_endpoint;

pub use authorization_endpoint::google_sign_in_endpoint;
pub use authorization_endpoint::sign_out_endpoint;
