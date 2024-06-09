use std::{error::Error, future::IntoFuture, sync::Arc};
pub mod auth {
    tonic::include_proto!("auth");
}

use app_state::{AppState, TokenStoreType};
use auth::{auth_server::AuthServer, VerifyTokenRequest, VerifyTokenResponse};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use domain::error::AuthAPIError;

use serde::{Deserialize, Serialize};

use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::{transport::Server, Request};

use utils::{
    auth::validate_token,
    constants::env::{BASE_PATH, DROPLET_IP},
};

use tower_http::{cors::CorsLayer, services::ServeDir};

use tonic::Response as TonicResponse;

pub mod app_state;

pub use auth::auth_server::Auth;

pub mod domain;
pub mod services;
pub mod utils;

pub mod routes;
use routes::{login, logout, signup, verify_2fa};

pub struct Application {
    // server: Serve<Router, Router>,
    pub address: String,
}

pub struct MyAuthService {
    token_store: TokenStoreType,
}

#[tonic::async_trait]
impl Auth for MyAuthService {
    async fn verify_token(
        &self,
        request: Request<VerifyTokenRequest>,
    ) -> Result<TonicResponse<VerifyTokenResponse>, tonic::Status> {
        let req = request.into_inner();

        println!("grpc: {:?}", req);

        let token = req.token;
        match validate_token(self.token_store.clone(), &token).await {
            Ok(_) => {
                let response: VerifyTokenResponse = VerifyTokenResponse {
                    success: true,
                    message: "2FA verified successfully".to_string(),
                };
                Ok(TonicResponse::new(response))
            }
            Err(e) => Err(tonic::Status::from_error(e.into())),
        }
    }
}

impl Application {
    pub async fn build(
        app_state: AppState,
        address: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let token_store = app_state.token_store.clone();

        let droplet_ip = DROPLET_IP;
        let base_path = BASE_PATH;
        let allowed_origins = [
            "http://localhost:8000".parse()?,
            "http://localhost:3001".parse()?,
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
            // .route("/verify-token", post(verify_token))
            .with_state(app_state)
            .layer(cors);

        let listener: tokio::net::TcpListener = tokio::net::TcpListener::bind(address).await?;

        let address = listener.local_addr()?.to_string();

        let http_server = axum::serve(listener, router.into_make_service()).into_future();

        // Setting up and spawning gRPC server
        let grpc_listener = TcpListener::bind("0.0.0.0:50051").await?;
        let grpc_stream = TcpListenerStream::new(grpc_listener);

        let grpc_server = Server::builder()
            .add_service(AuthServer::new(MyAuthService {
                token_store: token_store.clone(),
            }))
            .serve_with_incoming(grpc_stream);

        // let server_future = async move {
        //     tokio::spawn(http_server);
        //     tokio::spawn(grpc_server);
        // };
        tokio::spawn(http_server);
        tokio::spawn(grpc_server);

        Ok(Application { address })
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        println!("listening on {}", &self.address);
        tokio::signal::ctrl_c().await?;
        // self.server_future.await;
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
