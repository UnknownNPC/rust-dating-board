use core::str;

use actix_web::{web, HttpResponse, Responder};
use awc::http::header::LOCATION;
use chrono::Utc;
use futures::future::OptionFuture;
use log::{error, info};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    config::Config,
    db::{CommentModel, DbProvider, UserModel},
    web_api::{
        auth::AuthenticationGate,
        recaptcha::Recaptcha,
        routes::{
            common::{get_relative_photo_url, HeadContext, NavContext},
            constant::{HOME_DATE_FORMAT, MSG_COMMENT_ADDED_CODE, NO_PHOTO_URL},
            html_render::HtmlPage,
            validator::{ErrorContext, Validator},
        },
    },
};
use rust_i18n::t;

use super::{bot_detector_gate::BotDetector, error::HtmlError};

async fn resolve_view_profile_data_context(
    profile_id: &Uuid,
    message_code: &Option<String>,
    db_provider: &web::Data<DbProvider>,
    config: &web::Data<Config>,
    auth_gate: &AuthenticationGate,
    bot_detector: &BotDetector,
    skip_view_counter_increase: bool,
) -> Result<ViewProfilePageDataContext, HtmlError> {
    let profile_opt = db_provider.find_active_profile_by(&profile_id).await?;
    let profile = profile_opt.ok_or(HtmlError::NotFound)?;
    let profile_photos = db_provider.find_all_profile_photos_for(profile_id).await?;

    let photo_urls: Vec<String> = profile_photos
        .iter()
        .map(|profile_photo| get_relative_photo_url(profile_photo, &config.all_photos_folder_name))
        .collect();
    let photo_urls_or_placeholder = match photo_urls.is_empty() {
        true => vec![NO_PHOTO_URL.to_string()],
        false => photo_urls,
    };

    let is_user_profile_author = auth_gate
        .user_id
        .as_ref()
        .map(|auth_user_id| &profile.user_id == auth_user_id)
        .unwrap_or_default();

    //increase view counter
    // if not is_user_profile_author == regular page or search request
    if is_user_profile_author || bot_detector.is_bot || skip_view_counter_increase {
        info!(
            "Is user profile owner [{}] or bot [{}]. Do not increase view counter",
            is_user_profile_author, bot_detector.is_bot
        )
    } else {
        db_provider
            .increase_view_for_profiles(&vec![profile.id])
            .await?;
    }

    let all_profile_db_comments = db_provider.all_profile_comments(profile_id).await?;
    let comments = all_profile_db_comments
        .iter()
        .map(|db_comment| ProfileCommentResponse::from_db_comment_and_user(db_comment))
        .collect();

    let user_comment_opt_fut = auth_gate
        .user_id
        .as_ref()
        .map(|user_id| db_provider.find_comment_by_profile_user_ids(&profile_id, user_id));

    let user_comment_opt_fut_opt = OptionFuture::from(user_comment_opt_fut).await;
    let user_db_comment_opt = match user_comment_opt_fut_opt {
        Some(Ok(Some(inner_result))) => Ok(Some(inner_result)),
        Some(err) => err,
        _ => Ok(None),
    }?;
    let user_name = auth_gate.user_name.clone().unwrap_or_default();
    let user_comment = user_db_comment_opt.as_ref().map(|user_db_comment| {
        ProfileCommentResponse::from_db_comment(user_db_comment, &user_name)
    });

    Ok(ViewProfilePageDataContext {
        id: profile.id,
        name: profile.name,
        phone_num: profile.phone_number,
        height: profile.height as i64,
        weight: profile.weight as i64,
        city: profile.city,
        description: profile.description,
        photo_urls: photo_urls_or_placeholder,
        date_create: profile.created_at.format(HOME_DATE_FORMAT).to_string(),
        is_user_profile_author,
        view_count: profile.view_count,
        all_comments: comments,
        user_comment,
        message_code: message_code.clone(),
    })
}

