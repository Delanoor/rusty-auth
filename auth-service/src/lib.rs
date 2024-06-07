use app_state::AppState;
use auth::{auth_server::AuthServer, VerifyTwoFaRequest, VerifyTwoFaResponse};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    serve::Serve,
    Json, Router,
};
use domain::error::AuthAPIError;
use serde::{Deserialize, Serialize};
use tonic::{transport::Server, Request};

use std::{error::Error, future::IntoFuture, sync::Arc};

use utils::constants::env::{BASE_PATH, DROPLET_IP};

use tower_http::{cors::CorsLayer, services::ServeDir};

use tonic::{Response as TonicResponse, Status};

// pub struct Application {
//     server: Serve<Router, Router>,
//     pub address: String,
// }
pub struct Application {
    rest_address: String,
    grpc_address: String,
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

pub struct MyAuthService {}

#[tonic::async_trait]
impl Auth for MyAuthService {
    async fn verify_two_fa(
        &self,
        request: Request<VerifyTwoFaRequest>,
    ) -> Result<TonicResponse<VerifyTwoFaResponse>, Status> {
        let req = request.into_inner();
        // Implement your 2FA verification logic here using self.app_state

        // Example response
        let response = VerifyTwoFaResponse {
            success: true,
            message: "2FA verified successfully".into(),
        };
        return Ok(TonicResponse::new(response));
    }
}

impl Application {
    pub async fn build(
        app_state: AppState,
        rest_address: &str,
        grpc_address: &str,
    ) -> Result<Self, Box<dyn Error>> {
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

        // let router = Router::new()
        //     .nest_service("/", ServeDir::new("assets"))
        //     .route("/signup", post(signup))
        //     .route("/login", post(login))
        //     .route("/logout", post(logout))
        //     .route("/verify-2fa", post(verify_2fa))
        //     .route("/verify-token", post(verify_token))
        //     .with_state(app_state)
        //     .layer(cors);

        // let listener = tokio::net::TcpListener::bind(address).await?;
        // let address = listener.local_addr()?.to_string();

        // let axum_server = axum::serve(listener, router);
        // let grpc_server = tonic::transport::Server::builder()
        //     .add_service(AuthServer::new(MyAuthService {}))
        //     .serve(address.parse()?);
        //     // .await?;

        let rest_listener = tokio::net::TcpListener::bind(rest_address).await?;
        let grpc_listener = tokio::net::TcpListener::bind(grpc_address).await?;

        let rest_address = rest_listener.local_addr()?.to_string();
        let grpc_address = grpc_listener.local_addr()?.to_string();

        Ok(Application {
            rest_address,
            grpc_address,
        })

        // Ok(Application {
        //     server: axum_server,
        //     grpc_server,
        //     address,
        // })
    }

    // pub async fn run(self) -> Result<(), std::io::Error> {
    //     println!("listening on {}", &self.address);
    //     // self.server.await

    // }
    pub async fn run(self, app_state: AppState) -> Result<(), Box<dyn Error>> {
        let rest_listener = tokio::net::TcpListener::bind(&self.rest_address).await?;
        let grpc_listener = tokio::net::TcpListener::bind(&self.grpc_address).await?;

        let axum_server = async {
            let router = Router::new()
                .route("/signup", post(routes::signup))
                .route("/login", post(routes::login))
                .route("/logout", post(routes::logout))
                .route("/verify-2fa", post(routes::verify_2fa))
                .route("/verify-token", post(routes::verify_token))
                .with_state(app_state.clone())
                .layer(CorsLayer::new().allow_credentials(true).allow_origin([
                    "http://localhost:8000".parse().unwrap(),
                    format!("https://{}:8000", DROPLET_IP).parse().unwrap(),
                    format!("{}/app", BASE_PATH).parse().unwrap(),
                ]));

            axum::serve(rest_listener, router.into_make_service())
        };

        // let grpc_server = async {
        //     let mut incoming =
        //         tokio::net::TcpListener::from_std(grpc_listener.try_into().unwrap()).unwrap();
        //     Server::builder()
        //         .add_service(AuthServer::new(MyAuthService {}))
        //         .serve_with_incoming(incoming.map_ok(|conn| {
        //             let stream = tokio::net::TcpStream::from_std(conn.into_std().unwrap()).unwrap();
        //             Ok::<_, std::io::Error>(stream)
        //         }))
        //         .await
        // };   /// no idea

        tokio::select! {
            result = axum_server => result,
            result = grpc_server => result,
        }

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
