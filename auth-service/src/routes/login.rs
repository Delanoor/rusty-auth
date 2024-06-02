use std::borrow::BorrowMut;

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};

use crate::{
    app_state::{self, AppState},
    domain::{
        data_stores::{LoginAttemptId, TwoFACode},
        email::Email,
        password::Password,
        AuthAPIError,
    },
    utils::auth::generate_auth_cookie,
};

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<LoginRequest>,
) -> (CookieJar, Result<impl IntoResponse, AuthAPIError>) {
    let email = match Email::parse(request.email) {
        Ok(email) => email,
        Err(_) => return (jar, Err(AuthAPIError::IncorrectCredentials)),
    };

    let password = match Password::parse(request.password) {
        Ok(password) => password,
        Err(_) => return (jar, Err(AuthAPIError::InvalidCredentials)),
    };

    let user_store = &state.user_store.read().await;

    if user_store.validate_user(&email, &password).await.is_err() {
        return (jar, Err(AuthAPIError::IncorrectCredentials));
    }
    let user = match user_store.get_user(&email).await {
        Ok(user) => user,
        Err(_) => return (jar, Err(AuthAPIError::IncorrectCredentials)),
    };

    // let auth_cookie = match generate_auth_cookie(&user.email) {
    //     Ok(cookie) => cookie,
    //     Err(_) => return (jar, Err(AuthAPIError::UnexpectedError)),
    // };
    // // let updated_jar = jar.add(auth_cookie);

    match user.requires_2fa {
        true => handle_2fa(&user.email, &state, jar).await,
        false => handle_no_2fa(&user.email, jar).await,
    }
}

async fn handle_2fa(
    email: &Email,
    state: &AppState,
    jar: CookieJar,
) -> (
    CookieJar,
    Result<(StatusCode, Json<LoginResponse>), AuthAPIError>,
) {
    let login_attempt_id = LoginAttemptId::default();
    let two_fa_code = TwoFACode::default();

    let mut two_fa_code_store = state.two_fa_code_store.write().await;

    if two_fa_code_store
        .add_code(email.clone(), login_attempt_id.clone(), two_fa_code)
        .await
        .is_err()
    {
        return (jar, Err(AuthAPIError::UnexpectedError));
    };

    let response = Json(LoginResponse::TwoFactorAuth(TwoFactorAuthResponse {
        message: "2FA required".to_owned(),
        login_attempt_id: String::from(login_attempt_id.as_ref()),
    }));
    (jar, Ok((StatusCode::PARTIAL_CONTENT, response)))
}

async fn handle_no_2fa(
    email: &Email,
    jar: CookieJar,
) -> (
    CookieJar,
    Result<(StatusCode, Json<LoginResponse>), AuthAPIError>,
) {
    let auth_cookie = match generate_auth_cookie(email) {
        Ok(cookie) => cookie,
        Err(_) => return (jar, Err(AuthAPIError::UnexpectedError)),
    };
    let updated_jar = jar.add(auth_cookie);

    (
        updated_jar,
        Ok((StatusCode::OK, Json(LoginResponse::RegularAuth))),
    )
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum LoginResponse {
    RegularAuth,
    TwoFactorAuth(TwoFactorAuthResponse),
}

// if a user requires 2FA, this json body should be returned
#[derive(Debug, Serialize, Deserialize)]
pub struct TwoFactorAuthResponse {
    pub message: String,
    #[serde(rename = "loginAttemptId")]
    pub login_attempt_id: String,
}
