use actix_web::{web, Responder};
use futures::future::OptionFuture;
use serde::Deserialize;

use crate::{
    config::Config,
    db::DbProvider,
    web_api::{
        auth::AuthenticationGate,
        routes::{
            common::{
                redirect_to_home_if_not_authorized, redirect_to_home_page, NavContext,
                ProfilePageDataContext,
            },
            constants::{PROFILE_ADDED, SERVER_ERROR},
            html_render::HtmlPage,
        },
    },
};

pub async fn add_profile_get(
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
        false
    );

    let nav_context = NavContext::new(user.name, cities_names, String::from(""), false);

    HtmlPage::add_or_edit_profile(&nav_context, &data_contex)
}

pub async fn add_or_edit_profile_post(
    db_provider: web::Data<DbProvider>,
    auth_gate: AuthenticationGate,
    form: web::Form<AddOrEditProfileFormRequest>,
) -> impl Responder {

    //TODO form.profile_id means edititng
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

#[derive(Deserialize)]
pub struct AddOrEditProfileFormRequest {
    pub name: String,
    pub height: String,
    pub city: String,
    pub phone_number: String,
    pub description: String,
    // edit mode ON
    pub profile_id: Option<i64>
}
