use app_state::AppState;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    serve::Serve,
    Json, Router,
};
use domain::error::AuthAPIError;
use redis::{Client, RedisResult};

use secrecy::{ExposeSecret, Secret};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::error::Error;
use utils::{
    constants::env::DROPLET_IP,
    tracing::{make_span_with_request_id, on_request, on_response},
};

use tower_http::{cors::CorsLayer, services::ServeDir, trace::TraceLayer};

pub struct Application {
    server: Serve<Router, Router>,
    pub address: String,
}

pub mod app_state;
pub mod domain;
pub mod services;
pub mod utils;

pub mod routes;
use routes::{login, logout, signup, verify_2fa, verify_token};

impl Application {
    pub async fn build(app_state: AppState, address: &str) -> Result<Self, Box<dyn Error>> {
        let droplet_ip = DROPLET_IP;

        let allowed_origins: [axum::http::HeaderValue; 4] = [
            "http://localhost:3000".parse()?,
            "http://localhost:8080".parse()?,
            "http://172.17.0.1".parse()?,
            format!("https://{}:8000", droplet_ip).parse()?,
        ];

        let cors = CorsLayer::new()
            .allow_credentials(true)
            .allow_origin(allowed_origins);

        let router = Router::new()
            .nest_service("/", ServeDir::new("assets"))
            .route("/signup", post(signup))
            .route("/login", post(login))
            .route("/logout", post(logout))
            .route("/verify-2fa", post(verify_2fa))
            .route("/verify-token", post(verify_token))
            .with_state(app_state)
            .layer(cors)
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(make_span_with_request_id)
                    .on_request(on_request)
                    .on_response(on_response),
            );

        let listener = tokio::net::TcpListener::bind(address).await?;
        let address = listener.local_addr()?.to_string();
        let server = axum::serve(listener, router);

        Ok(Application { server, address })
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        tracing::info!("listening on {}", &self.address);
        self.server.await
    }
}

#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

impl IntoResponse for AuthAPIError {
    fn into_response(self) -> Response {
        log_error_chain(&self);
        let (status, error_message) = match self {
            AuthAPIError::UserAlreadyExists => (StatusCode::CONFLICT, "User already exists"),
            AuthAPIError::InvalidCredentials => (StatusCode::BAD_REQUEST, "Invalid credentials"),
            AuthAPIError::IncorrectCredentials => {
                (StatusCode::UNAUTHORIZED, "Incorrect credentials")
            }
            AuthAPIError::MissingToken => (StatusCode::BAD_REQUEST, "Missing token"),
            AuthAPIError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid token"),
            AuthAPIError::UnexpectedError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Unexpected error")
            }
        };

        let body = Json(ErrorResponse {
            error: error_message.to_string(),
        });

        (status, body).into_response()
    }
}

fn log_error_chain(e: &(dyn Error + 'static)) {
    let separator =
        "\n-----------------------------------------------------------------------------------\n";

    let mut report = format!("{}{:?}", separator, e);
    let mut current = e.source();
    while let Some(cause) = current {
        let str = format!("Caused by:\n\n{:?}", cause);
        report = format!("{}\n{}", report, str);
        current = cause.source();
    }
    report = format!("{}\n{}", report, separator);
    tracing::error!("{}", report);
}

pub async fn get_postgres_pool(url: &Secret<String>) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(5)
        .connect(url.expose_secret())
        .await
}

pub fn get_redis_client(
    redis_hostname: String,
    redis_password: String,
    redis_port: String,
) -> RedisResult<Client> {
    let redis_url = if redis_password.is_empty() {
        format!("redis://{}:{}/", redis_hostname, redis_port)
    } else {
        format!(
            "rediss://default:{}@{}:{}",
            redis_password, redis_hostname, redis_port
        )
    };
    redis::Client::open(redis_url)
}
