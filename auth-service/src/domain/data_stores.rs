use color_eyre::eyre::{eyre, Context, Report, Result};
use rand::Rng;
use secrecy::{ExposeSecret, Secret};
use thiserror::Error;

use super::{email::Email, password::Password, User};

#[async_trait::async_trait]
pub trait UserStore {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError>;
    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError>;
    async fn validate_user(&self, email: &Email, password: &Password)
        -> Result<(), UserStoreError>;
}

#[derive(Debug, Error)]
pub enum UserStoreError {
    #[error("User already exists")]
    UserAlreadyExists,
    #[error("User not found")]
    UserNotFound,
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Unexpected error")]
    UnexpectedError(#[source] Report),
}

impl PartialEq for UserStoreError {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Self::UserAlreadyExists, Self::UserAlreadyExists)
                | (Self::UserNotFound, Self::UserNotFound)
                | (Self::InvalidCredentials, Self::InvalidCredentials)
                | (Self::UnexpectedError(_), Self::UnexpectedError(_))
        )
    }
}

#[async_trait::async_trait]
pub trait BannedTokenStore {
    async fn store_token(&mut self, token: Secret<String>) -> Result<(), BannedTokenStoreError>;
    async fn contains_token(&self, token: Secret<String>) -> Result<bool, BannedTokenStoreError>;
}

#[derive(Debug, Error)]
pub enum BannedTokenStoreError {
    #[error("Unexpected error")]
    UnexpectedError(#[source] Report),
}

impl PartialEq for BannedTokenStoreError {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Self::UnexpectedError(_), Self::UnexpectedError(_))
        )
    }
}

#[async_trait::async_trait]
pub trait TwoFACodeStore {
    async fn add_code(
        &mut self,
        email: Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFACode,
    ) -> Result<(), TwoFACodeStoreError>;
    async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFACodeStoreError>;

    async fn get_code(
        &self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError>;
}

#[derive(Debug, Error)]
pub enum TwoFACodeStoreError {
    #[error("Login Attempt ID not found")]
    LoginAttemptIdNotFound,
    #[error("Unexpected error")]
    UnexpectedError(#[source] Report),
}

impl PartialEq for TwoFACodeStoreError {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Self::LoginAttemptIdNotFound, Self::LoginAttemptIdNotFound)
                | (Self::UnexpectedError(_), Self::UnexpectedError(_))
        )
    }
}

#[derive(Debug, Clone)]
pub struct LoginAttemptId(Secret<String>);

impl PartialEq for LoginAttemptId {
    fn eq(&self, other: &Self) -> bool {
        self.0.expose_secret() == other.0.expose_secret()
    }
}

impl LoginAttemptId {
    pub fn parse(id: Secret<String>) -> Result<Self> {
        match uuid::Uuid::parse_str(&id.expose_secret()).wrap_err("Invalid login attempt id") {
            Ok(id) => Ok(Self(Secret::new(id.to_string()))),
            Err(e) => Err(e.into()),
        }
    }
}

impl Default for LoginAttemptId {
    fn default() -> Self {
        let uuid = uuid::Uuid::new_v4();
        Self(Secret::new(uuid.to_string()))
    }
}

impl AsRef<Secret<String>> for LoginAttemptId {
    fn as_ref(&self) -> &Secret<String> {
        &self.0
    }
}

#[derive(Clone, Debug)]
pub struct TwoFACode(Secret<String>);

impl PartialEq for TwoFACode {
    fn eq(&self, other: &Self) -> bool {
        self.0.expose_secret() == other.0.expose_secret()
    }
}

impl TwoFACode {
    pub fn parse(code: Secret<String>) -> Result<Self> {
        let code_u32 = code
            .expose_secret()
            .parse::<u32>()
            .wrap_err("Invalid 2FA code")?;

        if (100_000..=999_999).contains(&code_u32) {
            Ok(Self(code))
        } else {
            Err(eyre!("Invalid 2FA code"))
        }
    }
}

impl Default for TwoFACode {
    fn default() -> Self {
        let mut rng = rand::thread_rng();
        let code: u32 = rng.gen_range(100_000..1_000_000);

        TwoFACode(Secret::new(code.to_string()))
    }
}

impl AsRef<Secret<String>> for TwoFACode {
    fn as_ref(&self) -> &Secret<String> {
        &self.0
    }
}
