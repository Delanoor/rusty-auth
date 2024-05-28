use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

use axum_extra::extract::CookieJar;
use serde::Deserialize;

use crate::{
    app_state::AppState,
    domain::{email::Email, password::Password, AuthAPIError},
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

    let auth_cookie = generate_auth_cookie(&email).unwrap();
    let updated_jar = jar.add(auth_cookie);

    let user_store = &state.user_store.read().await;

    let get_response = user_store.get_user(&email).await;
    if get_response.is_err() {
        return (updated_jar, Err(AuthAPIError::IncorrectCredentials));
    }

    let validate_response = user_store.validate_user(&email, &password).await;
    match validate_response {
        Ok(_) => (updated_jar, Ok(StatusCode::OK.into_response())),
        Err(_) => (updated_jar, Err(AuthAPIError::IncorrectCredentials)),
    }
}
