use std::collections::HashMap;

use crate::domain::{
    data_stores::{UserStore, UserStoreError},
    user::{Email, Password, User},
};

pub struct HashmapUserStore {
    users: HashMap<Email, User>,
}

impl Default for HashmapUserStore {
    fn default() -> Self {
        let mut user_store = Self {
            users: HashMap::new(),
        };

        user_store.users.insert(
            Email("admin@email.com".to_string()),
            User {
                email: Email("admin@email.com".to_string()),
                password: Password("123123".to_string()),
                requires_2fa: false,
            },
        );
        user_store
    }
}

#[async_trait::async_trait]
impl UserStore for HashmapUserStore {
    async fn add_user(&mut self, user: &User) -> Result<(), UserStoreError> {
        match self.users.get(&user.email) {
            Some(_) => Err(UserStoreError::UserAlreadyExists),
            None => {
                self.users.insert(user.email.clone(), user.clone());
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

    async fn validate_user(&self, email: Email, password: Password) -> Result<(), UserStoreError> {
        match self.users.get(&email) {
            Some(user) => {
                if user.password != password {
                    Err(UserStoreError::InvalidCredentials)
                } else {
                    Ok(())
                }
            }
            None => Err(UserStoreError::UserNotFound),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_user() {
        let mut user_map = HashmapUserStore::default();
        let new_user = User {
            email: Email("test_user".to_string()),
            password: Password("123123".to_string()),
            requires_2fa: false,
        };
        assert_eq!(
            Ok(()),
            HashmapUserStore::add_user(&mut user_map, &new_user).await
        )
    }

    #[tokio::test]
    async fn test_get_user() {
        let user_map = HashmapUserStore::default();

        let user_result = user_map
            .get_user(&Email("admin@email.com".to_string()))
            .await
            .unwrap();
        assert_eq!("admin@email.com", user_result.email.as_ref());
    }

    #[tokio::test]
    async fn test_validate_user() {
        let user_map = HashmapUserStore::default();
        assert!(user_map
            .validate_user(
                Email("admin@email.com".to_string()),
                Password("123123".to_string())
            )
            .await
            .is_ok());
    }
}
