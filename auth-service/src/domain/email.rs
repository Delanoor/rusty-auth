use validator::validate_email;

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct Email(String);

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        return &self.0;
    }
}

impl Email {
    pub fn parse(s: String) -> Result<Email, String> {
        match validate_email(&s) {
            true => Ok(Self(s)),
            false => Err(format!("{} is not a valid email.", s)),
        }
    }
}
