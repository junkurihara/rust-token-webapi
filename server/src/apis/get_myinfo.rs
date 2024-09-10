use super::{request::MyInfoRequest, response::MyInfoResponse};
use crate::{
  constants::DEFAULT_CLIENT_ID,
  entity::Entity,
  log::*,
  state::AppState,
  table::{UserSearchKey, UserTable},
};
use axum::{
  extract::State,
  http::StatusCode,
  response::{IntoResponse, Response},
  Json,
};
use serde_json::json;
use std::sync::Arc;

use libcommon::{
  token_fields::{Audiences, TryNewField},
  UserInfo,
};

#[derive(Debug)]
pub enum GetMyInfoError {
  InvalidPassword,
  Argon2Failure,
  UnauthorizedUser,
}
impl IntoResponse for GetMyInfoError {
  fn into_response(self) -> Response {
    let (status, error_message) = match self {
      GetMyInfoError::InvalidPassword => (StatusCode::UNAUTHORIZED, "Unauthorized"),
      GetMyInfoError::Argon2Failure => (StatusCode::INTERNAL_SERVER_ERROR, "Something failed in authentication"),
      GetMyInfoError::UnauthorizedUser => (StatusCode::UNAUTHORIZED, "Unauthorized"),
    };
    let body = Json(json!({
        "error": error_message,
    }));
    (status, body).into_response()
  }
}

pub async fn get_myinfo(
  State(state): State<Arc<AppState>>,
  Json(input): Json<MyInfoRequest>,
) -> Result<Json<MyInfoResponse>, GetMyInfoError> {
  // Getusername and password form
  let (username, password) = (input.auth.username, input.auth.password);

  // check user existence
  let Ok(user) = state.table.user.find_user(UserSearchKey::Username(&username)).await else {
    return Err(GetMyInfoError::UnauthorizedUser);
  };
  let Some(user) = user else {
    return Err(GetMyInfoError::UnauthorizedUser);
  };

  // verify password
  let Ok(password_verified) = password.verify(&user.encoded_hash) else {
    return Err(GetMyInfoError::Argon2Failure);
  };
  if !password_verified {
    return Err(GetMyInfoError::InvalidPassword);
  }

  debug!("{} is verified by password. Return my info", username.as_str());

  // get allowed client ids
  let allowed_apps = state
    .crypto
    .audiences
    .clone()
    .unwrap_or(Audiences::new(DEFAULT_CLIENT_ID).unwrap());

  Ok(Json(MyInfoResponse {
    info: UserInfo {
      username: username.as_str().to_string(),
      allowed_apps,
      issuer: state.crypto.issuer.clone(),
      subscriber_id: user.subscriber_id.clone(),
      is_admin: user.is_admin(),
    },
    message: "ok. my info.".to_string(),
  }))
}
