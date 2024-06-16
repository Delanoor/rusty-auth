use auth_service::{
    domain::email::Email, routes::TwoFactorAuthResponse, utils::constants::JWT_COOKIE_NAME,
};

use crate::helpers::{get_random_email, TestApp};

#[tokio::test]
async fn should_return_200_if_valid_credentials_and_2fa_disabled() {
    let app = TestApp::new().await;

    let random_email = get_random_email();

    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "pass1234",
        "requires2FA": false
    });

    let response = app.post_signup(&signup_body).await;

    assert_eq!(response.status().as_u16(), 201);

    let login_body = serde_json::json!({
        "email": random_email,
        "password": "pass1234"
    });

    let response = app.post_login(&login_body).await;
    assert_eq!(response.status().as_u16(), 200);

    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");

    assert!(!auth_cookie.value().is_empty());
}

#[tokio::test]
async fn should_return_206_if_valid_credentials_and_2fa_enabled() {
    let app = TestApp::new().await;
    let random_email = Email::parse(get_random_email()).unwrap();

    let signup_body = serde_json::json!({
        "email": random_email.as_ref(),
        "password": "pass1234",
        "requires2FA": true
    });

    let signup_response = app.post_signup(&signup_body).await;
    assert_eq!(signup_response.status().as_u16(), 201);

    let login_body = serde_json::json!({
        "email":random_email.as_ref(),
        "password": "pass1234",

    });

    let login_response = app.post_login(&login_body).await;

    assert_eq!(login_response.status().as_u16(), 206);

    let json_body = login_response
        .json::<TwoFactorAuthResponse>()
        .await
        .expect("Could not deserialize response body to TwoFactorAuthResponse");

    let two_fa_code_store = app.two_fa_code_store.read().await;

    let get_response = two_fa_code_store.get_code(&random_email).await;
    assert!(get_response.is_ok());
    assert_eq!(get_response.unwrap().0.as_ref(), json_body.login_attempt_id);
}

#[tokio::test]
async fn should_return_400_if_invalid() {
    let app = TestApp::new().await;

    let body = serde_json::json!({
        "email": get_random_email(),
        "password": "1234"
    });

    let response = app.post_login(&body).await;

    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn should_return_401_if_incorrect_credentials() {
    let app = TestApp::new().await;
    let body_signup = serde_json::json!({
        "email": "test@email.com",
        "password": "pwds1234",
        "requires2FA": false,
    });

    let signup_response = app.post_signup(&body_signup).await;
    assert_eq!(signup_response.status().as_u16(), 201);

    let body = serde_json::json!({
        "email": "test@email.com",
        "password": "12341234"
    });
    let response = app.post_login(&body).await;
    assert_eq!(response.status().as_u16(), 401);
}

#[tokio::test]
async fn should_return_422_if_malformed_credentials() {
    let app = TestApp::new().await;

    let body = serde_json::json!({
    "email": get_random_email()
    });

    let response = app.post_login(&body).await;

    assert_eq!(response.status().as_u16(), 422);
}
