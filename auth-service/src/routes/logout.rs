use axum::{extract::State, http::StatusCode, response::IntoResponse};
use axum_extra::extract::{cookie::Cookie, CookieJar};

use crate::{
    app_state::AppState,
    domain::AuthAPIError,
    utils::{auth::validate_token, constants::JWT_COOKIE_NAME},
};

pub async fn logout(
    State(state): State<AppState>,
    jar: CookieJar,
) -> (CookieJar, Result<impl IntoResponse, AuthAPIError>) {
    // Retrieve JWT cookie from the `CookieJar`
    // Return AuthAPIError::MissingToken is the cookie is not found
    let cookie = match jar.get(JWT_COOKIE_NAME) {
        Some(c) => c,
        None => return (jar, Err(AuthAPIError::MissingToken)),
    };

    let token = cookie.value().to_owned();

    let validate_response = validate_token(state.token_store.clone(), &token).await;

    if validate_response.is_err() {
        return (jar, Err(AuthAPIError::InvalidToken));
    }
    let mut token_store = state.token_store.write().await;

    let jar = jar.remove(Cookie::from(JWT_COOKIE_NAME));
    match token_store.store_token(token).await {
        Ok(_) => (jar, Ok(StatusCode::OK)),
        Err(_) => (jar, Err(AuthAPIError::UnexpectedError)),
    }
}
