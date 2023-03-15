use actix_web::web;
use actix_web::{http::header::LOCATION, HttpResponse, Responder};
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

use super::validator::Validator;

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
    form_raw: web::Form<AddOrEditProfileFormRequestRaw>,
    config: web::Data<Config>,
) -> Result<impl Responder, HtmlError> {
    async fn create_or_reuse_draft_profile(
        user_id: i64,
        db_provider: &web::Data<DbProvider>,
    ) -> Result<ProfileModel, HtmlError> {
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
            create_or_reuse_draft_profile(user_id, db_provider).await
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

    let user_id = auth_gate.user_id.unwrap();

    let form_validation = form_raw.validate();
    let form = if let Err(error_context) = form_validation {
        let error_json_str = serde_json::to_string(&error_context)?;
        if form_raw.profile_id.as_ref().is_none() {
            // error on submit. Let's create or re-use draft profile and fill with invalid data
            let path = format!("/add_profile?errors={}", error_json_str);
            println!("[route#add_or_edit_profile_post] Form includes errors. Add mode. Redirection to [{}]", &path);
            return Ok(redirect_response_to(path.as_str()));
        } else {
            let id = form_raw.profile_id.unwrap_or_default();
            let path = format!("/edit_profile?id={}&errors={}", id, error_json_str);
            println!("[route#add_or_edit_profile_post] Form includes errors. Edit mode. Redirection to [{}]", &path);
            return Ok(redirect_response_to(path.as_str()));
        }
    } else {
        let form = form_validation.unwrap();
        println!(
            "[route#add_or_edit_profile_post] Form passes validation: [{:?}]",
            &form
        );
        form
    };

    let captcha_score =
        Recaptcha::verify(&config.captcha_google_secret, &form.captcha_token).await?;

    if captcha_score < config.captcha_google_score {
        println!(
            "[routes#add_or_edit_profile_post] google captcha score is low [{}]",
            captcha_score
        );
        return Err(HtmlError::BotDetection);
    }

    let profile_model = resolve_profile(user_id, &form.profile_id, &db_provider).await?;

    let is_edit_mode = form.profile_id.is_some();

    db_provider
        .publish_profie(
            &profile_model,
            &form.name,
            form.height,
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
            let path = if is_edit_mode {
                format!("/?show_my=true&message={}", MSG_PROFILE_UPDATED_CODE)
            } else {
                format!("/?message={}", MSG_PROFILE_ADDED_CODE)
            };
            redirect_response_to(path.as_str())
        })
        .map_err(|err| err.into())
}

fn redirect_response_to(path: &str) -> HttpResponse {
    HttpResponse::Found()
        .append_header((LOCATION, path))
        .finish()
}

#[derive(Deserialize)]
pub struct AddOrEditProfileFormRequestRaw {
    pub name: String,
    pub height: String,
    pub city: String,
    pub phone_number: String,
    pub description: String,
    // edit mode ON
    pub profile_id: Option<i64>,
    pub captcha_token: String,
}

#[derive(Debug)]
pub struct AddOrEditProfileFormRequest {
    pub name: String,
    pub height: i16,
    pub city: String,
    pub phone_number: String,
    pub description: String,
    // edit mode ON
    pub profile_id: Option<i64>,
    pub captcha_token: String,
}

impl AddOrEditProfileFormRequest {
    pub fn from_raw(raw: &AddOrEditProfileFormRequestRaw) -> Self {
        AddOrEditProfileFormRequest {
            name: raw.name.clone(),
            height: raw.height.parse::<i16>().unwrap(),
            city: raw.city.clone(),
            phone_number: raw.phone_number.clone(),
            description: raw.description.clone(),
            profile_id: raw.profile_id,
            captcha_token: raw.captcha_token.clone(),
        }
    }
}
