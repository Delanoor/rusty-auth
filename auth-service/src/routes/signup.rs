use crate::domain::{email::Email, password::Password};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

use color_eyre::eyre::Result;
use secrecy::Secret;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{
    app_state::AppState,
    domain::{error::AuthAPIError, user::User},
};

#[derive(Deserialize, Validate)]
pub struct SignupRequest {
    pub email: String,
    pub password: Secret<String>,
    #[serde(rename = "requires2FA")]
    pub requires_2fa: bool,
}

// fn check_valid_email_password(email: &Email, password: &Password) -> bool {
//     dbg!(password);
//     email.parse().is_ok() && password.parse().is_ok()
// }

#[tracing::instrument(name = "Signup", skip_all)]
pub async fn signup(
    State(state): State<AppState>,
    Json(request): Json<SignupRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    let email =
        Email::parse(Secret::new(request.email)).map_err(|_| AuthAPIError::InvalidCredentials)?;
    let password =
        Password::parse(request.password).map_err(|_| AuthAPIError::InvalidCredentials)?;
    let user = User::new(email, password, request.requires_2fa);

    let mut user_store = state.user_store.write().await;

    if user_store.get_user(&user.email).await.is_ok() {
        return Err(AuthAPIError::UserAlreadyExists);
    }

    user_store
        .add_user(user)
        .await
        .map_err(|e| return AuthAPIError::UnexpectedError(e.into()))?;

    let response = Json(SignupResponse {
        message: "User created successfully!".to_string(),
    });

    Ok((StatusCode::CREATED, response))
}

#[derive(Serialize, Debug, Deserialize, PartialEq)]
pub struct SignupResponse {
    pub message: String,
}
