use actix_web::{web, Responder};
use serde::Deserialize;

use crate::{
    config::Config,
    db::{DbProvider, ProfileModel},
    web_api::{
        auth::AuthenticationGate,
        routes::{
            common::{redirect_to_home_page, NavContext, ProfilePageDataContext},
            constants::{HACK_DETECTED, RESTRICTED_AREA, SERVER_ERROR},
            html_render::HtmlPage,
        },
    },
};

pub async fn delete_user_profile(
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
            .delete_profile(target_profile, target_profile_photos)
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

pub async fn edit_profile_page(
    db_provider: web::Data<DbProvider>,
    auth_gate: AuthenticationGate,
    query: web::Query<EditProfileRequest>,
    config: web::Data<Config>,
) -> impl Responder {
    println!(
        "[route#edit_user_profile] Inside the edit profile endpoint. User auth status {}",
        auth_gate.is_authorized
    );

    if !auth_gate.is_authorized {
        return redirect_to_home_page(None, Some(RESTRICTED_AREA), None, false);
    }

    let taget_profile_id = query.id;

    let target_profile_model_opt =
        is_user_profile(auth_gate.user_id.unwrap(), taget_profile_id, &db_provider).await;

    if target_profile_model_opt.is_some() {
        let target_profile_photos = db_provider
            .find_all_profile_photos_for(taget_profile_id)
            .await
            .unwrap();

        let cities_models = db_provider.find_cities_on().await.unwrap();
        let cities_names = cities_models.iter().map(|city| city.name.clone()).collect();

        let data_contex = ProfilePageDataContext::new(
            &config.all_photos_folder_name,
            &target_profile_model_opt,
            target_profile_photos,
        );

        let user = db_provider
            .find_user_by_id(auth_gate.user_id.unwrap())
            .await
            .unwrap()
            .unwrap();

        let nav_context = NavContext::new(user.name, cities_names, String::from(""), false);

        HtmlPage::add_profile(&nav_context, &data_contex)
    } else {
        println!("Hack detected from user [{}]!", auth_gate.user_id.unwrap());
        redirect_to_home_page(None, Some(HACK_DETECTED), None, false)
    }
}

async fn is_user_profile(
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

#[derive(Deserialize)]
pub struct DeleteProfileRequest {
    pub id: i64,
}

#[derive(Deserialize)]
pub struct EditProfileRequest {
    pub id: i64,
}
