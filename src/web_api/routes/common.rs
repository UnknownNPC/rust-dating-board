use super::constant::NO_PHOTO_URL;
use crate::{
    config::Config,
    db::{ProfileModel, ProfilePhotoModel},
};
use serde::Serialize;
use uuid::Uuid;

pub struct HeadContext {
    pub title: String,
    pub description: String,
    pub preview_url: String,
}

impl HeadContext {
    pub fn new(
        title: &str,
        description: &str,
        config: &Config,
        profile_photo_opt: &Option<ProfilePhotoModel>,
    ) -> Self {
        let site_photo_url = profile_photo_opt
            .as_ref()
            .map(|profile_photo| {
                let relative_path = format!(
                    "/{}",
                    get_relative_photo_url(&profile_photo, &config.all_photos_folder_name)
                );
                get_absolute_url(&config, &relative_path)
            })
            .unwrap_or(get_absolute_url(&config, &NO_PHOTO_URL.to_string()));
        HeadContext {
            title: title.to_owned(),
            description: description.to_owned(),
            preview_url: site_photo_url,
        }
    }
}

pub struct NavContext {
    pub name: String,
    pub all_cities: Vec<String>,
    pub current_city: String,
    pub is_user_profiles: bool,
    pub search: Option<String>,
    pub google_captcha_id: String,
    pub google_oauth_client_id: String,
    pub google_oauth_sign_in_url: String,
}

impl NavContext {
    pub fn new(
        name: &str,
        current_city: &str,
        google_captcha_id: &str,
        is_user_profiles: bool,
        search: &Option<String>,
        cities: &Vec<String>,
        google_oauth_client_id: &str,
        google_oauth_sign_in_url: &str,
    ) -> Self {
        NavContext {
            name: name.to_owned(),
            all_cities: cities.to_owned(),
            current_city: current_city.to_owned(),
            is_user_profiles,
            search: search.clone(),
            google_captcha_id: google_captcha_id.to_owned(),
            google_oauth_client_id: google_oauth_client_id.to_owned(),
            google_oauth_sign_in_url: google_oauth_sign_in_url.to_owned(),
        }
    }
}

pub struct ProfilePageDataContext {
    pub id: Option<Uuid>,
    pub name: String,
    pub height: i16,
    pub weight: i16,
    pub description: String,
    pub phone_number: String,
    pub city: String,
    pub init_photos: AddProfilePhotoContext,
    pub is_edit_mode: bool,
}

impl ProfilePageDataContext {
    pub fn new(
        all_photos_folder: &str,
        profile_opt: &Option<ProfileModel>,
        db_photos: &Vec<ProfilePhotoModel>,
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
        let weight = &profile_opt.as_ref().map(|f| f.weight).unwrap_or(0);
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
            weight: weight.to_owned(),
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

pub fn get_absolute_url(config: &Config, path: &str) -> String {
    if config.site_port == 80 || config.site_port == 443 {
        format!("{}://{}{}", &config.site_protocol, &config.site_url, &path)
    } else {
        format!(
            "{}://{}:{}{}",
            &config.site_protocol, &config.site_url, &config.site_port, &path
        )
    }
}

pub fn get_relative_photo_url(
    profile_photo: &ProfilePhotoModel,
    all_photos_folder: &str,
) -> String {
    all_photos_folder.to_owned()
        + "/"
        + &profile_photo.profile_id.to_string()
        + "/"
        + &profile_photo.file_name
}

impl<'a> AddProfilePhotoContext {
    pub fn new_with_payload(
        all_photos_folder: &'a str,
        db_photos: &Vec<ProfilePhotoModel>,
    ) -> Self {
        let photo_urls = db_photos
            .iter()
            .map(|db_photo| get_relative_photo_url(db_photo, all_photos_folder))
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
