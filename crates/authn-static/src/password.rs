use crate::credentials::InvalidBasicCredentials;
use std::{fmt, str::FromStr};

/// A nonempty static-access password that can be encoded in HTTP Basic.
#[derive(Clone)]
pub struct Password(String);
impl Password {
    pub fn new(value: impl Into<String>) -> Result<Self, InvalidBasicCredentials> {
        let value = value.into();
        if value.is_empty() || value.chars().any(char::is_control) {
            return Err(InvalidBasicCredentials);
        }
        Ok(Self(value))
    }
    pub fn expose_secret(&self) -> &str {
        &self.0
    }
}
impl FromStr for Password {
    type Err = InvalidBasicCredentials;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}
impl fmt::Debug for Password {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Password([REDACTED])")
    }
}
