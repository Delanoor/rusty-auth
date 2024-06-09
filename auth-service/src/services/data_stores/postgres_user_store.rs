use std::error::Error;

use argon2::{
    password_hash::{Salt, SaltString},
    Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version,
};

use sqlx::PgPool;
use tokio::task;

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

#[async_trait::async_trait]
impl UserStore for PostgresUserStore {
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
    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError> {
        let query = sqlx::query!(
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
    async fn validate_user(
        &self,
        email: &Email,
        password: &Password,
    ) -> Result<(), UserStoreError> {
        let user = self.get_user(email).await?;

        verify_password_hash(user.password.as_ref(), password.as_ref())
            .map_err(|_| UserStoreError::InvalidCredentials)?;

        Ok(())
    }
}

fn verify_password_hash(
    expected_password_hash: &str,
    password_candidate: &str,
) -> Result<(), Box<dyn Error>> {
    let expected_password_hash: PasswordHash<'_> = PasswordHash::new(expected_password_hash)?;
    Argon2::default()
        .verify_password(password_candidate.as_bytes(), &expected_password_hash)
        .map_err(|e| e.into())
}

async fn compute_password_hash(password: &str) -> Result<String, Box<dyn Error>> {
    let password = password.to_string();
    let res = tokio::task::spawn_blocking(move || {
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
    .await??;

    Ok(res)
}
