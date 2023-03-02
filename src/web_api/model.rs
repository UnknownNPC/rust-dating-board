use crate::db::ProfilePhotoModel;
use actix_multipart::form::{tempfile::TempFile, text::Text, MultipartForm};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct GoogleSignInPost {
    pub credential: String,
    pub g_csrf_token: String,
}

#[derive(Deserialize)]
pub struct HomeQuery {
    pub error: Option<String>,
}

pub struct AddProfilePageContext {
    pub name: String,
    pub height: i16,
    pub description: String,
    pub phone_number: String,
    pub city: String,
    pub init_photos: AddProfilePhotoResponse,
}

impl<'a> AddProfilePageContext {
    pub fn new(
        all_photos_folder: &str,
        profile_id: i64,
        db_photos: Vec<ProfilePhotoModel>,
    ) -> Self {
        let profile_photo_response =
            AddProfilePhotoResponse::new_with_payload(all_photos_folder, profile_id, db_photos);
        AddProfilePageContext {
            name: String::from(""),
            height: 0,
            description: String::from(""),
            phone_number: String::from(""),
            city: String::from(""),
            init_photos: profile_photo_response,
        }
    }
}

#[derive(MultipartForm)]
pub struct AddProfilePhotoMultipart {
    #[multipart(rename = "fileId")]
    pub file_id: Text<String>,
    pub new_profile_photo: TempFile,
}

#[derive(Serialize, Debug)]
pub struct AddProfilePhotoResponse {
    pub error: Option<String>,
    pub initialPreview: Vec<String>,
    pub initialPreviewConfig: Vec<ProfilePhotoPreviewConfigResponse>,
    pub append: bool,
}

impl<'a> AddProfilePhotoResponse {
    pub fn new_with_error(error: &str) -> Self {
        AddProfilePhotoResponse {
            error: Some(error.to_string()),
            initialPreview: vec![],
            initialPreviewConfig: vec![],
            append: true,
        }
    }

    pub fn new_with_payload(
        all_photos_folder: &'a str,
        profile_id: i64,
        db_photos: Vec<ProfilePhotoModel>,
    ) -> Self {
        let photo_urls = db_photos
            .iter()
            .map(|db_photo| {
                all_photos_folder.to_owned()
                    + "/"
                    + &profile_id.to_string()
                    + "/"
                    + &db_photo.original_file_name
            })
            .collect();

        let photo_confings = db_photos
            .iter()
            .map(|db_photo| 
                ProfilePhotoPreviewConfigResponse::new(db_photo.id, 
                    &db_photo.original_file_name, 0)
            ).collect();

        AddProfilePhotoResponse {
            error: None,
            initialPreview: photo_urls,
            initialPreviewConfig: photo_confings,
            append: true,
        }
    }
}

#[derive(Serialize, Debug)]
pub struct ProfilePhotoPreviewConfigResponse {
    // photo name
    pub caption: String,
    // placeholder
    pub description: String,
    pub size: i64,
    // delete url
    pub url: String,
    // profile photo id
    pub key: i64,
}

impl<'a> ProfilePhotoPreviewConfigResponse {
    pub fn new(profile_photo_id: i64, os_photo_filename: &'a str, os_file_size: i64) -> Self {
        let description = format!("profile_photo_{}", profile_photo_id);
        let delete_photo_url = format!("/profile_photo/upload/{}", profile_photo_id);
        ProfilePhotoPreviewConfigResponse {
            caption: os_photo_filename.to_string(),
            description: description,
            size: os_file_size,
            url: delete_photo_url,
            key: profile_photo_id,
        }
    }
}
