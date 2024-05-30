use auth_service::{
    domain::email::Email,
    utils::{auth::generate_auth_cookie, constants::JWT_COOKIE_NAME},
};
use reqwest::Url;

use crate::helpers::TestApp;

#[tokio::test]
async fn should_return_200_valid_token() {
    let app = TestApp::new().await;
    let email = Email::parse("test@email.com".to_string()).unwrap();
    let cookie = generate_auth_cookie(&email).unwrap();

    app.cookie_jar.add_cookie_str(
        &format!(
            "{}={}; HttpOnly; SameSite=Lax; Secure; Path=/",
            JWT_COOKIE_NAME,
            cookie.value()
        ),
        &Url::parse(&app.address).expect("Failed to parse URL"),
    );

    let body = serde_json::json!({
        "token" : cookie.value()
    });
    let response = app.post_verify_token(&body).await;

    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn should_return_401_if_invalid_token() {
    let app = TestApp::new().await;

    let body = serde_json::json!({
        "token": "invalid",
    });
    let response = app.post_verify_token(&body).await;
    assert_eq!(response.status().as_u16(), 401);
}

#[tokio::test]
async fn should_return_401_if_banned_token() {
    let app = TestApp::new().await;
    let email = Email::parse("test@email.com".to_string()).unwrap();
    let cookie = generate_auth_cookie(&email).unwrap();

    app.cookie_jar.add_cookie_str(
        &format!(
            "{}={}; HttpOnly; SameSite=Lax; Secure; Path=/",
            JWT_COOKIE_NAME,
            cookie.value()
        ),
        &Url::parse(&app.address).expect("Failed to parse URL"),
    );

    let body = serde_json::json!({
        "token" : cookie.value()
    });

    {
        let mut token_store = app.token_store.write().await;
        let _ = token_store.store_token(cookie.value().to_string()).await;
    }
    let verify_response = app.post_verify_token(&body).await;
    assert_eq!(verify_response.status().as_u16(), 401);
}

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let app = TestApp::new().await;

    let inputs = [
        serde_json::json!({
            "email": "test@email.com",

        }),
        serde_json::json!({}),
    ];

    for input in inputs.iter() {
        let response = app.post_verify_token(&input).await;

        assert_eq!(response.status().as_u16(), 422);
    }
}
