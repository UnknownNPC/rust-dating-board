use actix_web::{web, Responder};
use serde::{Deserialize, Serialize};

use crate::{
    db::DbProvider,
    web_api::{auth::AuthenticationGate, routes::constants::{RESTRICTED_AREA, HACK_DETECTED}},
};

pub async fn delete_user_profile(
    db_provider: web::Data<DbProvider>,
    auth_gate: AuthenticationGate,
    query: web::Query<DeleteProfileRequest>,
) -> impl Responder {
    println!(
        "[route#delete_user_profile] Inside the delete profile endpoint. User auth status {}",
        auth_gate.is_authorized
    );

    if !auth_gate.is_authorized {
        return web::Json(DeleteProfileResponse::new_with_error(RESTRICTED_AREA));
    }

    let taget_profile_id = query.id;
    let all_user_profiles = db_provider
        .all_user_profiles(auth_gate.user_id.unwrap())
        .await
        .unwrap();

    let target_profile_model_opt = all_user_profiles
        .iter()
        .find(|profile_model| profile_model.id == taget_profile_id)
        .map(|model| model.id);
    if target_profile_model_opt.is_some() {
        let target_profile_id = target_profile_model_opt.unwrap();
        db_provider.delete_profile(target_profile_id).await.unwrap();

        web::Json(DeleteProfileResponse::empty())
    } else {
        web::Json(DeleteProfileResponse::new_with_error(HACK_DETECTED))
    }
}

#[derive(Deserialize)]
pub struct DeleteProfileRequest {
    pub id: i64,
}

#[derive(Serialize)]
pub struct DeleteProfileResponse {
    error: Option<String>,
}

impl DeleteProfileResponse {
    pub fn new_with_error(err: &str) -> Self {
        DeleteProfileResponse {
            error: Some(err.to_string()),
        }
    }

    pub fn empty() -> Self {
        DeleteProfileResponse { error: None }
    }
}
