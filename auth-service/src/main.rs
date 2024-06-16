use std::sync::Arc;

use auth_service::app_state::AppState;

use auth_service::services::data_stores::mock_email_client::MockEmailClient;
use auth_service::services::data_stores::postgres_user_store::PostgresUserStore;
use auth_service::services::data_stores::redis_banned_token_store::RedisBannedTokenStore;
use auth_service::services::data_stores::redis_two_fa_code_store::RedisTwoFACodeStore;

use auth_service::utils::constants::{
    prod, DATABASE_URL, REDIS_HOST_NAME, REDIS_PASSWORD, REDIS_PORT,
};

use auth_service::{get_postgres_pool, get_redis_client, Application};
use sqlx::PgPool;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    let pg_pool = configure_postgresql()
        .await
        .expect("Failed to configure PostgreSQL");
    let redis_config = configure_redis();
    let redis_code_config = configure_redis();

    let user_store = Arc::new(RwLock::new(PostgresUserStore::new(pg_pool)));
    let token_store = Arc::new(RwLock::new(RedisBannedTokenStore::new(Arc::new(
        RwLock::new(redis_config),
    ))));
    let two_fa_code_store = Arc::new(RwLock::new(RedisTwoFACodeStore::new(Arc::new(
        RwLock::new(redis_code_config),
    ))));
    let email_client = Arc::new(RwLock::new(MockEmailClient));

    let app_state: AppState =
        AppState::new(user_store, token_store, two_fa_code_store, email_client);
    let app: Application = Application::build(app_state, prod::APP_ADDRESS)
        .await
        .expect("Failed to build application");

    app.run().await.expect("Failed to run application");
}

async fn configure_postgresql() -> Result<PgPool, Box<dyn std::error::Error>> {
    let pg_pool = get_postgres_pool(&DATABASE_URL).await.map_err(|e| {
        eprintln!("Failed to create Postgres connection pool: {:?}", e);
        e
    })?;

    // run db migrations against our test database
    sqlx::migrate!()
        .run(&pg_pool)
        .await
        .expect("Failed to run migrations");

    Ok(pg_pool)
}

fn configure_redis() -> redis::Connection {
    get_redis_client(
        REDIS_HOST_NAME.to_string(),
        REDIS_PASSWORD.to_string(),
        REDIS_PORT.to_string(),
    )
    .expect("Failed to get Redis client")
    .get_connection()
    .expect("Failed to get Redis connection")
}
