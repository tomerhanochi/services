use crate::{password::Password, username::Username};
use std::fmt;

/// Parsed HTTP Basic credentials. Debug output never includes secrets.
#[derive(Clone)]
pub struct BasicCredentials {
    username: Username,
    password: Password,
}

impl BasicCredentials {
    pub fn new(username: Username, password: Password) -> Self {
        Self { username, password }
    }

    pub fn parse(decoded: &str) -> Result<Self, InvalidBasicCredentials> {
        let (username, password) = decoded.split_once(':').ok_or(InvalidBasicCredentials)?;
        Ok(Self::new(
            Username::new(username)?,
            Password::new(password)?,
        ))
    }

    pub fn username(&self) -> &Username {
        &self.username
    }
    pub fn password(&self) -> &Password {
        &self.password
    }
}

impl fmt::Debug for BasicCredentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("BasicCredentials([REDACTED])")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidBasicCredentials;
impl fmt::Display for InvalidBasicCredentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid HTTP Basic credentials")
    }
}
impl std::error::Error for InvalidBasicCredentials {}