async fn resolve_nav_context(
    db_provider: &web::Data<DbProvider>,
    auth_gate: &AuthenticationGate,
    config: &web::Data<Config>,
) -> Result<NavContext, HtmlError> {
    let name = auth_gate
        .user_name
        .as_ref()
        .map(|f| f.as_str())
        .unwrap_or_default();
    let cities_names = db_provider.find_city_names().await?;

    Ok(NavContext::new(
        name,
        "",
        &config.captcha_google_id,
        false,
        &Option::None,
        &cities_names,
        &config.oauth_google_client_id,
        &config.oauth_google_redirect_url,
    ))
}

async fn resolve_head_context(
    data_context: &ViewProfilePageDataContext,
    db_provider: &web::Data<DbProvider>,
    profile_id: &Uuid,
    config: &web::Data<Config>,
) -> Result<HeadContext, HtmlError> {
    let page_title = format!(
        "{} {} – 0{}",
        t!("view_profile_page_title"),
        &data_context.name,
        &data_context.phone_num
    );
    let profile_photos = db_provider.find_all_profile_photos_for(profile_id).await?;
    let page_description: String = data_context.description.clone().chars().take(100).collect();
    Ok(HeadContext::new(
        &page_title,
        &page_description,
        &config,
        &profile_photos.first().cloned(),
    ))
}

pub async fn add_comment(
    db_provider: web::Data<DbProvider>,
    auth_gate: AuthenticationGate,
    config: web::Data<Config>,
    form_raw: web::Form<AddCommentFormRequestRaw>,
    bot_detector: BotDetector,
) -> Result<impl Responder, HtmlError> {
    info!(
        "Add comment profile ID. User auth status: [{}]. User ID: [{}]. Is bot: [{}]",
        auth_gate.is_authorized,
        auth_gate.user_id.unwrap_or_default(),
        false
    );

    if !auth_gate.is_authorized {
        return Err(HtmlError::NotAuthorized);
    }

    let profile_id = Uuid::parse_str(&form_raw.0.profile_id)?;
    let user_id = &auth_gate.user_id.unwrap();
    // hack checks
    let user_has_left_comment = db_provider
        .find_comment_by_profile_user_ids(&profile_id, user_id)
        .await?;
    if user_has_left_comment.is_some() {
        error!(
            "Adding comment hack detected. User ID: [{}]",
            auth_gate.user_id.unwrap_or_default(),
        );
        return Err(HtmlError::BotDetection);
    }

    let form_validation = form_raw.0.validate();
    let form = if let Err(error_context) = form_validation {
        info!(
            "Add comment form includes errors: [{:?}]. Buidling contexts...",
            &error_context
        );
        let temporary_comment = ProfileCommentResponse {
            date_create: Utc::now().naive_utc().format(HOME_DATE_FORMAT).to_string(),
            text: form_raw.0.text,
            user_name: auth_gate.user_name.clone().unwrap_or_default(),
            id: Uuid::new_v4(),
            is_draft: true,
        };

        let mut data_context = resolve_view_profile_data_context(
            &profile_id,
            &None,
            &db_provider,
            &config,
            &auth_gate,
            &bot_detector,
            true,
        )
        .await?;
        data_context.user_comment = Some(temporary_comment);
        let head_context =
            resolve_head_context(&data_context, &db_provider, &profile_id, &config).await?;
        let nav_context = resolve_nav_context(&db_provider, &auth_gate, &config).await?;

        return Ok(HtmlPage::view_profile(
            &head_context,
            &nav_context,
            &data_context,
            &error_context,
        ));
    } else {
        form_validation.unwrap()
    };

    let captcha_score =
        Recaptcha::verify(&config.captcha_google_secret, &form.captcha_token).await?;
    if captcha_score < config.captcha_google_score {
        error!("Google captcha score is low [{}]", captcha_score);
        return Err(HtmlError::BotDetection);
    }

    let new_db_comment = db_provider
        .add_comment(&profile_id, user_id, &form.text)
        .await?;
    info!("New comment was added: [{:?}]", &new_db_comment);

    let redirect_to_view_page = format!(
        "/view_profile?id={}&message_code={}",
        &profile_id, MSG_COMMENT_ADDED_CODE
    );

    Ok(HttpResponse::Found()
        .append_header((LOCATION, redirect_to_view_page))
        .finish())
}

