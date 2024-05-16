use crate::helpers::TestApp;

// Tokio's test macro is use to run the test in an async environment
#[tokio::test]
async fn root_returns_auth_ui() {
  let app = TestApp::new().await;
  
  let res = app.get_root().await;
  
  assert_eq!(res.status().as_u16(), 200);
  assert_eq!(res.headers().get("content-type").unwrap(), "text/html");
}

#[tokio::test]
async fn signup_succeded() {
  let app = TestApp::new().await;
  let res = app.post_signup().await;

  assert_eq!(res.status().as_u16(), 200);
}

// TODO: Implement tests for all other routes (signup, login, logout, verify-2fa, and verify-token)
// For now, simply assert that each route returns a 200 HTTP status code.