use auth_service::Application;
use reqwest::{Client, Response};

pub struct TestApp {
  pub address: String,
  pub http_client: Client
}

impl TestApp {
  pub async fn new() -> Self {
    let app = Application::build("127.0.0.1:0").await.expect("Failed to bild application");
    let address = format!("http://{}", app.address.clone());

    #[allow(clippy::let_underscore_future)]
    let _ = tokio::spawn(app.run());

    let http_client = Client::new();

    Self {
      address,
      http_client
    }
  }

  pub async fn get_root(&self) -> Response {
    self.http_client.get(format!("{}/", &self.address))
    .send()
    .await
    .expect("Could not receive response from /")
  }

  pub async fn post_signup(&self) -> Response {
    self.http_client.post(format!("{}/signup", &self.address))
    .send()
    .await
    .expect("Could not receive response from /signup")
  }
  // TODO: Implement helper functions for all other routes (signup, login, logout, verify-2fa, and verify-token)
}