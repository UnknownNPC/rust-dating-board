use actix_web::{web, Responder};
use serde::Deserialize;

use crate::{
    db::DbProvider,
    web_api::{
        auth::AuthenticationGate,
        routes::{
            constants::{HACK_DETECTED, RESTRICTED_AREA, SERVER_ERROR},
            util::redirect_to_home_page,
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
    let all_user_profiles = db_provider
        .all_user_profiles(auth_gate.user_id.unwrap())
        .await
        .unwrap();

    let target_profile_model_opt = all_user_profiles
        .iter()
        .find(|profile_model| profile_model.id == taget_profile_id)
        .cloned();
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
                    taget_profile_id,
                    target_profile_photos_len
                );
                redirect_to_home_page(None, None, None, true)
            })
            .map_err(|err| {
                println!("Unable to delete profile! {}", err.to_string());
                redirect_to_home_page(None, Some(SERVER_ERROR), None, false)
            })
            .unwrap()
    } else {
        redirect_to_home_page(None, Some(HACK_DETECTED), None, false)
    }
}

#[derive(Deserialize)]
pub struct DeleteProfileRequest {
    pub id: i64,
}
