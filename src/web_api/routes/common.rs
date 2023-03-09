use crate::db::{ProfileModel, ProfilePhotoModel};
use actix_web::cookie::Cookie;
use actix_web::http::header;
use actix_web::http::StatusCode;
use actix_web::HttpResponse;
use serde::Deserialize;
use serde::Serialize;

//common models
#[derive(Deserialize)]
pub enum QueryFilterTypeRequest {
    #[serde(rename = "my")]
    My,
}

#[derive(Deserialize)]
pub struct QueryRequest {
    pub error: Option<String>,
    pub filter_type: Option<QueryFilterTypeRequest>,
    pub filter_city: Option<String>,
    pub page: Option<u64>,
}

pub struct NavContext {
    pub name: String,
    pub all_cities: Vec<String>,
    pub current_city: String,
    pub is_user_profiles: bool,
}

impl NavContext {
    pub fn new(
        name: String,
        cities: Vec<String>,
        current_city: String,
        is_user_profiles: bool,
    ) -> Self {
        NavContext {
            name,
            all_cities: cities,
            current_city,
            is_user_profiles,
        }
    }
}

pub struct ActionContext<'a> {
    pub error_msg: &'a str,
}

impl<'a> ActionContext<'a> {
    pub fn new(error_msg: &'a str) -> Self {
        ActionContext { error_msg }
    }
}

pub struct ProfilePageDataContext {
    pub name: String,
    pub height: i16,
    pub description: String,
    pub phone_number: String,
    pub city: String,
    pub init_photos: AddProfilePhotoContext,
}

impl<'a> ProfilePageDataContext {
    pub fn new(
        all_photos_folder: &str,
        profile_opt: &Option<ProfileModel>,
        db_photos: Vec<ProfilePhotoModel>,
    ) -> Self {
        let profile_photo_response =
            AddProfilePhotoContext::new_with_payload(all_photos_folder, db_photos);

        let name = profile_opt
            .as_ref()
            .map(|f| f.name.clone())
            .unwrap_or_default();
        let height = &profile_opt.as_ref().map(|f| f.height).unwrap_or(0);
        let description = profile_opt
            .as_ref()
            .map(|f| f.description.clone())
            .unwrap_or_default();
        let phone_number = profile_opt
            .as_ref()
            .map(|f| f.phone_number.clone())
            .unwrap_or_default();
        let city = profile_opt
            .as_ref()
            .map(|f| f.city.clone())
            .unwrap_or_default();
        ProfilePageDataContext {
            name: name,
            height: height.to_owned(),
            description: description,
            phone_number: phone_number,
            city: city,
            init_photos: profile_photo_response,
        }
    }
}

#[derive(Serialize, Debug)]
pub struct AddProfilePhotoContext {
    pub error: Option<String>,
    #[serde(rename = "initialPreview")]
    pub initial_preview: Vec<String>,
    #[serde(rename = "initialPreviewConfig")]
    pub initial_preview_config: Vec<ProfilePhotoPreviewContext>,
    pub append: bool,
}

impl<'a> AddProfilePhotoContext {
    pub fn new_with_error(error: &str) -> Self {
        AddProfilePhotoContext {
            error: Some(error.to_string()),
            initial_preview: vec![],
            initial_preview_config: vec![],
            append: true,
        }
    }

    pub fn new_with_payload(all_photos_folder: &'a str, db_photos: Vec<ProfilePhotoModel>) -> Self {
        let photo_urls = db_photos
            .iter()
            .map(|db_photo| {
                all_photos_folder.to_owned()
                    + "/"
                    + &db_photo.profile_id.to_string()
                    + "/"
                    + &db_photo.file_name
            })
            .collect();

        let photo_confings = db_photos
            .iter()
            .map(|db_photo| {
                ProfilePhotoPreviewContext::new(db_photo.id, &db_photo.file_name, db_photo.size)
            })
            .collect();

        AddProfilePhotoContext {
            error: None,
            initial_preview: photo_urls,
            initial_preview_config: photo_confings,
            append: true,
        }
    }
}

#[derive(Serialize, Debug)]
pub struct ProfilePhotoPreviewContext {
    // photo name
    pub caption: String,
    pub size: i64,
    // delete url
    pub url: String,
    // profile photo id
    pub key: i64,
}

impl<'a> ProfilePhotoPreviewContext {
    pub fn new(profile_photo_id: i64, os_photo_filename: &'a str, os_file_size: i64) -> Self {
        ProfilePhotoPreviewContext {
            caption: os_photo_filename.to_string(),
            size: os_file_size,
            url: String::from("/profile_photo/delete"),
            key: profile_photo_id,
        }
    }
}

// commmon functions
pub fn redirect_to_home_if_not_authorized(is_authorized: bool) -> Result<(), HttpResponse> {
    if !is_authorized {
        println!(
            "[route#...] endpoint for authorized only. Auth status {}. Redirection!",
            is_authorized
        );
        Result::Err(redirect_to_home_page(
            None,
            Some("restricted_area"),
            None,
            false,
        ))
    } else {
        println!(
            "[route#...] endpoint for authorized only. Auth status {}. OK!",
            is_authorized
        );
        Ok(())
    }
}

pub fn redirect_to_home_page(
    jwt_cookie: Option<Cookie>,
    error: Option<&str>,
    msg: Option<&str>,
    to_user_profiles: bool,
) -> HttpResponse {
    let mut response_builder = HttpResponse::build(StatusCode::FOUND);

    if error.is_some() {
        response_builder.append_header((header::LOCATION, format!("/?error={}", error.unwrap())))
    } else if msg.is_some() {
        response_builder.append_header((header::LOCATION, format!("/?msg={}", msg.unwrap())))
    } else if to_user_profiles {
        response_builder.append_header((header::LOCATION, "/?filter_type=my"))
    } else {
        response_builder.append_header((header::LOCATION, "/"))
    };
    if jwt_cookie.is_some() {
        response_builder.cookie(jwt_cookie.unwrap());
    };
    response_builder.finish()
}
