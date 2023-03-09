use std::error::Error;

use crate::web_api::photo::PhotoService;
use crate::web_api::routes::constants::PROCESS_ERROR;
use crate::{
    config::Config,
    db::{DbProvider, ProfileModel, ProfilePhotoModel},
    web_api::{
        auth::AuthenticationGate,
        routes::{
            common::{is_user_profile, redirect_to_home_page, AddProfilePhotoContext},
            constants::{HACK_DETECTED, RESTRICTED_AREA, SERVER_ERROR},
        },
    },
};
use actix_multipart::form::tempfile::TempFile;
use actix_multipart::form::text::Text;
use actix_multipart::form::MultipartForm;
use actix_web::{web, Responder};
use serde::{Deserialize, Serialize};

pub async fn delete_profile_endpoint(
    db_provider: web::Data<DbProvider>,
    auth_gate: AuthenticationGate,
    form: web::Form<DeleteProfileRequest>,
) -> impl Responder {
    println!(
        "[route#delete_user_profile] Inside the delete profile endpoint. User auth status {}",
        auth_gate.is_authorized
    );

    if !auth_gate.is_authorized {
        return redirect_to_home_page(None, Some(RESTRICTED_AREA), None, false);
    }

    let taget_profile_id = form.id;

    let target_profile_model_opt =
        is_user_profile(auth_gate.user_id.unwrap(), taget_profile_id, &db_provider).await;

    if target_profile_model_opt.is_some() {
        let target_profile = target_profile_model_opt.unwrap();
        let target_profile_photos = db_provider
            .find_all_profile_photos_for(target_profile.id)
            .await
            .unwrap();
        let target_profile_photos_len = target_profile_photos.len();

        db_provider
            .delete_profile_and_photos(target_profile, target_profile_photos)
            .await
            .map(|_| {
                println!(
                    "User profile {} with photos {} was marked as deleted!",
                    taget_profile_id, target_profile_photos_len
                );
                redirect_to_home_page(None, None, None, true)
            })
            .map_err(|err| {
                println!("Unable to delete profile! {}", err.to_string());
                redirect_to_home_page(None, Some(SERVER_ERROR), None, false)
            })
            .unwrap()
    } else {
        println!("Hack detected from user [{}]!", auth_gate.user_id.unwrap());
        redirect_to_home_page(None, Some(HACK_DETECTED), None, false)
    }
}

pub async fn add_profile_photo_endpoint(
    db_provider: web::Data<DbProvider>,
    auth_gate: AuthenticationGate,
    config: web::Data<Config>,
    form: MultipartForm<AddProfilePhotoMultipartRequest>,
) -> impl Responder {
    if !auth_gate.is_authorized {
        return web::Json(AddProfilePhotoContext::new_with_error(RESTRICTED_AREA));
    }

    let user_id = auth_gate.user_id.unwrap();

    let profile_model: ProfileModel = if form.profile_id.is_some() {
        let profile_id = form.profile_id.as_ref().unwrap().0;
        let profile_opt = db_provider
            .find_active_profile_by(profile_id)
            .await
            .unwrap();
        if let Some(profile) = profile_opt {
            println!("[routes#add_profile_photo_endpoint] Edit profile flow. Found active profile. Re-useing");
            profile
        } else {
            println!(
                "[routes#add_profile_photo_endpoint] Cant find profile for photo id. Hack! User {}",
                user_id
            );
            return web::Json(AddProfilePhotoContext::new_with_error(HACK_DETECTED));
        }
    } else {
        // Find old or create new draft profile
        let draft_profile_opt = db_provider.find_draft_profile_for(user_id).await.unwrap();

        let draft_profile = if let Some(draft_profile) = draft_profile_opt {
            println!("[routes#add_profile_photo_endpoint] Draft profile flow. Found draft profile. Re-useing");
            draft_profile
        } else {
            println!("[routes#add_profile_photo_endpoint] Draft profile flow. Creating new draft profile");
            db_provider.add_draft_profile_for(user_id).await.unwrap()
        };
        draft_profile
    };

    if profile_model.user_id != user_id {
        println!("[routes#add_profile_photo_endpoint] User doesnt own this profile");
        return web::Json(AddProfilePhotoContext::new_with_error(HACK_DETECTED));
    }

    //Save photo to FS for this profile
    let photo_fs_save_result = PhotoService::save_photo_on_fs(
        &form.0.new_profile_photo,
        &config.all_photos_folder_name,
        profile_model.id,
    )
    .unwrap();
    println!("[routes#add_profile_photo_endpoint]: Photo saved into fs");

    //Save profile photo db entity
    let profile_photo_db = db_provider
        .add_profile_photo(
            profile_model.id,
            &photo_fs_save_result.name.as_str(),
            photo_fs_save_result.size,
        )
        .await
        .unwrap();
    println!("[routes#add_profile_photo_endpoint]: Photo saved into database");

    let new_file_response = AddProfilePhotoContext::new_with_payload(
        &config.all_photos_folder_name,
        vec![profile_photo_db],
    );
    println!("[routes#add_profile_photo_endpoint]: Response is ready");

    web::Json(new_file_response)
}

