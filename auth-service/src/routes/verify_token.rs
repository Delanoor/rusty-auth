use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use color_eyre::eyre::Result;
use secrecy::Secret;

use crate::{app_state::AppState, domain::AuthAPIError, utils::auth::validate_token};

use serde::Deserialize;

#[derive(Deserialize)]
pub struct VerifyTokenRequest {
    token: String,
}

#[tracing::instrument(name = "VerifyToken", skip_all)]
pub async fn verify_token(
    State(state): State<AppState>,
    Json(request): Json<VerifyTokenRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    let token = request.token;
    match validate_token(state.token_store.clone(), Secret::new(token)).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(_) => Err(AuthAPIError::InvalidToken),
    }
}
