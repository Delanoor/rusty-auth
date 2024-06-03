use auth_service::utils::constants::JWT_COOKIE_NAME;
use reqwest::Url;

use crate::helpers::{get_random_email, TestApp};

#[tokio::test]
async fn should_return_200_if_valid_jwt_cookie() {
    let app = TestApp::new().await;

    let email = get_random_email();
    let signup_body = serde_json::json!({
        "email": email,
        "password": "pass1234",
        "requires2FA": false
    });

    let signup_response = app.post_signup(&signup_body).await;
    assert_eq!(signup_response.status().as_u16(), 201);

    let login_body = serde_json::json!({
        "email":email,
        "password": "pass1234",
    });

    let login_response = app.post_login(&login_body).await;
    assert_eq!(login_response.status().as_u16(), 200);

    let cookie = login_response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");
    assert!(!cookie.value().is_empty());

    let token = cookie.value();
    let logout_response = app.post_logout().await;
    assert_eq!(logout_response.status().as_u16(), 200);

    let cookie = logout_response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");
    assert!(cookie.value().is_empty());

    let token_store = app.token_store.read().await;
    let get_token_response = token_store.get_token(token).await;

    assert!(get_token_response.is_ok());
}

#[tokio::test]
async fn should_return_400_if_jwt_cookie_missing() {
    let app = TestApp::new().await;

    let logout_response = app.post_logout().await;

    assert_eq!(logout_response.status().as_u16(), 400);
}

#[tokio::test]
async fn should_return_400_if_logout_called_twice_in_a_row() {
    let app = TestApp::new().await;
    let random_email = get_random_email();

    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "pass1234",
        "requires2FA": false
    });

    let signup_response = app.post_signup(&signup_body).await;
    assert_eq!(signup_response.status().as_u16(), 201);

    let signin_body = serde_json::json!({
        "email": random_email,
        "password": "pass1234"
    });

    let signin_response = app.post_login(&signin_body).await;

    assert_eq!(signin_response.status().as_u16(), 200);

    let cookie = signin_response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");
    assert!(!cookie.value().is_empty());

    let logout_response = app.post_logout().await;
    assert_eq!(logout_response.status().as_u16(), 200);

    let cookie = logout_response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");
    assert!(cookie.value().is_empty());

    // let cookie = generate_auth_cookie(&email).unwrap();
    // app.cookie_jar.add_cookie_str(
    //     &format!(
    //         "{}={}; HttpOnly; SameSite=Lax; Secure; Path=/",
    //         JWT_COOKIE_NAME,
    //         cookie.value()
    //     ),
    //     &Url::parse(&app.address).expect("Failed to parse URL"),
    // );

    let second_logout_response = app.post_logout().await;
    assert_eq!(second_logout_response.status().as_u16(), 400);
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
