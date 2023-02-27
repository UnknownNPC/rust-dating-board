
mod google;

pub use google::get_google_user;

pub struct OAuthUser {
    pub email: String,
    pub name: String,
}
