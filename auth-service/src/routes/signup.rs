use crate::domain::user::{Email, Password};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{
    app_state::AppState,
    domain::{error::AuthAPIError, user::User},
};

#[derive(Deserialize, Validate)]
pub struct SignupRequest {
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
    #[serde(rename = "requires2FA")]
    pub requires_2fa: bool,
}

fn check_valid_email_password(email: &Email, password: &Password) -> bool {
    dbg!(password);
    email.parse().is_ok() && password.parse().is_ok()
}

pub async fn signup(
    state: State<AppState>,
    Json(request): Json<SignupRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    let email = Email(request.email);
    let password = Password(request.password);

    let is_email_password_valid = check_valid_email_password(&email, &password);

    if !is_email_password_valid {
        return Err(AuthAPIError::InvalidCredentials);
    }

    let mut user_store = state.user_store.write().await;

    if user_store.get_user(&email).await.is_ok() {
        return Err(AuthAPIError::UserAlreadyExists);
    } else {
        let user = User::new(email, password, request.requires_2fa);
        let _ = user_store
            .add_user(&user)
            .await
            .map_err(|_| return AuthAPIError::UnexpectedError);

        let response = Json(SignupResponse {
            message: "User created successfully!".to_string(),
        });

        Ok((StatusCode::CREATED, response))
    }
}

#[derive(Serialize, Debug, Deserialize, PartialEq)]
pub struct SignupResponse {
    pub message: String,
}
