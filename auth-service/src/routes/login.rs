use axum::{extract::State, response::IntoResponse, Json};
use reqwest::StatusCode;
use serde::Deserialize;

use crate::{
    app_state::AppState,
    domain::{email::Email, password::Password, AuthAPIError},
};

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

pub async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    let email = Email::parse(request.email).map_err(|_| AuthAPIError::InvalidCredentials)?;
    let password =
        Password::parse(request.password).map_err(|_| AuthAPIError::InvalidCredentials)?;

    let user_store = &state.user_store.read().await;

    let get_response = user_store.get_user(&email).await;
    if get_response.is_err() {
        return Err(AuthAPIError::IncorrectCredentials);
    }

    let validate_response = user_store.validate_user(&email, &password).await;
    match validate_response {
        Ok(_) => Ok(StatusCode::OK.into_response()),
        Err(_) => Err(AuthAPIError::IncorrectCredentials),
    }
}
