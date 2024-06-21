use std::sync::Arc;

use auth_service::app_state::AppState;

use auth_service::domain::Email;
use auth_service::services::aws_email_client::AWSEmailClient;
use auth_service::services::data_stores::postgres_user_store::PostgresUserStore;
use auth_service::services::data_stores::redis_banned_token_store::RedisBannedTokenStore;
use auth_service::services::data_stores::redis_two_fa_code_store::RedisTwoFACodeStore;

use auth_service::services::postmark_email_client::PostmarkEmailClient;
use auth_service::utils::configuration::{
    get_configuration, prod, PostgresSettings, RedisSettings,
};

use auth_service::utils::tracing::init_tracing;
use auth_service::{get_postgres_pool, get_redis_client, Application};

use aws_config::meta::region::RegionProviderChain;
use aws_config::Region;
use aws_sdk_sesv2::Client as AWSClient;
use reqwest::Client;
use secrecy::Secret;
use sqlx::PgPool;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    color_eyre::install().expect("Failed to install color_eyre");
    init_tracing().expect("Failed to initialize tracing");

    let configuration = get_configuration().expect("Failed to get configurations");

    let postgres_settings = &configuration.postgres;
    let redis_settings = &configuration.redis;
    let pg_pool: sqlx::Pool<sqlx::Postgres> = configure_postgresql(&postgres_settings)
        .await
        .expect("Failed to configure PostgreSQL");
    let redis_config = configure_redis(&redis_settings);
    let redis_code_config = configure_redis(&redis_settings);

    let user_store = Arc::new(RwLock::new(PostgresUserStore::new(pg_pool)));
    let token_store = Arc::new(RwLock::new(RedisBannedTokenStore::new(Arc::new(
        RwLock::new(redis_config),
    ))));
    let two_fa_code_store = Arc::new(RwLock::new(RedisTwoFACodeStore::new(Arc::new(
        RwLock::new(redis_code_config),
    ))));

    // let email_client = Arc::new(configure_postmark_email_client());
    let email_client = Arc::new(configure_aws_ses_client().await);

    let app_state: AppState = AppState::new(
        user_store,
        token_store,
        two_fa_code_store,
        email_client,
        configuration.clone(),
    );
    let app: Application = Application::build(app_state, &configuration.app_address)
        .await
        .expect("Failed to build application");

    app.run().await.expect("Failed to run application");
}

async fn configure_postgresql(
    settings: &PostgresSettings,
) -> Result<PgPool, Box<dyn std::error::Error>> {
    let pg_pool = get_postgres_pool(&settings.database_url)
        .await
        .map_err(|e| {
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

fn configure_redis(settings: &RedisSettings) -> redis::Connection {
    get_redis_client(
        settings.host_name.to_owned(),
        settings.password.to_owned(),
        settings.port.to_owned(),
    )
    .expect("Failed to get Redis client")
    .get_connection()
    .expect("Failed to get Redis connection")
}

fn configure_postmark_email_client() -> PostmarkEmailClient {
    let configuration = get_configuration().expect("Failed to get configurations.");

    let http_client = Client::builder()
        .timeout(prod::email_client::TIMEOUT)
        .build()
        .expect("Failed to build HTTP client");

    PostmarkEmailClient::new(
        prod::email_client::BASE_URL.to_owned(),
        Email::parse(Secret::new(prod::email_client::SENDER.to_owned())).unwrap(),
        configuration.postmark_auth_token.to_owned(),
        http_client,
    )
}

async fn configure_aws_ses_client() -> AWSEmailClient {
    // let config = aws_config::load_from_env().await;
    let configuration: auth_service::utils::configuration::Settings =
        get_configuration().expect("Failed to get configurations.");
    let region_provider = RegionProviderChain::first_try(configuration.region.map(Region::new))
        .or_default_provider()
        .or_else(Region::new("us-west-1"));
    let shared_config = aws_config::from_env().region(region_provider).load().await;
    let client = AWSClient::new(&shared_config);
    AWSEmailClient::new(client)
}
