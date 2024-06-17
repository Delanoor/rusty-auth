use config::{Config, File, FileFormat};

use dotenvy::{dotenv, from_filename};
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
        let app_env = env::var("APP_ENV").unwrap_or_else(|_| "local".into());
        let env_file = match app_env.as_str() {
            "production" => "config.production.yaml",
            _ => "config.local.yaml",
        };

        let config = Config::builder()
            .add_source(File::with_name(env_file))
            .build()?;

        config.try_deserialize::<Settings>()
    }
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    Settings::new()
}
