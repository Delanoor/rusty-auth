use crate::helpers::{get_random_email, TestApp};

#[tokio::test]
async fn should_return_422_if_malformed_credentials() {
    let app = TestApp::new().await;

    let body = serde_json::json!({
    "email": get_random_email()
    });

    let response = app.post_login(&body).await;

    assert_eq!(response.status().as_u16(), 422);
}

#[tokio::test]
async fn should_return_400_if_invalid() {
    let app = TestApp::new().await;

    let body = serde_json::json!({
        "email": get_random_email(),
        "password": ""
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
