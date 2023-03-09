use actix_web::{web, Responder};
use serde::Deserialize;

use crate::{
    config::Config,
    db::{DbProvider},
    web_api::{
        auth::AuthenticationGate,
        routes::{
            common::{is_user_profile, redirect_to_home_page, NavContext, ProfilePageDataContext},
            constants::{HACK_DETECTED, RESTRICTED_AREA},
            html_render::HtmlPage,
        },
    },
};

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
            true
        );

        let user = db_provider
            .find_user_by_id(auth_gate.user_id.unwrap())
            .await
            .unwrap()
            .unwrap();

        let nav_context = NavContext::new(user.name, cities_names, String::from(""), false);

        HtmlPage::add_or_edit_profile(&nav_context, &data_contex)
    } else {
        println!("Profile id is invalid. Hack detected from user [{}]!", auth_gate.user_id.unwrap());
        redirect_to_home_page(None, Some(HACK_DETECTED), None, false)
    }
}

#[derive(Deserialize)]
pub struct EditProfileRequest {
    pub id: i64,
}
