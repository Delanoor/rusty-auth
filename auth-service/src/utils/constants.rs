use dotenvy::dotenv;
use lazy_static::lazy_static;
use std::env as std_env;

// Define a lazily evaluated static
lazy_static! {
    pub static ref JWT_SECRET: String = set_token();
    pub static ref DATABASE_URL: String = get_db();
    pub static ref REDIS_HOST_NAME: String = set_redis_host();
    pub static ref REDIS_PASSWORD: String = set_redis_password();
    pub static ref REDIS_PORT: String = set_redis_port();
}

fn set_token() -> String {
    dotenv().ok(); // Load env variables
    let secret = std_env::var(env::JWT_SECRET_ENV_VAR).expect("JWT_SECRET must be set.");
    if secret.is_empty() {
        panic!("JWT_SECRET must not be empty.")
    }

    secret
}
fn get_db() -> String {
    dotenv().ok(); // Load env variables
    std_env::var(env::DATABASE_URL).expect("DATABASE_URL must be set.")
}

fn set_redis_host() -> String {
    dotenv().ok();
    std_env::var(env::REDIS_HOST_NAME_ENV_VAR).unwrap_or(DEFAULT_REDIS_HOST_NAME.to_owned())
}
fn set_redis_password() -> String {
    dotenv().ok();
    std_env::var(env::REDIS_PASSWORD_ENV_VAR).expect("REDIS PASSWORD required")
}
fn set_redis_port() -> String {
    dotenv().ok();
    std_env::var(env::REDIS_PORT_ENV_VAR).expect("REDIS PORT required")
}

pub mod env {
    pub const JWT_SECRET_ENV_VAR: &str = "JWT_SECRET";
    pub const DATABASE_URL: &str = "DATABASE_URL";
    pub const DROPLET_IP: &str = "DROPLET_IP";
    pub const REDIS_HOST_NAME_ENV_VAR: &str = "REDIS_HOST_NAME";
    pub const REDIS_PASSWORD_ENV_VAR: &str = "REDIS_PASSWORD";
    pub const REDIS_PORT_ENV_VAR: &str = "REDIS_PORT";
}

pub const JWT_COOKIE_NAME: &str = "jwt";
pub const DEFAULT_REDIS_HOST_NAME: &str = "127.0.0.1";

pub mod prod {
    pub const APP_ADDRESS: &str = "0.0.0.0:3000";
}

pub mod test {
    pub const APP_ADDRESS: &str = "127.0.0.1:0";
}
