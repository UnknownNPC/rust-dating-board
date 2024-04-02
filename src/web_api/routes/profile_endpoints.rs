use crate::web_api::photo::PhotoService;
use crate::web_api::routes::constant::MAX_PROFILE_PHOTOS;
use crate::web_api::routes::error::HtmlError;
use crate::web_api::routes::error::JsonError;
use crate::{
    config::Config,
    db::{DbProvider, ProfileModel, ProfilePhotoModel},
    web_api::{auth::AuthenticationGate, routes::common::AddProfilePhotoContext},
};
use actix_multipart::form::tempfile::TempFile;
use actix_multipart::form::text::Text;
use actix_multipart::form::MultipartForm;
use actix_web::http::header::LOCATION;
use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse, Responder};
use futures::future;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub async fn delete_profile_endpoint(
    db_provider: web::Data<DbProvider>,
    auth_gate: AuthenticationGate,
    form: web::Form<DeleteProfileRequest>,
    config: web::Data<Config>,
) -> Result<impl Responder, HtmlError> {
    if !auth_gate.is_authorized {
        return Err(HtmlError::NotAuthorized);
    }

    println!(
        "[route#delete_profile_endpoint] User auth status: [{}]. User ID: [{}]",
        auth_gate.is_authorized,
        auth_gate.user_id.unwrap_or_default()
    );

    let profile_id = form.id;
    let profile_opt = db_provider
        .find_active_profile_by_id_and_user_id(&profile_id, auth_gate.user_id.unwrap())
        .await?;
    let profile = profile_opt.ok_or(HtmlError::NotFound)?;
    let profile_photos = db_provider.find_all_profile_photos_for(&profile.id).await?;

    println!(
        "[route#delete_profile_endpoint] Deleting profile: [{}]. Starting IO",
        &profile_id
    );

    db_provider
        .delete_profile_and_photos(&profile, &profile_photos)
        .await?;

    PhotoService::delete_profile_from_fs(&config.all_photos_folder_name, &profile_id)?;

    Ok(HttpResponse::build(StatusCode::FOUND)
        .append_header((LOCATION, "/?show_my=true"))
        .finish())
}

pub async fn add_profile_photo_endpoint(
    db_provider: web::Data<DbProvider>,
    auth_gate: AuthenticationGate,
    config: web::Data<Config>,
    form: MultipartForm<AddProfilePhotoMultipartRequest>,
) -> Result<impl Responder, JsonError> {
    async fn resolve_profile(
        user_id: i64,
        profile_id_opt: &Option<Uuid>,
        db_provider: &web::Data<DbProvider>,
    ) -> Result<ProfileModel, JsonError> {
        if profile_id_opt.is_some() {
            println!("[routes#add_profile_photo_endpoint] Edit flow. Searching active profile");
            let profile_id = profile_id_opt.unwrap_or_default();
            db_provider
                .find_active_profile_by_id_and_user_id(&profile_id, user_id)
                .await?
                .ok_or(JsonError::BadParams)
        } else {
            let draft_profile_opt = db_provider.find_draft_profile_for(user_id).await?;
            match draft_profile_opt {
                Some(draft_profile) => {
                    println!("[routes#add_profile_photo_endpoint] Draft profile flow. Found draft profile. Re-useing");
                    Ok(draft_profile)
                }
                None => {
                    println!("[routes#add_profile_photo_endpoint] Draft profile flow. Creating new draft profile");
                    db_provider
                        .add_draft_profile_for(user_id)
                        .await
                        .map_err(|op| op.into())
                }
            }
        }
    }

    if !auth_gate.is_authorized {
        return Err(JsonError::NotAuthorized);
    }

    println!(
        "[route#add_profile_photo_endpoint] User auth status: [{}]. User ID: [{}]",
        auth_gate.is_authorized,
        auth_gate.user_id.unwrap_or_default()
    );

    let user_id = auth_gate.user_id.unwrap();
    let profile_id_opt = form.0.profile_id.map(|f| f.0);
    let profile = resolve_profile(user_id, &profile_id_opt, &db_provider).await?;
    let profile_photos = db_provider.count_profile_photos(&profile.id).await?;
    if profile_photos > MAX_PROFILE_PHOTOS.to_owned() {
        return Err(JsonError::BadParams);
    }

    async fn process_image(
        new_profile_photo: &TempFile,
        config: &Config,
        profile_id: &Uuid,
        db_provider: &web::Data<DbProvider>,
    ) -> Result<ProfilePhotoModel, JsonError> {
        let photo_fs_save_result = PhotoService::save_photo_on_fs(
            new_profile_photo,
            &config.all_photos_folder_name,
            profile_id,
        )?;

        println!(
            "[routes#add_profile_photo_endpoint]: Photo saved into fs with name: [{:?}]",
            &photo_fs_save_result
        );
        let save_result = db_provider
            .add_profile_photo(
                profile_id,
                &photo_fs_save_result.name.as_str(),
                photo_fs_save_result.size,
            )
            .await;

        println!(
            "[routes#add_profile_photo_endpoint]: Photo saved into database with id: [{:?}]",
            &save_result
        );
        save_result.map_err(|err| err.into())
    }

    let db_photos =
        future::try_join_all(form.0.new_profile_photos.iter().map(|new_profile_photo| {
            process_image(new_profile_photo, &config, &profile.id, &db_provider)
        }))
        .await
        .unwrap();

    let response =
        AddProfilePhotoContext::new_with_payload(&config.all_photos_folder_name, &db_photos);
    Ok(web::Json(response))
}

