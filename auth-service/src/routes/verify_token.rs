use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

use crate::{app_state::AppState, domain::AuthAPIError, utils::auth::validate_token};

use serde::Deserialize;

#[derive(Deserialize)]
pub struct VerifyTokenRequest {
    token: String,
}

pub async fn verify_token(
    State(state): State<AppState>,
    Json(request): Json<VerifyTokenRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    let token = request.token;

    match validate_token(&token).await {
        Ok(_) => Ok(()),
        Err(_) => Err(AuthAPIError::InvalidToken),
    }
}
