use crate::helpers::TestApp;

#[tokio::test]
async fn logout_returns_status_code() {
    let app = TestApp::new().await;

    let response = app.get_logout().await;

    assert_eq!(response.status().as_u16(), 200);
}
