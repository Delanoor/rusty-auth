use config::{Config, File};

use lazy_static::lazy_static;
use secrecy::Secret;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Deserialize, Clone)]
pub struct Settings {
    pub app_address: String,
    pub test_app_address: String,
    pub jwt_secret: Secret<String>,
    pub jwt_cookie_name: Secret<String>,
    pub postgres: PostgresSettings,
    pub region: Option<String>,
    pub redis: RedisSettings,
}

#[derive(Deserialize, Clone)]
pub struct PostgresSettings {
    pub database_url: Secret<String>,
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
        let app_env: String = env::var("APP_ENV").unwrap_or_else(|_| "local".into());

        let env_file = match app_env.as_str() {
            "production" => "src/config/production.toml",
            _ => "src/config/development.toml",
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

pub const JWT_COOKIE_NAME: &str = "jwt";

lazy_static! {
    pub static ref CONFIG: Settings = get_configuration().expect("Failed to load configuration");
}

pub fn get_jwt_seret() -> Secret<String> {
    CONFIG.jwt_secret.clone()
}
