// use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
// use axum_extra::extract::CookieJar;
// use serde::Deserialize;

// use crate::{
//     app_state::AppState,
//     domain::{AuthAPIError, Email},
//     utils::auth::generate_auth_cookie,
// };

// pub async fn verify_2fa(
//     State(state): State<AppState>,
//     jar: CookieJar,
//     Json(request): Json<Verify2FARequest>,
// ) -> (CookieJar, Result<impl IntoResponse, AuthAPIError>) {
//     let email = match Email::parse(request.email) {
//         Ok(email) => email,
//         Err(_) => {
//             return (jar, Err(AuthAPIError::InvalidCredentials));
//         }
//     };

//     let (login_attempt_id, two_fa_code) = {
//         let two_fa_code_store = state.two_fa_code_store.write().await;
//         match two_fa_code_store.get_code(&email).await {
//             Ok((id, code)) => (id, code),
//             Err(_) => {
//                 return (jar, Err(AuthAPIError::IncorrectCredentials));
//             }
//         }
//     };

//     // Validate the credentials
//     if request.login_attempt_id != login_attempt_id.as_ref()
//         || request.two_fa_code != two_fa_code.as_ref()
//     {
//         return (jar, Err(AuthAPIError::IncorrectCredentials));
//     }

//     let auth_cookie = match generate_auth_cookie(&email) {
//         Ok(cookie) => cookie,
//         Err(_) => {
//             return (jar, Err(AuthAPIError::UnexpectedError));
//         }
//     };

//     let updated_jar = jar.add(auth_cookie);

//     {
//         let mut code_store = state.two_fa_code_store.write().await;
//         if code_store.remove_code(&email).await.is_err() {
//             return (updated_jar, Err(AuthAPIError::UnexpectedError));
//         }
//     }
//     (updated_jar, Ok(StatusCode::OK.into_response()))
// }

// #[derive(Deserialize)]
// pub struct Verify2FARequest {
//     pub email: String,
//     #[serde(rename = "loginAttemptId")]
//     pub login_attempt_id: String,
//     #[serde(rename = "2FACode")]
//     pub two_fa_code: String,
// }