pub async fn view_profile_page(
    db_provider: web::Data<DbProvider>,
    auth_gate: AuthenticationGate,
    bot_detector: BotDetector,
    config: web::Data<Config>,
    query: web::Query<ViewProfileQuery>,
) -> Result<impl Responder, HtmlError> {
    info!(
        "View profile ID [{}]. User auth status: [{}]. User ID: [{}]. Is bot: [{}]",
        &query.id,
        auth_gate.is_authorized,
        auth_gate.user_id.unwrap_or_default(),
        bot_detector.is_bot
    );

    let nav_context = resolve_nav_context(&db_provider, &auth_gate, &config).await?;
    let data_context = resolve_view_profile_data_context(
        &query.id,
        &query.message_code,
        &db_provider,
        &config,
        &auth_gate,
        &bot_detector,
        false,
    )
    .await?;

    let head_context =
        resolve_head_context(&data_context, &db_provider, &query.id, &config).await?;

    Ok(HtmlPage::view_profile(
        &head_context,
        &nav_context,
        &data_context,
        &ErrorContext::empty(),
    ))
}

#[derive(Deserialize)]
pub struct ViewProfileRequest {
    pub id: i64,
}

pub struct ViewProfilePageDataContext {
    pub id: Uuid,
    pub name: String,
    pub phone_num: String,
    pub height: i64,
    pub weight: i64,
    pub city: String,
    pub description: String,
    pub photo_urls: Vec<String>,
    pub date_create: String,
    pub is_user_profile_author: bool,
    pub view_count: i64,
    pub all_comments: Vec<ProfileCommentResponse>,
    pub user_comment: Option<ProfileCommentResponse>,
    pub message_code: Option<String>,
}

pub struct ProfileCommentResponse {
    pub id: Uuid,
    pub user_name: String,
    pub date_create: String,
    pub text: String,
    pub is_draft: bool,
}

impl ProfileCommentResponse {
    pub fn from_db_comment_and_user(input: &(CommentModel, Option<UserModel>)) -> Self {
        ProfileCommentResponse {
            id: input.0.id,
            user_name: input
                .1
                .as_ref()
                .map(|user| user.name.clone())
                .unwrap_or_default(),
            date_create: input.0.created_at.format(HOME_DATE_FORMAT).to_string(),
            text: input.0.text.clone(),
            is_draft: false,
        }
    }

    pub fn from_db_comment(input: &CommentModel, user_name: &String) -> Self {
        ProfileCommentResponse {
            id: input.id,
            user_name: user_name.clone(),
            date_create: input.created_at.format(HOME_DATE_FORMAT).to_string(),
            text: input.text.clone(),
            is_draft: false,
        }
    }
}

#[derive(Deserialize)]
pub struct AddCommentFormRequestRaw {
    pub profile_id: String,
    pub text: String,
    pub captcha_token: String,
}

#[derive(Debug)]
pub struct AddCommentFormRequest {
    pub profile_id: String,
    pub text: String,
    pub captcha_token: String,
}

impl AddCommentFormRequest {
    pub fn from_raw(raw: &AddCommentFormRequestRaw) -> Self {
        AddCommentFormRequest {
            profile_id: raw.profile_id.clone(),
            captcha_token: raw.captcha_token.clone(),
            text: raw.text.clone(),
        }
    }
}

#[derive(Deserialize)]
pub struct ViewProfileQuery {
    pub id: Uuid,
    pub message_code: Option<String>,
}
