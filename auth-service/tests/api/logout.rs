use auth_service::{
    domain::email::Email,
    utils::{auth::generate_auth_cookie, constants::JWT_COOKIE_NAME},
};
use reqwest::Url;

use crate::helpers::TestApp;

#[tokio::test]
async fn should_return_200_if_valid_jwt_cookie() {
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

    let logout_response = app.post_logout().await;

    assert_eq!(logout_response.status().as_u16(), 200);
}

#[tokio::test]
async fn should_return_400_if_logout_called_twice_in_a_row() {
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

    let logout_response = app.post_logout().await;
    assert_eq!(logout_response.status().as_u16(), 200);
    let second_logout_response = app.post_logout().await;
    assert_eq!(second_logout_response.status().as_u16(), 400);
}

#[tokio::test]
async fn should_return_400_if_jwt_cookie_missing() {
    let app = TestApp::new().await;

    let logout_response = app.post_logout().await;

    assert_eq!(logout_response.status().as_u16(), 400);
}

#[tokio::test]
async fn should_return_401_if_invalid_token() {
    let app = TestApp::new().await;

    // add invalid cookie
    app.cookie_jar.add_cookie_str(
        &format!(
            "{}=invalid; HttpOnly; SameSite=Lax; Secure; Path=/",
            JWT_COOKIE_NAME
        ),
        &Url::parse("http://127.0.0.1").expect("Failed to parse URL"),
    );

    let logout_response = app.post_logout().await;

    assert_eq!(logout_response.status().as_u16(), 401);
}
