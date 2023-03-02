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


pub struct AddProfilePageDto<'a> {
    pub name: &'a str,
    pub height: i16,
    pub description: &'a str,
    pub phone_number: &'a str,
    pub city: &'a str,
    pub photos: Vec<(i64, &'a str)>,
}

impl AddProfilePageDto<'_> {
    pub fn empty() -> Self {
        AddProfilePageDto {
            name: "",
            height: 0,
            description: "",
            phone_number: "",
            city: "",
            photos: Vec::new()
        }
    }
}