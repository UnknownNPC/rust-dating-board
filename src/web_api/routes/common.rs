use crate::db::DbProvider;
use crate::db::{ProfileModel, ProfilePhotoModel};
use actix_web::cookie::Cookie;
use actix_web::http::header;
use actix_web::http::StatusCode;
use actix_web::web;
use actix_web::HttpResponse;
use serde::Serialize;

//common models

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

pub struct ProfilePageDataContext {
    pub id: Option<i64>,
    pub name: String,
    pub height: i16,
    pub description: String,
    pub phone_number: String,
    pub city: String,
    pub init_photos: AddProfilePhotoContext,
    pub is_edit_mode: bool,
}

impl<'a> ProfilePageDataContext {
    pub fn new(
        all_photos_folder: &str,
        profile_opt: &Option<ProfileModel>,
        db_photos: Vec<ProfilePhotoModel>,
        is_edit_mode: bool,
    ) -> Self {
        let profile_photo_response =
            AddProfilePhotoContext::new_with_payload(all_photos_folder, db_photos);

        let id = profile_opt.as_ref().map(|f| f.id);
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
            id,
            name,
            height: height.to_owned(),
            description: description,
            phone_number: phone_number,
            city,
            init_photos: profile_photo_response,
            is_edit_mode,
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
    jwt_cookie_opt: Option<Cookie>,
    error_text_opt: Option<&str>,
    msg_code_opt: Option<&str>,
    to_user_profiles: bool,
) -> HttpResponse {
    let mut response_builder = HttpResponse::build(StatusCode::FOUND);

    let message_param_opt = msg_code_opt.map(|f| format!("msg={}", f));

    if error_text_opt.is_some() {
        response_builder.append_header((header::LOCATION, format!("/?error={}", error_text_opt.unwrap())))
    } else if to_user_profiles {
        let message_param = message_param_opt
            .map(|f| format!("&{}", f))
            .unwrap_or_default();
        response_builder.append_header((
            header::LOCATION,
            format!("/?filter_type=my{}", message_param),
        ))
    } else {
        let message_param = message_param_opt
            .map(|f| format!("?{}", f))
            .unwrap_or_default();
        response_builder.append_header((header::LOCATION, format!("/{}", message_param)))
    };
    if jwt_cookie_opt.is_some() {
        response_builder.cookie(jwt_cookie_opt.unwrap());
    };
    response_builder.finish()
}

pub async fn is_user_profile(
    user_id: i64,
    profile_id: i64,
    db_provider: &web::Data<DbProvider>,
) -> Option<ProfileModel> {
    let all_user_profiles = db_provider.all_user_profiles(user_id).await.unwrap();

    all_user_profiles
        .iter()
        .find(|profile_model| profile_model.id == profile_id)
        .cloned()
}
