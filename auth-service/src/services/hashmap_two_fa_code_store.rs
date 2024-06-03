use std::collections::HashMap;

use crate::domain::{
    data_stores::{LoginAttemptId, TwoFACode, TwoFACodeStore, TwoFACodeStoreError},
    email::Email,
};

#[derive(Default)]
pub struct HashmapTwoFACodeStore {
    codes: HashMap<Email, (LoginAttemptId, TwoFACode)>,
}

#[async_trait::async_trait]
impl TwoFACodeStore for HashmapTwoFACodeStore {
    async fn add_code(
        &mut self,
        email: Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFACode,
    ) -> Result<(), TwoFACodeStoreError> {
        match self.codes.insert(email, (login_attempt_id, code)) {
            Some(_) => Err(TwoFACodeStoreError::UnexpectedError),
            None => Ok(()),
        }
    }

    async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFACodeStoreError> {
        match self.codes.get(email) {
            Some(_) => {
                let _ = self.codes.remove(email);
                return Ok(());
            }
            None => return Err(TwoFACodeStoreError::LoginAttemptIdNotFound),
        }
    }

    async fn get_code(
        &self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError> {
        match self.codes.get(email) {
            Some((login_attempt_id, two_fa_code)) => {
                Ok((login_attempt_id.clone(), two_fa_code.clone()))
            }
            None => Err(TwoFACodeStoreError::LoginAttemptIdNotFound),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    #[tokio::test]
    async fn tests_add_code() {
        let mut store = HashmapTwoFACodeStore::default();
        let email = Email::parse("test@email.com".to_string()).unwrap();
        let login_attempt_id = LoginAttemptId::default();
        let code = TwoFACode::default();

        let response = store.add_code(email.clone(), login_attempt_id, code).await;
        assert!(response.is_ok());
        assert!(store.get_code(&email).await.is_ok());
    }
}