pub async fn delete_profile_photo_endpoint(
    db_provider: web::Data<DbProvider>,
    auth_gate: AuthenticationGate,
    form: web::Form<DeleteProfilePhotoFormRequest>,
    config: web::Data<Config>,
) -> impl Responder {
    async fn process_deleting(
        profile_id: i64,
        profile_photo: ProfilePhotoModel,
        db_provider: &web::Data<DbProvider>,
        config: &web::Data<Config>,
    ) -> Result<(), Box<dyn Error>> {
        let target_profile_photo_name = profile_photo.file_name.clone();
        db_provider
            .update_profile_photo_with_delete_status(profile_photo)
            .await?;

        PhotoService::delete_photo_from_fs(
            &config.all_photos_folder_name,
            profile_id,
            &target_profile_photo_name,
        )
    }

    if !auth_gate.is_authorized {
        return web::Json(DeleteProfilePhotoJsonResponse {
            error: Some(RESTRICTED_AREA.to_owned()),
        });
    }
    let user_id = auth_gate.user_id.unwrap();

    let request_profile_photo_id: i64 = form.0.key.parse().unwrap();
    let profile_photo_opt = db_provider
        .find_active_profile_photo_with_profile_by(request_profile_photo_id)
        .await
        .unwrap();

    if let Some(profile_data) = profile_photo_opt {
        let photo_profile_model = profile_data.0;
        let profile_model = profile_data.1;

        if profile_model.user_id == user_id {
            println!(
                "[route#delete_profile_photo_endpoint] User {} requested profile photo {} delete. OK",
                &auth_gate.user_id.unwrap(),
                &request_profile_photo_id
            );

            process_deleting(
                profile_model.id,
                photo_profile_model,
                &db_provider,
                &config,
            )
            .await
            .map(|_| {
                println!("[route#delete_profile_photo_endpoint] IO actions were done. Deleted: OK!");
                web::Json(DeleteProfilePhotoJsonResponse::new())
            })
            .map_err(|error| {
                println!(
                    "[route#delete_profile_photo_endpoint] IO processing exception. Return Error: {}",
                    error
                );
                web::Json(DeleteProfilePhotoJsonResponse::new_with_error(
                    PROCESS_ERROR,
                ))
            })
            .unwrap()
        } else {
            println!(
                "[route#delete_profile_photo_endpoint] User {} tries someones photo {}. HACK? :3",
                user_id, &request_profile_photo_id
            );
            return web::Json(DeleteProfilePhotoJsonResponse::new_with_error(
                HACK_DETECTED,
            ));
        }
    } else {
        println!(
            "[route#delete_profile_photo_endpoint] User {} tries to delete unknown photo {}. HACK? :3",
            user_id, &request_profile_photo_id
        );
        return web::Json(DeleteProfilePhotoJsonResponse::new_with_error(
            HACK_DETECTED,
        ));
    }
}

#[derive(MultipartForm)]
pub struct AddProfilePhotoMultipartRequest {
    #[multipart(rename = "fileId")]
    pub file_id: Text<String>,
    pub new_profile_photo: TempFile,
    // it means edit mode
    pub profile_id: Option<Text<i64>>,
}

#[derive(Deserialize)]
pub struct DeleteProfileRequest {
    pub id: i64,
}

#[derive(Deserialize)]
pub struct DeleteProfilePhotoFormRequest {
    pub key: String,
}

#[derive(Serialize, Debug)]
pub struct DeleteProfilePhotoJsonResponse {
    pub error: Option<String>,
}

impl<'a> DeleteProfilePhotoJsonResponse {
    pub fn new_with_error(error: &str) -> Self {
        DeleteProfilePhotoJsonResponse {
            error: Some(error.to_string()),
        }
    }

    pub fn new() -> Self {
        DeleteProfilePhotoJsonResponse { error: None }
    }
}
