use actix_web::{web, Responder};
use futures::future::OptionFuture;
use serde::Deserialize;

use crate::{
    config::Config,
    db::{DbProvider, ProfileModel},
    web_api::{
        auth::AuthenticationGate,
        routes::{
            common::{
                redirect_to_home_if_not_authorized, redirect_to_home_page, NavContext,
                ProfilePageDataContext,
            },
            constants::{HACK_DETECTED, PROFILE_ADDED, SERVER_ERROR, PROFILE_UPDATED, CAPTCHA_CHECK_ERROR},
            html_render::HtmlPage,
        }, recaptcha::Recaptcha,
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
        false,
    );

    let nav_context = NavContext::new(user.name, cities_names, String::from(""), false,
    false, config.captcha_google_id.clone());

    HtmlPage::add_or_edit_profile(&nav_context, &data_contex)
}

pub async fn add_or_edit_profile_post(
    db_provider: web::Data<DbProvider>,
    auth_gate: AuthenticationGate,
    form: web::Form<AddOrEditProfileFormRequest>,
    config: web::Data<Config>,
) -> impl Responder {
    async fn resolve_profile(
        profile_id_opt: &Option<i64>,
        user_id: i64,
        db_provider: &web::Data<DbProvider>,
    ) -> Result<ProfileModel, String> {
        if profile_id_opt.is_some() {
            let profile_id = profile_id_opt.as_ref().unwrap().to_owned();
            let profile_opt = db_provider
                .find_active_profile_by(profile_id)
                .await
                .unwrap();
            if let Some(profile) = profile_opt {
                println!("[routes#add_or_edit_profile_post] Edit profile flow. Found active profile. Re-useing");
                Ok(profile)
            } else {
                println!(
                    "[routes#add_or_edit_profile_post] Cant find profile for photo id. Hack! User {}",
                    user_id
                );
                Err(HACK_DETECTED.to_string())
            }
        } else {
            // Find old or create new draft profile
            let draft_profile_opt = db_provider.find_draft_profile_for(user_id).await.unwrap();

            if let Some(draft_profile) = draft_profile_opt {
                println!("[routes#add_or_edit_profile_post] Draft profile flow. Found draft profile. Re-useing");
                Ok(draft_profile)
            } else {
                println!("[routes#add_or_edit_profile_post] Draft profile flow. Creating new draft profile");
                db_provider
                    .add_draft_profile_for(user_id)
                    .await
                    .map_err(|_| SERVER_ERROR.to_string())
            }
        }
    }

    println!(
        "[route#add_or_edit_profile_post] Inside the add_profile post. User auth status {}",
        auth_gate.is_authorized
    );

    let captcha_verify_res = Recaptcha::verify(&config.captcha_google_secret, &form.captcha_token).await;
    if let Err(response) = captcha_verify_res {
        println!("[routes#add_or_edit_page] google captcha flow failed [{}]", response);
        return redirect_to_home_page(None, Some(SERVER_ERROR), None, false)
    }

    let captcha_score = captcha_verify_res.unwrap();
    if captcha_score < config.captcha_google_score {
        println!("[routes#add_or_edit_page] google captcha score is low [{}]", captcha_score );
        return redirect_to_home_page(None, Some(CAPTCHA_CHECK_ERROR), None, false)
    }

    if let Err(response) = redirect_to_home_if_not_authorized(auth_gate.is_authorized) {
        return response;
    }

    let user_id = auth_gate.user_id.unwrap();
    let profile_model_or_err = resolve_profile(&form.profile_id, user_id, &db_provider).await;

    let profile_model = match profile_model_or_err {
        Ok(profile) => profile,
        Err(err) => return redirect_to_home_page(None, Some(err.as_str()), None, false),
    };
    let is_edit_mode = form.profile_id.is_some();

    db_provider
        .publish_profie(
            profile_model,
            &form.name,
            form.height.parse::<i16>().unwrap(),
            &form.city,
            &form.description,
            &form.phone_number,
        )
        .await
        .map(|_| {
            println!("[route#add_or_edit_profile_post] Advert was updated and published. Redirect to users profiles: {}", is_edit_mode);
            let response_code = if is_edit_mode { PROFILE_UPDATED } else { PROFILE_ADDED }; 
            redirect_to_home_page(None, None, Some(response_code), is_edit_mode)
        })
        .map_err(|_| {
            println!("[route#add_or_edit_profile_post] Error. Advert wasn't published");
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
    pub profile_id: Option<i64>,
    pub captcha_token: String
}
