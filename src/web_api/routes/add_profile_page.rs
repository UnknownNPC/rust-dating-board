use core::fmt;

use actix_web::web;
use actix_web::{http::header::LOCATION, HttpResponse, Responder};
use futures::future::OptionFuture;
use log::{error, info};
use serde::Deserialize;
use uuid::Uuid;

use crate::web_api::routes::common::HeadContext;
use crate::web_api::routes::error::HtmlError;
use crate::web_api::routes::validator::ErrorContext;
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
use rust_i18n::t;

use super::validator::Validator;

pub async fn add_profile_page(
    db_provider: web::Data<DbProvider>,
    auth_gate: AuthenticationGate,
    config: web::Data<Config>,
) -> Result<impl Responder, HtmlError> {
    if !auth_gate.is_authorized {
        return Err(HtmlError::NotAuthorized);
    }

    info!(
        "User auth status: [{}]. User ID: [{}]",
        auth_gate.is_authorized,
        auth_gate.user_id.unwrap_or_default()
    );

    let user_id = auth_gate.user_id.unwrap();
    let draft_profile_opt = db_provider.find_draft_profile_for(user_id).await?;

    let draft_profile_photos = OptionFuture::from(
        draft_profile_opt
            .as_ref()
            .map(|profile| db_provider.find_all_profile_photos_for(&profile.id)),
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
        &Option::None,
        &cities_names,
        &config.oauth_google_client_id,
        &config.oauth_google_redirect_url,
    );
    let error_context = ErrorContext::empty();
    let head_context = HeadContext::new(
        t!("add_profile_page_title").to_string().as_str(),
        t!("add_profile_page_description").to_string().as_str(),
        &config,
        &Option::None,
    );

    Ok(HtmlPage::add_or_edit_profile(
        &head_context,
        &nav_context,
        &data_contex,
        &error_context,
    ))
}

