use crate::domain::{
    data_stores::{UserStore, UserStoreError},
    email::Email,
    password::Password,
    user::User,
};
use secrecy::Secret;
use std::collections::HashMap;

pub struct HashmapUserStore {
    users: HashMap<Email, User>,
}

impl Default for HashmapUserStore {
    fn default() -> Self {
        let mut user_store = Self {
            users: HashMap::new(),
        };

        user_store.users.insert(
            Email::parse(Secret::new("admin@email.com".to_owned())).unwrap(),
            User {
                email: Email::parse(Secret::new("admin@email.com".to_string())).unwrap(),
                password: Password::parse(Secret::new("12341234".to_owned())).unwrap(),
                requires_2fa: false,
            },
        );
        user_store
    }
}

#[async_trait::async_trait]
impl UserStore for HashmapUserStore {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        match self.users.get(&user.email) {
            Some(_) => Err(UserStoreError::UserAlreadyExists),
            None => {
                self.users.insert(user.email.clone(), user);
                Ok(())
            }
        }
    }

    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError> {
        match self.users.get(email) {
            Some(user) => Ok(user.clone()),
            None => Err(UserStoreError::UserNotFound),
        }
    }

    async fn validate_user(
        &self,
        email: &Email,
        password: &Password,
    ) -> Result<(), UserStoreError> {
        match self.users.get(email) {
            Some(user) => {
                if user.password.eq(password) {
                    Ok(())
                } else {
                    Err(UserStoreError::InvalidCredentials)
                }
            }
            None => Err(UserStoreError::UserNotFound),
        }
    }
}

#[cfg(test)]
mod tests {
    use secrecy::ExposeSecret;

    use super::*;

    #[tokio::test]
    async fn test_add_user() {
        let mut user_store = HashmapUserStore::default();
        let new_user = User {
            email: Email::parse(Secret::new("test@email.com".to_string())).unwrap(),
            password: Password::parse(Secret::new("12341234".to_string())).unwrap(),
            requires_2fa: false,
        };
        let result = user_store.add_user(new_user.clone()).await;
        assert!(result.is_ok());

        let second_result = user_store.add_user(new_user).await;
        assert_eq!(Err(UserStoreError::UserAlreadyExists), second_result)
    }

    #[tokio::test]
    async fn test_get_user() {
        let user_map = HashmapUserStore::default();

        let user_result = user_map
            .get_user(&Email::parse(Secret::new("admin@email.com".to_string())).unwrap())
            .await
            .unwrap();
        assert_eq!(
            "admin@email.com",
            user_result.email.as_ref().expose_secret()
        );
    }

    #[tokio::test]
    async fn test_validate_user() {
        let user_map = HashmapUserStore::default();
        assert!(user_map
            .validate_user(
                &Email::parse(Secret::new("admin@email.com".to_string())).unwrap(),
                &Password::parse(Secret::new("12341234".to_string())).unwrap()
            )
            .await
            .is_ok());
    }
}
