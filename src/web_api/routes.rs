use std::error::Error;

use actix_multipart::form::MultipartForm;
use actix_web::{
    cookie::Cookie,
    http::{header, StatusCode},
    web, HttpRequest, HttpResponse, Responder,
};
use futures::future::OptionFuture;

use crate::{
    config::Config,
    db::DbProvider,
    db::ProfilePhotoModel,
    db::UserModel,
    web_api::photo::PhotoService,
    web_api::{auth::AuthSessionManager, html_page::HtmlPage},
};

use super::{auth::AuthenticationGate, model::*, sign_in::get_google_user};

pub async fn index_page(
    db_provider: web::Data<DbProvider>,
    auth_gate: AuthenticationGate,
    query: web::Query<HomeQuery>,
) -> impl Responder {
    println!(
        "[route#index_page] Inside the index page. User auth status {}",
        auth_gate.is_authorized
    );

    let user = if auth_gate.is_authorized {
        db_provider
            .find_user_by_id(auth_gate.user_id.unwrap())
            .await
            .unwrap()
    } else {
        None
    };

    let user_name = user.map(|f| f.name);

    HtmlPage::homepage("Home page", query.error.as_deref(), user_name.as_deref())
}

pub async fn add_profile_page(
    db_provider: web::Data<DbProvider>,
    auth_gate: AuthenticationGate,
    config: web::Data<Config>,
) -> impl Responder {
    println!(
        "[route#add_profile_page] Inside the add_profile page. User auth status {}",
        auth_gate.is_authorized
    );

    if let Err(response) = redirect_to_home_if_not_authorized(&auth_gate) {
        return response;
    }

    let user = db_provider
        .find_user_by_id(auth_gate.user_id.unwrap())
        .await
        .unwrap()
        .unwrap();

    let draft_profile_opt = db_provider.find_draft_profile_for(user.id).await.unwrap();

    let profile_photos = OptionFuture::from(
        draft_profile_opt
            .as_ref()
            .map(|profile| db_provider.find_all_profile_photos_for(profile.id)),
    )
    .await
    .unwrap_or(Ok(vec![]))
    .unwrap();

    let context =
        AddProfilePageContext::new(&config.all_photos_folder_name, user.id, profile_photos);

    HtmlPage::add_profile("Add new profile", &user.name, &context)
}

pub async fn add_profile_photo_endpoint(
    db_provider: web::Data<DbProvider>,
    auth_gate: AuthenticationGate,
    config: web::Data<Config>,
    form: MultipartForm<AddProfilePhotoMultipart>,
) -> impl Responder {
    if !auth_gate.is_authorized {
        return web::Json(AddProfilePhotoResponse::new_with_error("resticted_area"));
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

    let new_file_response = AddProfilePhotoResponse::new_with_payload(
        &config.all_photos_folder_name,
        draft_profile_id,
        vec![profile_photo_db],
    );
    println!("[routes#add_profile_photo_endpoint]: Response is ready");

    web::Json(new_file_response)
}

pub async fn delete_profile_photo_endpoint(
    db_provider: web::Data<DbProvider>,
    auth_gate: AuthenticationGate,
    form: web::Form<DeleteProfilePhotoMultipart>,
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
        return web::Json(DeleteProfilePhotoResponse {
            error: Some("resrited_area".to_owned()),
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
            web::Json(DeleteProfilePhotoResponse::new())
        })
        .map_err(|error| {
            println!(
                "[route#delete_profile_photo_endpoint] IO processing exception. Return Error: {}",
                error
            );
            web::Json(DeleteProfilePhotoResponse::new_with_error("process_error"))
        })
        .unwrap()
    } else {
        println!(
            "[route#delete_profile_photo_endpoint] User {} tries DELETE SOMEONE'S PHOTO {1}. HACCKKKER :3",
            &auth_gate.user_id.unwrap(), &request_profile_photo_id
        );
        web::Json(DeleteProfilePhotoResponse::new_with_error("bad_hacker"))
    };

    response
}

pub async fn sign_out_endpoint(auth_gate: AuthenticationGate) -> impl Responder {
    let empty_cookie = AuthSessionManager::get_empty_jwt_token();
    if auth_gate.is_authorized {
        println!(
            "[route#sign_out_endpoint] auth user {} is loging out",
            auth_gate.user_id.unwrap()
        );
        redirect_to_home_page(Some(empty_cookie), None)
    } else {
        redirect_to_home_page(None, None)
    }
}

pub async fn google_sign_in_endpoint(
    db_provider: web::Data<DbProvider>,
    config: web::Data<Config>,
    callback_payload: web::Form<GoogleSignInPost>,
    request: HttpRequest,
) -> impl Responder {
    async fn fetch_and_save_user(
        db_provider: &web::Data<DbProvider>,
        callback_payload: &web::Form<GoogleSignInPost>,
        config: &web::Data<Config>,
    ) -> Result<UserModel, Box<dyn Error>> {
        let oauth_user = get_google_user(&callback_payload.credential, &config).await?;
        let db_user_opt = db_provider.find_user_by_email(&oauth_user.email).await?;

        let user = if let Some(db_user) = db_user_opt {
            println!(
                "[route#google_sign_in_endpoint] email {} exists. Just reusing",
                &oauth_user.email
            );
            db_user
        } else {
            println!(
                "[route#google_sign_in_endpoint] email {} is new. Creating new user",
                &oauth_user.email
            );
            let new_user_model = db_provider
                .add_user(None, &oauth_user.name, &oauth_user.email, Some("Google"))
                .await?;
            new_user_model
        };

        Ok(user)
    }

    if callback_payload.credential.is_empty() {
        return redirect_to_home_page(None, Some("lost_credential"));
    }
    if callback_payload.g_csrf_token.is_empty() {
        return redirect_to_home_page(None, Some("lost_g_csrf_token"));
    }

    if Some(callback_payload.g_csrf_token.clone())
        != request
            .cookie("g_csrf_token")
            .map(|f| f.value().to_string())
    {
        return redirect_to_home_page(None, Some("invalid_g_csrf_token"));
    }

    let user_res = fetch_and_save_user(&db_provider, &callback_payload, &config).await;

    match user_res {
        Ok(user) => {
            let session_manager = AuthSessionManager::new(&config);
            let jwt_cookie = session_manager.get_valid_jwt_token(user.id).await;
            redirect_to_home_page(Some(jwt_cookie), None)
        }
        Err(err) => {
            println!(
                "[route#google_sign_in_endpoint] error happened during user fetch: {}",
                err
            );
            redirect_to_home_page(None, Some("invalid_user"))
        }
    }
}

fn redirect_to_home_if_not_authorized(auth_gate: &AuthenticationGate) -> Result<(), HttpResponse> {
    if !auth_gate.is_authorized {
        println!(
            "[route#...] endpoint for authorized only. Auth status {}. Redirection!",
            auth_gate.is_authorized
        );
        Result::Err(redirect_to_home_page(None, Some("restricted_area")))
    } else {
        println!(
            "[route#...] endpoint for authorized only. Auth status {}. OK!",
            auth_gate.is_authorized
        );
        Ok(())
    }
}

fn redirect_to_home_page(jwt_cookie: Option<Cookie>, error: Option<&str>) -> HttpResponse {
    let mut response_builder = HttpResponse::build(StatusCode::FOUND);

    if error.is_some() {
        response_builder.append_header((header::LOCATION, format!("/?error={}", error.unwrap())))
    } else {
        response_builder.append_header((header::LOCATION, "/"))
    };
    if jwt_cookie.is_some() {
        response_builder.cookie(jwt_cookie.unwrap());
    };
    response_builder.finish()
}
