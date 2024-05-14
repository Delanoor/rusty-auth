use crate::helpers::TestApp;

#[tokio::test]
async fn root_returns_auth_ui() {
    let app = TestApp::new().await;

    let response = app.get_root().await;

    assert_eq!(response.status().as_u16(), 200);
    assert_eq!(response.headers().get("content-type").unwrap(), "text/html");
}
// TODO: Implement tests for all other routes (signup, login, logout, verify-2fa, and verify-token)
// For now, simply assert that each route returns a 200 HTTP status code.

#[tokio::test]
async fn signup_returns_status_code() {
    let app = TestApp::new().await;

    let response = app.get_signup().await;

    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn login_returns_status_code() {
    let app = TestApp::new().await;

    let response = app.get_login().await;

    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn logout_returns_status_code() {
    let app = TestApp::new().await;

    let response = app.get_logout().await;

    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn verify_2fa_returns_status_code() {
    let app = TestApp::new().await;

    let response = app.get_verify_2fa().await;

    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn verify_token_returns_status_code() {
    let app = TestApp::new().await;

    let response = app.get_verify_token().await;

    assert_eq!(response.status().as_u16(), 200);
}
