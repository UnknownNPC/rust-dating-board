use std::error::Error;

use actix_multipart::form::{tempfile::TempFile, text::Text, MultipartForm};
use actix_web::{web, Responder};
use futures::future::OptionFuture;
use serde::{Deserialize, Serialize};

use crate::{
    config::Config,
    db::DbProvider,
    db::ProfilePhotoModel,
    web_api::photo::PhotoService,
    web_api::{
        auth::AuthenticationGate,
        routes::{
            common::{
                redirect_to_home_if_not_authorized, redirect_to_home_page, AddProfilePhotoContext,
                NavContext, ProfilePageDataContext,
            },
            constants::{
                HACK_DETECTED, PROCESS_ERROR, PROFILE_ADDED, RESTRICTED_AREA, SERVER_ERROR,
            },
            html_render::HtmlPage,
        },
    },
};

pub async fn add_profile_page(
    db_provider: web::Data<DbProvider>,
    auth_gate: AuthenticationGate,
    config: web::Data<Config>,
) -> impl Responder {
    println!(
        "[route#add_profile_page] Inside the add_profile page. User auth status {}",
        auth_gate.is_authorized
    );

    if let Err(response) = redirect_to_home_if_not_authorized(auth_gate.is_authorized) {
        return response;
    }

    let user = db_provider
        .find_user_by_id(auth_gate.user_id.unwrap())
        .await
        .unwrap()
        .unwrap();

    //do not want create draft profile on page opening
    let draft_profile_opt = db_provider.find_draft_profile_for(user.id).await.unwrap();

    let profile_photos = OptionFuture::from(
        draft_profile_opt
            .as_ref()
            .map(|profile| db_provider.find_all_profile_photos_for(profile.id)),
    )
    .await
    .unwrap_or(Ok(vec![]))
    .unwrap();

    let cities_models = db_provider.find_cities_on().await.unwrap();
    let cities_names = cities_models.iter().map(|city| city.name.clone()).collect();

    let data_contex = ProfilePageDataContext::new(
        &config.all_photos_folder_name,
        &draft_profile_opt,
        profile_photos,
    );

    let nav_context = NavContext::new(user.name, cities_names, String::from(""), false);

    HtmlPage::add_profile(&nav_context, &data_contex)
}

pub async fn add_profile_post(
    db_provider: web::Data<DbProvider>,
    auth_gate: AuthenticationGate,
    form: web::Form<AddProfileFormRequest>,
) -> impl Responder {
    println!(
        "[route#add_profile_post] Inside the add_profile post. User auth status {}",
        auth_gate.is_authorized
    );

    if let Err(response) = redirect_to_home_if_not_authorized(auth_gate.is_authorized) {
        return response;
    }

    let user_id = auth_gate.user_id.unwrap();

    let draft_profile_opt = db_provider.find_draft_profile_for(user_id).await.unwrap();
    let draft_profile = match draft_profile_opt {
        Some(profile_model) => {
            println!("[route#add_profile_post] Draft exists. Re-using");
            profile_model
        }
        None => {
            println!("[route#add_profile_post] Draft profile wasn't find. Creating new");
            db_provider.add_draft_profile_for(user_id).await.unwrap()
        }
    };

    let height = form.height.parse::<i16>().unwrap();
    db_provider
        .publish_profie(
            draft_profile,
            &form.name,
            height,
            &form.city,
            &form.description,
            &form.phone_number,
        )
        .await
        .map(|_| {
            println!("[route#add_profile_post] Advert was updated and published");
            redirect_to_home_page(None, None, Some(PROFILE_ADDED), false)
        })
        .map_err(|_| {
            println!("[route#add_profile_post] Error. Advert wasn't published");
            redirect_to_home_page(None, Some(SERVER_ERROR), None, false)
        })
        .unwrap()
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

    // Find old or create new draft profile
    let draft_profile_opt = db_provider.find_draft_profile_for(user_id).await.unwrap();
    let draft_profile_id = if draft_profile_opt.is_some() {
        println!("[routes#add_profile_photo_endpoint]: Found draft profile. Re-useing");
        draft_profile_opt.unwrap().id
    } else {
        println!("[routes#add_profile_photo_endpoint]: Creating new draft profile");
        db_provider.add_draft_profile_for(user_id).await.unwrap().id
    };

    //Save photo to FS for this profile
    let photo_fs_save_result = PhotoService::save_photo_on_fs(
        &form.0.new_profile_photo,
        &config.all_photos_folder_name,
        draft_profile_id,
    )
    .unwrap();
    println!("[routes#add_profile_photo_endpoint]: Photo saved into fs");

    //Save profile photo db entity
    let profile_photo_db = db_provider
        .add_profile_photo(
            draft_profile_id,
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
    form: web::Form<DeleteProfilePhotoMultipartRequest>,
    config: web::Data<Config>,
) -> impl Responder {
    async fn process_deleting(
        profile_id: i64,
        profile_photo_id: i64,
        profile_photos: Vec<ProfilePhotoModel>,
        db_provider: &web::Data<DbProvider>,
        config: &web::Data<Config>,
    ) -> Result<(), Box<dyn Error>> {
        let target_profile_photo = profile_photos
            .iter()
            .find(|element| element.id == profile_photo_id)
            .unwrap()
            .to_owned();
        let target_profile_photo_name = target_profile_photo.file_name.clone();
        db_provider
            .update_profile_photo_with_delete_status(target_profile_photo)
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

    let request_profile_photo_id: i64 = form.0.key.parse().unwrap();

    let draft_profile_opt = db_provider
        .find_draft_profile_for(auth_gate.user_id.unwrap())
        .await
        .unwrap();

    let profile_photos = OptionFuture::from(
        draft_profile_opt
            .as_ref()
            .map(|profile| db_provider.find_all_profile_photos_for(profile.id)),
    )
    .await
    .unwrap_or(Ok(vec![]))
    .unwrap();

    let draft_profile_photos_ids: Vec<i64> = profile_photos.iter().map(|f| f.id).collect();
    let request_id_is_valid = draft_profile_photos_ids.contains(&request_profile_photo_id);

    let response = if request_id_is_valid {
        println!(
            "[route#delete_profile_photo_endpoint] User {} requested profile photo {1} delete. OK",
            &auth_gate.user_id.unwrap(),
            &request_profile_photo_id
        );

        process_deleting(
            draft_profile_opt.as_ref().unwrap().id,
            request_profile_photo_id,
            profile_photos,
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
            "[route#delete_profile_photo_endpoint] User {} tries DELETE SOMEONE'S PHOTO {1}. HACCKKKER :3",
            &auth_gate.user_id.unwrap(), &request_profile_photo_id
        );
        web::Json(DeleteProfilePhotoJsonResponse::new_with_error(
            HACK_DETECTED,
        ))
    };

    response
}

#[derive(MultipartForm)]
pub struct AddProfilePhotoMultipartRequest {
    #[multipart(rename = "fileId")]
    pub file_id: Text<String>,
    pub new_profile_photo: TempFile,
}

#[derive(Deserialize)]
pub struct AddProfileFormRequest {
    pub name: String,
    pub height: String,
    pub city: String,
    pub phone_number: String,
    pub description: String,
}

#[derive(Deserialize)]
pub struct DeleteProfilePhotoMultipartRequest {
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
