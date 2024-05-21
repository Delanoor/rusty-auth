use std::collections::HashMap;

use crate::domain::user::User;

#[derive(Debug, PartialEq)]
pub enum UserStoreError {
    UserAlreadyExists,
    UserNotFound,
    InvalidCredentials,
    UnexpectedError,
}

pub struct HashmapUserStore {
    users: HashMap<String, User>,
}

impl Default for HashmapUserStore {
    fn default() -> Self {
        let mut user_store = Self {
            users: HashMap::new(),
        };

        user_store.users.insert(
            "admin@email.com".to_string(),
            User {
                email: "admin@email.com".to_string(),
                password: "123123".to_string(),
                requires_2fa: false,
            },
        );
        user_store
    }
}

impl HashmapUserStore {
    pub fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        match self.users.get(&user.email) {
            Some(_) => Err(UserStoreError::UserAlreadyExists),
            None => {
                self.users.insert(user.email.to_string(), user);
                Ok(())
            }
        }
    }

    pub fn get_user(&self, email: &str) -> Result<User, UserStoreError> {
        match self.users.get(email) {
            Some(user) => Ok(user.clone()),
            None => Err(UserStoreError::UserNotFound),
        }
    }

    pub fn validate_user(&self, email: &str, password: &str) -> Result<(), UserStoreError> {
        match self.users.get(email) {
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
            email: "test_user".to_string(),
            password: "123123".to_string(),
            requires_2fa: false,
        };
        assert_eq!(Ok(()), HashmapUserStore::add_user(&mut user_map, new_user))
    }

    #[tokio::test]
    async fn test_get_user() {
        let user_map = HashmapUserStore::default();

        let user_result = user_map.get_user("admin@email.com").unwrap();
        assert_eq!("admin@email.com".to_owned(), user_result.email);
    }

    #[tokio::test]
    async fn test_validate_user() {
        let user_map = HashmapUserStore::default();
        assert!(user_map.validate_user("admin@email.com", "123123").is_ok());
    }
}
