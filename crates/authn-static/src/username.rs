use crate::credentials::InvalidBasicCredentials;
use std::{fmt, str::FromStr};

/// A nonempty static-access username that can be encoded in HTTP Basic.
#[derive(Clone, PartialEq, Eq)]
pub struct Username(String);
impl Username {
    pub fn new(value: impl Into<String>) -> Result<Self, InvalidBasicCredentials> {
        let value = value.into();
        if value.is_empty() || value.contains(':') || value.chars().any(char::is_control) {
            return Err(InvalidBasicCredentials);
        }
        Ok(Self(value))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl FromStr for Username {
    type Err = InvalidBasicCredentials;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}
impl fmt::Debug for Username {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Username([REDACTED])")
    }
}
