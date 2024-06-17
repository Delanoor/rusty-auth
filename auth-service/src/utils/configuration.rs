use dotenvy::dotenv;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Deserialize, Serialize, Clone)]
pub struct Settings {
    pub app_address: String,
    pub test_app_address: String,
    pub jwt_secret: String,
    pub postgres: PostgresSettings,
    pub redis: RedisSettings,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct PostgresSettings {
    pub database_url: String,
    pub password: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct RedisSettings {
    pub host_name: String,
    pub password: String,
    pub port: String,
}

impl Settings {
    pub fn new() -> Result<Self, config::ConfigError> {
        dotenv().ok();

        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "local".into());
        let (app_address, test_app_address, jwt_secret, postgres, redis) = match run_mode.as_str() {
            "production" => (
                env::var("APP_ADDRESS").unwrap(),
                env::var("TEST_APP_ADDRESS").unwrap(),
                env::var("JWT_SECRET").unwrap(),
                PostgresSettings {
                    database_url: env::var("POSTGRES_DATABASE_URL").unwrap(),
                    password: env::var("POSTGRES_PASSWORD").unwrap(),
                },
                RedisSettings {
                    host_name: env::var("REDIS_HOST_NAME").unwrap(),
                    password: env::var("REDIS_PASSWORD").unwrap(),
                    port: env::var("REDIS_PORT").unwrap(),
                },
            ),
            _ => (
                env::var("LOCAL_APP_ADDRESS").unwrap(),
                env::var("LOCAL_TEST_APP_ADDRESS").unwrap(),
                env::var("LOCAL_JWT_SECRET").unwrap(),
                PostgresSettings {
                    database_url: env::var("LOCAL_POSTGRES_DATABASE_URL").unwrap(),
                    password: env::var("LOCAL_POSTGRES_PASSWORD").unwrap(),
                },
                RedisSettings {
                    host_name: env::var("LOCAL_REDIS_HOST_NAME").unwrap(),
                    password: env::var("LOCAL_REDIS_PASSWORD").unwrap(),
                    port: env::var("LOCAL_REDIS_PORT").unwrap(),
                },
            ),
        };

        Ok(Settings {
            app_address,
            test_app_address,
            jwt_secret,
            postgres,
            redis,
        })
    }
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    Settings::new()
}