pub async fn add_or_edit_profile_post(
    db_provider: web::Data<DbProvider>,
    auth_gate: AuthenticationGate,
    form_raw: web::Form<AddOrEditProfileFormRequestRaw>,
    config: web::Data<Config>,
) -> Result<impl Responder, HtmlError> {
    async fn resolve_profile(
        user_id: i64,
        profile_id_opt: &Option<Uuid>,
        db_provider: &web::Data<DbProvider>,
    ) -> Result<ProfileModel, HtmlError> {
        if profile_id_opt.is_some() {
            let profile_id = profile_id_opt.unwrap();
            info!("Active profile flow. Edit flow. Profile: {}", profile_id);
            db_provider
                .find_active_profile_by_id_and_user_id(&profile_id, user_id)
                .await?
                .ok_or(HtmlError::BadParams)
        } else {
            let draft_profile_opt = db_provider.find_draft_profile_for(user_id).await?;
            match draft_profile_opt {
                Some(draft_profile) => {
                    info!("Draft profile flow. Found draft profile. Re-useing");
                    Ok(draft_profile)
                }
                None => {
                    info!("Draft profile flow. Creating new draft profile");
                    db_provider
                        .add_draft_profile_for(user_id)
                        .await
                        .map_err(|op| op.into())
                }
            }
        }
    }

    fn update_profile_with_raw_data(
        profile: &mut ProfileModel,
        form_raw: &web::Form<AddOrEditProfileFormRequestRaw>,
    ) {
        profile.name = form_raw.name.clone();
        if let Ok(height) = form_raw.height.parse::<i16>() {
            profile.height = height
        } else {
            profile.height = 0
        }
        if let Ok(weight) = form_raw.weight.parse::<i16>() {
            profile.weight = weight
        } else {
            profile.weight = 0
        }
        profile.city = form_raw.city.clone();
        profile.phone_number = form_raw.phone_number.clone();
        profile.description = form_raw.description.clone()
    }

    info!(
        "User auth status: [{}]. User ID: [{}]",
        auth_gate.is_authorized,
        auth_gate.user_id.unwrap_or_default()
    );

    if !auth_gate.is_authorized {
        return Err(HtmlError::NotAuthorized);
    }

    let user_id = auth_gate.user_id.unwrap();

    let form_validation = form_raw.validate();
    let form = if let Err(error_context) = form_validation {
        //if data with error
        info!(
            "Form includes errors: [{:?}]. Buidling contexts...",
            &error_context
        );
        let user_id = auth_gate.user_id.unwrap();
        let user_name = auth_gate.user_name.unwrap();
        let google_captcha_id = &config.captcha_google_id.as_str();
        let cities = db_provider.find_city_names().await?;
        let nav_context = NavContext::new(
            &user_name,
            "",
            google_captcha_id,
            false,
            &Option::None,
            &cities,
            &config.oauth_google_client_id,
            &config.oauth_google_redirect_url,
        );

        let mut profile = resolve_profile(user_id, &form_raw.profile_id, &db_provider).await?;
        update_profile_with_raw_data(&mut profile, &form_raw);

        let profile_photos = db_provider.find_all_profile_photos_for(&profile.id).await?;
        let is_edit = form_raw.profile_id.is_some();

        let data_context = ProfilePageDataContext::new(
            &config.all_photos_folder_name,
            &Some(profile),
            &profile_photos,
            is_edit,
        );

        let title = if data_context.is_edit_mode {
            t!("edit_profile_page_title").to_string()
        } else {
            t!("add_profile_page_title").to_string()
        };
        let description = if data_context.is_edit_mode {
            t!("edit_profile_page_description").to_string()
        } else {
            t!("add_profile_page_description").to_string()
        };
        let head_context = HeadContext::new(
            title.as_str(),
            description.as_str(),
            &config,
            &profile_photos.first().cloned(),
        );

        return Ok(HtmlPage::add_or_edit_profile(
            &head_context,
            &nav_context,
            &data_context,
            &error_context,
        ));
    } else {
        let form = form_validation.unwrap();
        info!("Form passes validation: [{:?}]", &form);
        form
    };

    let captcha_score =
        Recaptcha::verify(&config.captcha_google_secret, &form.captcha_token).await?;

    if captcha_score < config.captcha_google_score {
        error!("Google captcha score is low [{}]", captcha_score);
        return Err(HtmlError::BotDetection);
    }

    let profile_model = resolve_profile(user_id, &form.profile_id, &db_provider).await?;

    let is_edit_mode = form.profile_id.is_some();

    let new_db_profile = db_provider
        .publish_profie(
            &profile_model,
            &form.name,
            form.height,
            form.weight,
            &form.city,
            &form.description,
            &form.phone_number,
        )
        .await?;

    info!(
        "Profile [{}] was updated and published. Edit mode: {}",
        new_db_profile.id, is_edit_mode
    );
    let path = if is_edit_mode {
        format!("/?show_my=true&message={}", MSG_PROFILE_UPDATED_CODE)
    } else {
        format!("/?message={}", MSG_PROFILE_ADDED_CODE)
    };
    Ok(redirect_response_to(path.as_str()))
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
    pub weight: String,
    pub city: String,
    pub phone_number: String,
    pub description: String,
    // edit mode ON
    pub profile_id: Option<Uuid>,
    pub captcha_token: String,
}

pub struct AddOrEditProfileFormRequest {
    pub name: String,
    pub height: i16,
    pub weight: i16,
    pub city: String,
    pub phone_number: String,
    pub description: String,
    // edit mode ON
    pub profile_id: Option<Uuid>,
    pub captcha_token: String,
}

impl fmt::Debug for AddOrEditProfileFormRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug_struct = f.debug_struct("AddOrEditProfileFormRequest");
        debug_struct.field("name", &self.name);
        debug_struct.field("height", &self.height);
        debug_struct.field("weight", &self.weight);
        debug_struct.field("city", &self.city);
        debug_struct.field("phone_number", &self.phone_number);
        debug_struct.field("description", &self.description);
        if let Some(profile_id) = self.profile_id {
            debug_struct.field("profile_id", &profile_id);
        } else {
            debug_struct.field("profile_id", &"None");
        }
        debug_struct.finish()
    }
}

impl AddOrEditProfileFormRequest {
    pub fn from_raw(raw: &AddOrEditProfileFormRequestRaw) -> Self {
        AddOrEditProfileFormRequest {
            name: raw.name.clone(),
            height: raw.height.parse::<i16>().unwrap(),
            weight: raw.weight.parse::<i16>().unwrap(),
            city: raw.city.clone(),
            phone_number: raw.phone_number.clone(),
            description: raw.description.clone(),
            profile_id: raw.profile_id,
            captcha_token: raw.captcha_token.clone(),
        }
    }
}
