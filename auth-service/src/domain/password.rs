use validator::validate_length;

#[derive(Debug, Clone, PartialEq)]
pub struct Password(pub String);

impl AsRef<str> for Password {
    fn as_ref(&self) -> &str {
        return &self.0;
    }
}

impl Password {
    pub fn parse(s: String) -> Result<Password, String> {
        match validate_length(&s, Some(8), None, None) {
            true => Ok(Self(s)),
            false => Err("Failed to parse string to a Password type".to_string()),
        }
    }
}
