use std::error::Error;

use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::post;
use axum::{serve::Serve, Router};

use axum::{response::Html, routing::get};
use tower_http::services::ServeDir;

// This struct encapsulates our application-related logic.
pub struct Application {
    server: Serve<Router, Router>,
    pub address: String,
}

impl Application {
    pub async fn build(address: &str) -> Result<Self, Box<dyn Error>> {
        let router = Router::new()
        .nest_service("/", ServeDir::new("assets"))
        .route("/hello", get(hello_handler))
        .route("/signup", post(signup));

        let listener = tokio::net::TcpListener::bind(address).await?;
        let address = listener.local_addr()?.to_string();
        let server = axum::serve(listener, router);

        Ok(Application {address, server})
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        println!("listening on {}", &self.address);
        self.server.await
    }
}


// Example route handler.
// For now we will simply return a 200 (OK) status code.
pub async fn signup() -> impl IntoResponse {
    StatusCode::OK.into_response()
}

async fn hello_handler() -> Html<&'static str> {
    Html("<h1>Hello, Rusty World!</h1>")
}