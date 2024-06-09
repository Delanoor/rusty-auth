use std::sync::Arc;

use auth_service::app_state::AppState;

use auth_service::services::data_stores::postgres_user_store::PostgresUserStore;
use auth_service::services::data_stores::{
    hashmap_two_fa_code_store::HashmapTwoFACodeStore,
    hashset_banned_token_store::HashsetBannedTokenStore, mock_email_client::MockEmailClient,
};

use auth_service::utils::constants::prod;
use auth_service::utils::constants::DATABASE_URL;
use auth_service::{get_postgres_pool, Application};
use sqlx::PgPool;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    let pg_pool = configure_postgresql().await;

    let user_store = Arc::new(RwLock::new(PostgresUserStore::new(pg_pool)));
    let token_store = Arc::new(RwLock::new(HashsetBannedTokenStore::default()));
    let two_fa_code_store = Arc::new(RwLock::new(HashmapTwoFACodeStore::default()));
    let email_client = Arc::new(RwLock::new(MockEmailClient));

    let app_state = AppState::new(user_store, token_store, two_fa_code_store, email_client);
    let app = Application::build(app_state, prod::APP_ADDRESS)
        .await
        .expect("Failed to build application");

    app.run().await.expect("Failed to run application");
}

async fn configure_postgresql() -> PgPool {
    let pg_pool = get_postgres_pool(&DATABASE_URL)
        .await
        .expect("Failed to create Postgres connection pool!");

    // run db migrations against our test database
    sqlx::migrate!()
        .run(&pg_pool)
        .await
        .expect("Failed to run migrations");

    pg_pool
}
