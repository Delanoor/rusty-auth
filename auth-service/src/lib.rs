use app_state::AppState;
use auth::{auth_server::AuthServer, VerifyTokenRequest, VerifyTokenResponse};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    serve::Serve,
    Json, Router,
};
use domain::error::AuthAPIError;
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tonic::{transport::Server, Request};

use std::{error::Error, sync::Arc};

use utils::constants::env::{BASE_PATH, DROPLET_IP};

use tower_http::{cors::CorsLayer, services::ServeDir};

use tonic::{Response as TonicResponse, Status};

pub struct Application {
    // axum_server: Serve<Router, Router>,
    pub address: String,
}

pub mod app_state;
pub mod domain;
pub mod services;
pub mod utils;

pub mod routes;
use routes::{login, logout, signup, verify_2fa, verify_token};

pub mod auth {
    tonic::include_proto!("auth");
}

pub use auth::auth_server::Auth;

use tokio_stream::wrappers::TcpListenerStream;

pub struct MyAuthService {}

#[tonic::async_trait]
impl Auth for MyAuthService {
    async fn verify_token(
        &self,
        request: Request<VerifyTokenRequest>,
    ) -> Result<TonicResponse<VerifyTokenResponse>, Status> {
        let req = request.into_inner();
        dbg!(req);

        let response = VerifyTokenResponse {
            success: true,
            // message: "2FA verified successfully".into(),
        };
        return Ok(TonicResponse::new(response));
    }
}

impl Application {
    pub async fn build(app_state: AppState, address: &str) -> Result<Self, Box<dyn Error>> {
        let droplet_ip = DROPLET_IP;
        let base_path = BASE_PATH;
        let allowed_origins = [
            "http://localhost:8000".parse()?,
            format!("https://{}:8000", droplet_ip).parse()?,
            format!("{}/app", base_path).parse()?,
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
            .layer(cors);

        // let listener = tokio::net::TcpListener::bind(address).await?;
        // let address = listener.local_addr()?.to_string();

        // let axum_server = axum::serve(listener, router);
        // let grpc_server = tonic::transport::Server::builder()
        //     .add_service(AuthServer::new(MyAuthService {}))
        //     .serve(address.parse()?)
        //     .await?;

        let http_listener = TcpListener::bind(address).await?;
        let grpc_listener = TcpListener::bind("127.0.0.1:50051").await?;

        let http_address = http_listener.local_addr()?.to_string();
        let grpc_address = grpc_listener.local_addr()?.to_string();

        tokio::spawn(async move {
            axum::serve(http_listener, router).await.unwrap();
        });

        let grpc_stream = TcpListenerStream::new(grpc_listener);

        tokio::spawn(async move {
            Server::builder()
                .add_service(AuthServer::new(MyAuthService {}))
                .serve_with_incoming(grpc_stream)
                .await
                .unwrap();
        });

        Ok(Application {
            address: http_address,
        })

        // Ok(Application {
        //     axum_server,
        //     address,
        // })
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        println!("listening on {}", &self.address);
        // self.axum_server.await
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

impl IntoResponse for AuthAPIError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthAPIError::UserAlreadyExists => (StatusCode::CONFLICT, "User already exists"),
            AuthAPIError::InvalidCredentials => (StatusCode::BAD_REQUEST, "Invalid credentials"),
            AuthAPIError::IncorrectCredentials => {
                (StatusCode::UNAUTHORIZED, "Incorrect credentials")
            }
            AuthAPIError::MissingToken => (StatusCode::BAD_REQUEST, "Missing token"),
            AuthAPIError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid token"),
            AuthAPIError::UnexpectedError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Unexpected error")
            }
        };

        let body = Json(ErrorResponse {
            error: error_message.to_string(),
        });

        (status, body).into_response()
    }
}
