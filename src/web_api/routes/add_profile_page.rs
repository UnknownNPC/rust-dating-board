use actix_web::web;
use actix_web::{
    http::{header::LOCATION, StatusCode},
    HttpResponse, Responder,
};
use futures::future::OptionFuture;
use serde::Deserialize;

use crate::web_api::routes::error::HtmlError;
use crate::{
    config::Config,
    db::{DbProvider, ProfileModel},
    web_api::{
        auth::AuthenticationGate,
        recaptcha::Recaptcha,
        routes::{
            common::{NavContext, ProfilePageDataContext},
            constant::{MSG_PROFILE_ADDED_CODE, MSG_PROFILE_UPDATED_CODE},
            html_render::HtmlPage,
        },
    },
};

pub async fn add_profile_page(
    db_provider: web::Data<DbProvider>,
    auth_gate: AuthenticationGate,
    config: web::Data<Config>,
) -> Result<impl Responder, HtmlError> {
    if !auth_gate.is_authorized {
        return Err(HtmlError::NotAuthorized);
    }

    println!(
        "[route#add_profile_page] User auth status: [{}]. User ID: [{}]",
        auth_gate.is_authorized,
        auth_gate.user_id.unwrap_or_default()
    );

    let user_id = auth_gate.user_id.unwrap();
    let draft_profile_opt = db_provider.find_draft_profile_for(user_id).await?;

    let draft_profile_photos = OptionFuture::from(
        draft_profile_opt
            .as_ref()
            .map(|profile| db_provider.find_all_profile_photos_for(profile.id)),
    )
    .await
    .unwrap_or(Ok(vec![]))?;

    let cities_names = db_provider.find_city_names().await?;

    let data_contex = ProfilePageDataContext::new(
        &config.all_photos_folder_name,
        &draft_profile_opt,
        &draft_profile_photos,
        false,
    );

    let user_name = auth_gate.user_name.unwrap();
    let nav_context = NavContext::new(
        &user_name,
        "",
        &config.captcha_google_id,
        false,
        false,
        &cities_names,
    );

    Ok(HtmlPage::add_or_edit_profile(&nav_context, &data_contex))
}

pub async fn add_or_edit_profile_post(
    db_provider: web::Data<DbProvider>,
    auth_gate: AuthenticationGate,
    form: web::Form<AddOrEditProfileFormRequest>,
    config: web::Data<Config>,
) -> Result<impl Responder, HtmlError> {
    async fn resolve_profile(
        user_id: i64,
        profile_id_opt: &Option<i64>,
        db_provider: &web::Data<DbProvider>,
    ) -> Result<ProfileModel, HtmlError> {
        if profile_id_opt.is_some() {
            let profile_id = profile_id_opt.unwrap_or_default();
            db_provider
                .find_active_profile_by_id_and_user_id(profile_id, user_id)
                .await?
                .ok_or(HtmlError::BadParams)
        } else {
            let draft_profile_opt = db_provider.find_draft_profile_for(user_id).await?;
            match draft_profile_opt {
                Some(draft_profile) => {
                    println!("[routes#add_or_edit_profile_post] Draft profile flow. Found draft profile. Re-useing");
                    Ok(draft_profile)
                }
                None => {
                    println!("[routes#add_or_edit_profile_post] Draft profile flow. Creating new draft profile");
                    db_provider
                        .add_draft_profile_for(user_id)
                        .await
                        .map_err(|op| op.into())
                }
            }
        }
    }

    println!(
        "[route#add_or_edit_profile_post] User auth status: [{}]. User ID: [{}]",
        auth_gate.is_authorized,
        auth_gate.user_id.unwrap_or_default()
    );

    if !auth_gate.is_authorized {
        return Err(HtmlError::NotAuthorized);
    }

    let captcha_score =
        Recaptcha::verify(&config.captcha_google_secret, &form.captcha_token).await?;

    if captcha_score < config.captcha_google_score {
        println!(
            "[routes#add_or_edit_profile_post] google captcha score is low [{}]",
            captcha_score
        );
        return Err(HtmlError::BotDetection);
    }

    let user_id = auth_gate.user_id.unwrap();
    let profile_model = resolve_profile(user_id, &form.profile_id, &db_provider).await?;

    let is_edit_mode = form.profile_id.is_some();

    db_provider
        .publish_profie(
            &profile_model,
            &form.name,
            form.height.parse::<i16>().unwrap(),
            &form.city,
            &form.description,
            &form.phone_number,
        )
        .await
        .map(|_| {
            println!(
                "[route#add_or_edit_profile_post] Profile [{}] was updated and published. Edit mode: {}",
                profile_model.id, is_edit_mode
            );
            let response_code = if is_edit_mode {
                MSG_PROFILE_UPDATED_CODE
            } else {
                MSG_PROFILE_ADDED_CODE
            };
            let path = format!("/?message={}", response_code);
            HttpResponse::build(StatusCode::FOUND)
                .append_header((LOCATION, path))
                .finish()
        })
        .map_err(|err| err.into())
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
    pub captcha_token: String,
}
