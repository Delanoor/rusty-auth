use config::Config;
use dotenvy::dotenv;
use serde::{Deserialize, Serialize};

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

        let run_mode = std::env::var("RUN_MODE").unwrap_or_else(|_| "local".into());
        let config_file = match run_mode.as_str() {
            "production" => "config.production.yaml",
            _ => "config.local.yaml",
        };
        let settings = Config::builder()
            .add_source(config::File::new(config_file, config::FileFormat::Yaml))
            .build()?;

        settings.try_deserialize::<Settings>()
    }
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    Settings::new()
}