pub async fn delete_profile_photo_endpoint(
    db_provider: web::Data<DbProvider>,
    auth_gate: AuthenticationGate,
    form: web::Form<DeleteProfilePhotoFormRequest>,
    config: web::Data<Config>,
) -> Result<impl Responder, JsonError> {
    async fn process_deleting(
        profile_id: &Uuid,
        profile_photo: &ProfilePhotoModel,
        db_provider: &web::Data<DbProvider>,
        config: &web::Data<Config>,
    ) -> Result<(), JsonError> {
        db_provider
            .update_profile_photo_with_delete_status(profile_photo)
            .await?;

        PhotoService::delete_photo_from_fs(
            &config.all_photos_folder_name,
            &profile_id,
            &profile_photo.file_name,
        )
        .map_err(|_| JsonError::BadParams)
    }

    if !auth_gate.is_authorized {
        return Err(JsonError::NotAuthorized);
    }

    println!(
        "[route#delete_profile_photo_endpoint] User auth status: [{}]. User ID: [{}]",
        auth_gate.is_authorized,
        auth_gate.user_id.unwrap_or_default()
    );

    let user_id = auth_gate.user_id.unwrap();

    let profile_photo_id: i64 = form.0.key.parse().unwrap();
    let profile_photo_profile_opt = db_provider
        .find_active_profile_photo_with_profile_by_id_and_user_id(profile_photo_id, user_id)
        .await?;
    let (profile_photo, profile) = profile_photo_profile_opt.ok_or(JsonError::BadParams)?;

    process_deleting(&profile.id, &profile_photo, &db_provider, &config)
        .await
        .map(|_| {
            println!("[route#delete_profile_photo_endpoint] IO actions were done. Deleted: OK!");
            web::Json(DeleteProfilePhotoJsonResponse::new())
        })
}

#[derive(MultipartForm)]
pub struct AddProfilePhotoMultipartRequest {
    pub new_profile_photos: Vec<TempFile>,
    // it means edit mode
    pub profile_id: Option<Text<Uuid>>,
}

#[derive(Deserialize)]
pub struct DeleteProfileRequest {
    pub id: Uuid,
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
    pub fn new() -> Self {
        DeleteProfilePhotoJsonResponse { error: None }
    }
}
