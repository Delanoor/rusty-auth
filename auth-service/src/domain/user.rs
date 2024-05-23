use super::AuthAPIError;
use serde::Deserialize;
use validator::{validate_email, validate_length};

#[derive(Debug, Deserialize, Eq, Hash, PartialEq, Clone)]
pub struct Email(pub String);

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        return &self.0;
    }
}

impl Email {
    pub fn parse(&self) -> Result<(), AuthAPIError> {
        match validate_email(&self.0) {
            true => Ok(()),
            false => Err(AuthAPIError::InvalidCredentials),
        }
    }
}

#[derive(Debug, Deserialize, Eq, Hash, PartialEq, Clone)]
pub struct Password(pub String);

impl AsRef<str> for Password {
    fn as_ref(&self) -> &str {
        return &self.0;
    }
}

impl Password {
    pub fn parse(&self) -> Result<(), AuthAPIError> {
        match validate_length(&self.0, Some(8), None, None) {
            true => Ok(()),
            false => Err(AuthAPIError::InvalidCredentials),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct User {
    pub email: Email,
    pub password: Password,
    pub requires_2fa: bool,
}

impl User {
    pub fn new(email: Email, password: Password, requires_2fa: bool) -> Self {
        User {
            email,
            password,
            requires_2fa,
        }
    }
}
