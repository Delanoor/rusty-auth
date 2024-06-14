use auth_service::{
    app_state::{AppState, EmailClientType, TokenStoreType, TwoFACodeStoreType},
    get_postgres_pool, get_redis_client,
    services::data_stores::{
        mock_email_client::MockEmailClient, postgres_user_store::PostgresUserStore,
        redis_banned_token_store::RedisBannedTokenStore,
        redis_two_fa_code_store::RedisTwoFACodeStore,
    },
    utils::constants::{test, DATABASE_URL, REDIS_HOST_NAME},
    Application,
};
use reqwest::cookie::Jar;
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    Connection, Executor, PgConnection, PgPool,
};
use tokio::sync::RwLock;

use std::{str::FromStr, sync::Arc};
use uuid::Uuid;

pub struct TestApp {
    pub address: String,
    pub cookie_jar: Arc<Jar>,
    pub token_store: TokenStoreType,
    pub two_fa_code_store: TwoFACodeStoreType,
    pub email_client: EmailClientType,
    pub http_client: reqwest::Client,
    pub db_name: String,
    pub app_state: Arc<RwLock<AppState>>,
}

impl TestApp {
    pub async fn new() -> Self {
        let pg_pool = configure_postgresql().await;

        let db_name = match pg_pool.connect_options().get_database() {
            Some(name) => name.to_owned(),
            None => {
                panic!("Failed to retrieve db name")
            }
        };

        let redis_config = configure_redis();
        let redis_two_fa_conig = configure_redis();

        let user_store = Arc::new(RwLock::new(PostgresUserStore::new(pg_pool)));
        let token_store = Arc::new(RwLock::new(RedisBannedTokenStore::new(Arc::new(
            RwLock::new(redis_config),
        ))));
        let two_fa_code_store = Arc::new(RwLock::new(RedisTwoFACodeStore::new(Arc::new(
            RwLock::new(redis_two_fa_conig),
        ))));
        let email_client = Arc::new(RwLock::new(MockEmailClient));

        let app_state = AppState::new(
            user_store,
            token_store.clone(),
            two_fa_code_store.clone(),
            email_client.clone(),
            false,
        );
        let app: Application = Application::build(app_state.clone(), test::APP_ADDRESS)
            .await
            .expect("Failed to build app");

        let address = format!("http://{}", app.address.clone());

        // separate async task to avoid blocking the main thread
        #[allow(clippy::let_underscore_future)]
        let _ = tokio::spawn(app.run());

        let cookie_jar = Arc::new(Jar::default());
        let http_client = reqwest::Client::builder()
            .cookie_provider(cookie_jar.clone())
            .build()
            .unwrap();

        TestApp {
            address,
            cookie_jar,
            http_client,
            token_store,
            two_fa_code_store,
            email_client,
            db_name,
            app_state: Arc::new(RwLock::new(app_state.clone())),
        }
    }

    pub async fn get_root(&self) -> reqwest::Response {
        self.http_client
            .get(&format!("{}/", &self.address))
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn post_logout(&self) -> reqwest::Response {
        self.http_client
            .post(&format!("{}/logout", &self.address))
            .send()
            .await
            .expect(" -> StringFailed to execute request")
    }

    pub async fn post_verify_token<Body: serde::Serialize>(
        &self,
        body: &Body,
    ) -> reqwest::Response {
        self.http_client
            .post(&format!("{}/verify-token", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_signup<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(&format!("{}/signup", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_login<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(&format!("{}/login", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_verify_2fa<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(&format!("{}/verify-2fa", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn clean_up(&self) {
        delete_database(&self.db_name).await;
        let mut app_state = self.app_state.write().await;
        app_state.clean_up_called = true;
    }
}

pub fn get_random_email() -> String {
    format!("{}@example.com", Uuid::new_v4())
}

async fn configure_postgresql() -> PgPool {
    let postgresql_conn_url = DATABASE_URL.to_owned();

    let db_name = Uuid::new_v4().to_string();

    configure_database(&postgresql_conn_url, &db_name).await;

    let postgresql_conn_url_with_db = format!("{}/{}", postgresql_conn_url, db_name);

    get_postgres_pool(&postgresql_conn_url_with_db)
        .await
        .expect("Failed to create Postgres connection pool!")
}

async fn configure_database(db_conn_string: &str, db_name: &str) {
    let connection = PgPoolOptions::new()
        .connect(db_conn_string)
        .await
        .expect("Failed to create Postgres connection pool.");

    // create a new db
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, db_name).as_str())
        .await
        .expect("Failed to create database.");

    // connect to new db
    let db_conn_string = format!("{}/{}", db_conn_string, db_name);

    let connection = PgPoolOptions::new()
        .connect(&db_conn_string)
        .await
        .expect("Failed to create Postgres connection pool.");

    // run migrations against new db
    sqlx::migrate!()
        .run(&connection)
        .await
        .expect("Failed to migrate the database.");
}

fn configure_redis() -> redis::Connection {
    get_redis_client(REDIS_HOST_NAME.to_string())
        .expect("Failed to get Redis client")
        .get_connection()
        .expect("Failed to get Redis connection")
}

async fn delete_database(db_name: &str) {
    let postgresql_conn_url = DATABASE_URL.to_owned();

    let connection_options = PgConnectOptions::from_str(&postgresql_conn_url)
        .expect("Failed to parse PostgreSQL connection string");

    let mut connection = PgConnection::connect_with(&connection_options)
        .await
        .expect("Failed to connect to Postgres");

    // Kill active connections to db
    connection
        .execute(
            format!(
                r#"
                SELECT pg_terminate_backend(pg_stat_activity.pid)
                FROM pg_stat_activity
                WHERE pg_stat_activity.datname = '{}'
                AND pid <> pg_backend_pid();
            "#,
                db_name
            )
            .as_str(),
        )
        .await
        .expect("Failed to drop the database.");

    // Drop the db
    connection
        .execute(format!(r#"DROP DATABASE "{}";"#, db_name).as_str())
        .await
        .expect("Failed to drop the database.");
}
