use crate::prelude::OidcError;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use sha2::{Digest, Sha256};
use std::fmt;
#[derive(Clone)]
pub struct SessionId(String);

impl SessionId {
    /// Parses the canonical base64url encoding of a 256-bit session secret.
    pub fn from_cookie(value: impl Into<String>) -> Result<Self, OidcError> {
        let value = value.into();
        let mut decoded = [0; 32];
        if value.len() != 43 || URL_SAFE_NO_PAD.decode_slice(&value, &mut decoded).ok() != Some(32)
        {
            return Err(OidcError::InvalidCallback);
        }
        Ok(Self(value))
    }

    pub fn expose_secret(&self) -> &str {
        &self.0
    }

    pub(crate) fn digest(&self) -> [u8; 32] {
        Sha256::digest(self.0.as_bytes()).into()
    }
}

impl fmt::Debug for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SessionId([REDACTED])")
    }
}

impl std::str::FromStr for SessionId {
    type Err = OidcError;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::from_cookie(value)
    }
}
