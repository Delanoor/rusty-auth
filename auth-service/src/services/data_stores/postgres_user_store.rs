use std::error::Error;

use argon2::{
    password_hash::SaltString, Algorithm, Argon2, Params, PasswordHash, PasswordHasher,
    PasswordVerifier, Version,
};

use serde::Deserialize;
use sqlx::PgPool;

use crate::domain::{
    data_stores::{UserStore, UserStoreError},
    password::Password,
    Email, User,
};

pub struct PostgresUserStore {
    pool: PgPool,
}

impl PostgresUserStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow, Deserialize)]
pub struct PostgresUser {
    email: String,
    password_hash: String,
    requires_2fa: bool,
}

#[async_trait::async_trait]
impl UserStore for PostgresUserStore {
    #[tracing::instrument(name = "Adding user to PostgreSQL", skip_all)]
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        let password_hash = compute_password_hash(user.password.as_ref())
            .await
            .map_err(|_| UserStoreError::InvalidCredentials)?;

        sqlx::query!(
            r#"
                INSERT INTO users (email, password_hash, requires_2fa)
                VALUES ($1, $2, $3)
            "#,
            user.email.as_ref(),
            password_hash,
            user.requires_2fa
        )
        .execute(&self.pool)
        .await
        .map_err(|_| UserStoreError::UnexpectedError)?;

        Ok(())
    }

    #[tracing::instrument(name = "Retrieving user from PostgreSQL", skip_all)]
    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError> {
        let query = sqlx::query_as!(
            PostgresUser,
            r#"
                select email, password_hash, requires_2fa
                from users
                where email = $1
            "#,
            email.as_ref()
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| UserStoreError::UserNotFound)?;

        let email = Email::parse(query.email).unwrap();
        let password = Password::parse(query.password_hash).unwrap();

        Ok(User {
            email,
            password,
            requires_2fa: query.requires_2fa,
        })
    }

    #[tracing::instrument(name = "Validating user credentials in PostgreSQL", skip_all)]
    async fn validate_user(
        &self,
        email: &Email,
        password: &Password,
    ) -> Result<(), UserStoreError> {
        let user = self.get_user(email).await?;

        verify_password_hash(user.password.as_ref(), password.as_ref())
            .await
            .map_err(|_| UserStoreError::InvalidCredentials)?;

        Ok(())
    }
}

#[tracing::instrument(name = "Verify password hash", skip_all)]
async fn verify_password_hash(
    expected_password_hash: &str,
    password_candidate: &str,
) -> Result<(), Box<dyn Error>> {
    // retrieves the current span from the tracing context.
    // The span represents the execution context for the compute_password_hash func
    let current_span: tracing::Span = tracing::Span::current();

    let expected_password_hash = expected_password_hash.to_string();
    let password_candidate = password_candidate.to_string();

    let res = tokio::task::spawn_blocking(move || {
        current_span.in_scope(|| {
            let expected_password_hash =
                PasswordHash::new(&expected_password_hash).map_err(|e| e)?;
            Argon2::default()
                .verify_password(password_candidate.as_bytes(), &expected_password_hash)
                .map_err(|e| e)
        })
    })
    .await??;

    Ok(res)
}

#[tracing::instrument(name = "Computing password hash", skip_all)]
async fn compute_password_hash(password: &str) -> Result<String, Box<dyn Error>> {
    let current_span: tracing::Span = tracing::Span::current();

    let password = password.to_owned();
    let res = tokio::task::spawn_blocking(move || {
        current_span.in_scope(|| {
            let salt: SaltString = SaltString::generate(&mut rand::thread_rng());
            let argon2 = Argon2::new(
                Algorithm::Argon2id,
                Version::V0x13,
                Params::new(15000, 2, 1, None)?,
            );

            match argon2.hash_password(password.as_bytes(), &salt) {
                Ok(pass) => Ok(pass.to_string()),
                Err(e) => Err(e),
            }
        })
    })
    .await??;

    Ok(res)
}
