use auth_service::{
    domain::Email, routes::TwoFactorAuthResponse, utils::constants::JWT_COOKIE_NAME,
};

use crate::helpers::{get_random_email, TestApp};

#[tokio::test]
async fn should_return_200_if_correct_code() {
    let mut app = TestApp::new().await;
    let random_email = get_random_email();
    let email = Email::parse(random_email.clone()).unwrap();

    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "pass1234",
        "requires2FA": true
    });
    let signup_response = app.post_signup(&signup_body).await;
    assert_eq!(signup_response.status().as_u16(), 201);

    let login_body = serde_json::json!({
        "email": random_email,
        "password": "pass1234"
    });

    let login_response = app.post_login(&login_body).await;
    assert_eq!(login_response.status().as_u16(), 206);

    let (attempt_id, code) = {
        let two_fa_code_store = app.two_fa_code_store.read().await;
        two_fa_code_store.get_code(&email).await.unwrap()
    };

    let verify_two_fa_body = serde_json::json!({
        "email": random_email,
        "loginAttemptId": attempt_id.as_ref(),
        "2FACode": code.as_ref()
    });

    let verify_two_fa_response = app.post_verify_2fa(&verify_two_fa_body).await;

    assert_eq!(verify_two_fa_response.status().as_u16(), 200);

    let cookie = verify_two_fa_response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");

    assert!(!cookie.value().is_empty());

    app.clean_up().await
}

#[tokio::test]
async fn should_return_400_if_invalid_input() {
    let mut app = TestApp::new().await;

    let inputs = [
        serde_json::json!({
            "email": "test",
            "loginAttemptId": "test",
            "2FACode": "test"
        }),
        // Missing loginAttemptId field
        serde_json::json!({
            "email": "test",
            "loginAttemptId": "123123",
            "2FACode": "123456",
        }),
        // Missing 2FACode field
        serde_json::json!({
            "email": "test",
            "loginAttemptId": "123123",
            "2FACode": "123456"
        }),
    ];

    for input in inputs.iter() {
        let response = app.post_verify_2fa(input).await;

        assert_eq!(response.status().as_u16(), 400);
    }

    app.clean_up().await
}

#[tokio::test]
async fn should_return_401_if_incorrect_credentials() {
    let mut app = TestApp::new().await;
    let random_email = get_random_email();

    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "pass1234",
        "requires2FA": true
    });
    let signup_response = app.post_signup(&signup_body).await;
    assert_eq!(signup_response.status().as_u16(), 201);

    let login_body = serde_json::json!({
        "email": random_email,
        "password": "pass1234"
    });

    let login_response = app.post_login(&login_body).await;
    assert_eq!(login_response.status().as_u16(), 206);

    let login_attempt_id = login_response
        .json::<TwoFactorAuthResponse>()
        .await
        .expect("Could not deserialize response body to TwoFactorAuthResponse")
        .login_attempt_id;

    let verify_two_fa_body = serde_json::json!({
        "email": random_email,
        "loginAttemptId": login_attempt_id,
        "2FACode": "123123"
    });

    let verify_two_fa_response = app.post_verify_2fa(&verify_two_fa_body).await;

    assert_eq!(verify_two_fa_response.status().as_u16(), 401);

    app.clean_up().await
}

#[tokio::test]
async fn should_return_401_if_same_code_twice() {
    let mut app = TestApp::new().await;
    let random_email = get_random_email();
    let email = Email::parse(random_email.clone()).unwrap();

    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "pass1234",
        "requires2FA": true
    });
    let signup_response = app.post_signup(&signup_body).await;
    assert_eq!(signup_response.status().as_u16(), 201);

    let login_body = serde_json::json!({
        "email": random_email,
        "password": "pass1234"
    });

    let login_response = app.post_login(&login_body).await;
    assert_eq!(login_response.status().as_u16(), 206);

    let (attempt_id, code) = {
        let two_fa_code_store = app.two_fa_code_store.read().await;
        two_fa_code_store.get_code(&email).await.unwrap()
    };

    let verify_two_fa_body = serde_json::json!({
        "email": random_email,
        "loginAttemptId": attempt_id.as_ref(),
        "2FACode": code.as_ref()
    });

    let verify_two_fa_response = app.post_verify_2fa(&verify_two_fa_body).await;

    assert_eq!(verify_two_fa_response.status().as_u16(), 200);

    let cookie = verify_two_fa_response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");

    assert!(!cookie.value().is_empty());

    let second_verify_two_fa_response = app.post_verify_2fa(&verify_two_fa_body).await;
    assert_eq!(second_verify_two_fa_response.status().as_u16(), 401);

    app.clean_up().await
}

#[tokio::test]
async fn should_return_401_if_old_code() {
    let mut app = TestApp::new().await;
    let random_email = get_random_email();

    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "pass1234",
        "requires2FA": true
    });
    let signup_response = app.post_signup(&signup_body).await;
    assert_eq!(signup_response.status().as_u16(), 201);

    let login_body = serde_json::json!({
        "email": random_email,
        "password": "pass1234"
    });

    let login_response = app.post_login(&login_body).await;
    assert_eq!(login_response.status().as_u16(), 206);

    let login_attempt_id = login_response
        .json::<TwoFactorAuthResponse>()
        .await
        .expect("Could not deserialize response body to TwoFactorAuthResponse")
        .login_attempt_id;

    let verify_two_fa_body = serde_json::json!({
        "email": random_email,
        "loginAttemptId": login_attempt_id,
        "2FACode": "123123"
    });

    app.post_login(&login_body).await;

    let verify_two_fa_response = app.post_verify_2fa(&verify_two_fa_body).await;

    assert_eq!(verify_two_fa_response.status().as_u16(), 401);

    app.clean_up().await
}

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let mut app = TestApp::new().await;

    let inputs = [
        serde_json::json!({
            "email": ""
        }),
        serde_json::json!({
            "email" : "test@email.com",
            "loginAttemptId": "123123"
        }),
        serde_json::json!({
            "email": "test@email.com",
            "loginAttemptId": "123123",
            "2FACode": true
        }),
        // // Completely missing body
        serde_json::json!({}),
    ];

    for input in inputs.iter() {
        let response = app.post_verify_2fa(input).await;
        assert_eq!(response.status().as_u16(), 422);
    }

    app.clean_up().await
}
