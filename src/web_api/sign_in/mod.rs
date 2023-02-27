
mod google;

pub use google::get_google_user;


#[derive(Debug)]
pub struct OAuthUser {
    pub email: String,
    pub name: String,
}
