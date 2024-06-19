use color_eyre::eyre::{eyre, Result};
use secrecy::{ExposeSecret, Secret};

use validator::ValidateLength;

#[derive(Debug, Clone)]
pub struct Password(Secret<String>);

impl PartialEq for Password {
    fn eq(&self, other: &Self) -> bool {
        self.0.expose_secret() == other.0.expose_secret()
    }
}

impl AsRef<Secret<String>> for Password {
    fn as_ref(&self) -> &Secret<String> {
        return &self.0;
    }
}

fn validate_password(s: &Secret<String>) -> bool {
    s.expose_secret().len() >= 8
}

impl Password {
    pub fn parse(s: Secret<String>) -> Result<Password> {
        match ValidateLength::validate_length(&s.expose_secret(), Some(8), None, None) {
            true => Ok(Self(s)),
            false => Err(eyre!("Failed to parse string to a Password type")),
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::Password;

//     use fake::faker::internet::en::Password as FakePassword;
//     use fake::Fake;
//     use secrecy::Secret; // New!

//     #[test]
//     fn empty_string_is_rejected() {
//         let password = Secret::new("".to_string()); // Updated!
//         assert!(Password::parse(password).is_err());
//     }
//     #[test]
//     fn string_less_than_8_characters_is_rejected() {
//         let password = Secret::new("1234567".to_string()); // Updated!
//         assert!(Password::parse(password).is_err());
//     }

//     #[derive(Debug, Clone)]
//     struct ValidPasswordFixture(pub Secret<String>); // Updated!

//     impl quickcheck::Arbitrary for ValidPasswordFixture {
//         fn arbitrary<G: quickcheck::Gen>(g: &mut G) -> Self {
//             let password = FakePassword(8..30).fake_with_rng(g);
//             Self(Secret::new(password)) // Updated!
//         }
//     }
//     #[quickcheck_macros::quickcheck]
//     fn valid_passwords_are_parsed_successfully(valid_password: ValidPasswordFixture) -> bool {
//         Password::parse(valid_password.0).is_ok()
//     }
// }
