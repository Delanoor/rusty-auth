use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

use serde::{Deserialize, Serialize};

use crate::{
    app_state::AppState,
    domain::{error::AuthAPIError, user::User},
};

#[derive(Deserialize)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
    #[serde(rename = "requires2FA")]
    pub requires_2fa: bool,
}

fn check_valid_email_password(email: &str, password: &str) -> bool {
    email.contains("@") && email.len() > 2 && password.len() >= 8
}

pub async fn signup(
    state: State<AppState>,
    Json(request): Json<SignupRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    let email = request.email;
    let password = request.password;

    let is_email_password_valid = check_valid_email_password(&email, &password);

    if !is_email_password_valid {
        return Err(AuthAPIError::InvalidCredentials);
    }

    let user = User::new(email, password, request.requires_2fa);

    let mut user_store = state.user_store.write().await;

    if user_store.get_user(&user.email).await.is_ok() {
        return Err(AuthAPIError::UserAlreadyExists);
    }

    let _ = user_store
        .add_user(user)
        .await
        .map_err(|_| return AuthAPIError::UnexpectedError);

    let response = Json(SignupResponse {
        message: "User created successfully!".to_string(),
    });

    Ok((StatusCode::CREATED, response))
}

#[derive(Serialize, Debug, Deserialize, PartialEq)]
pub struct SignupResponse {
    pub message: String,
}
