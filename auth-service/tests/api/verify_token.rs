use crate::helpers::TestApp;

#[tokio::test]
async fn verify_token_returns_status_code() {
    let app = TestApp::new().await;

    let response = app.get_verify_token().await;

    assert_eq!(response.status().as_u16(), 200);
}