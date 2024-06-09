use askama::Template;
use auth::auth_client::AuthClient;
use auth::VerifyTokenRequest;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
    Json, Router,
};
use axum_extra::extract::CookieJar;
use serde::Serialize;
use std::env;
use tonic::Request;
use tower_http::services::ServeDir;

mod auth {
    tonic::include_proto!("auth");
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .nest_service("/assets", ServeDir::new("assets"))
        .route("/", get(root))
        .route("/protected", get(protected));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();

    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    login_link: String,
    logout_link: String,
}

async fn root() -> impl IntoResponse {
    let mut address = env::var("BASE_PATH").unwrap_or("localhost".to_owned());
    if address.is_empty() {
        address = "localhost".to_owned();
    }
    let login_link = format!("{}/auth", address);
    let logout_link = format!("{}/auth/logout", address);

    let template = IndexTemplate {
        login_link,
        logout_link,
    };
    Html(template.render().unwrap())
}

async fn protected(jar: CookieJar) -> impl IntoResponse {
    let jwt_cookie = match jar.get("jwt") {
        Some(cookie) => cookie,
        None => {
            return StatusCode::UNAUTHORIZED.into_response();
        }
    };

    let auth_hostname = env::var("AUTH_SERVICE_HOST_NAME").unwrap_or("auth-service".to_owned());
    let url = format!("http://{}:50051", auth_hostname);

    let mut client = AuthClient::connect(url).await.unwrap();
    let request = Request::new(VerifyTokenRequest {
        token: jwt_cookie.value().to_string(),
    });

    let response = match client.verify_token(request).await {
        Ok(response) => response.into_inner(),
        Err(_) => {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    if response.success {
        Json(ProtectedRouteResponse {
            img_url: "https://i.ibb.co/YP90j68/Light-Live-Bootcamp-Certificate.png".to_owned(),
        })
        .into_response()
    } else {
        StatusCode::UNAUTHORIZED.into_response()
    }
}

#[derive(Serialize)]
pub struct ProtectedRouteResponse {
    pub img_url: String,
}
