use crate::digest::TokenDigest;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use rand::{RngCore, rngs::OsRng};
use sha2::{Digest, Sha256};
use std::fmt;
/// An opaque bearer credential. Debug output never includes the secret.
#[derive(Clone)]
pub struct Token(Vec<u8>);

impl Token {
    /// Generates a bearer token with 256 bits of OS-provided randomness.
    pub fn generate() -> Result<Self, rand::Error> {
        let mut bytes = [0; 32];
        OsRng.try_fill_bytes(&mut bytes)?;
        Ok(Self(URL_SAFE_NO_PAD.encode(bytes).into_bytes()))
    }

    /// Parses a nonempty bearer credential using HTTP token68 syntax.
    pub fn new(secret: impl AsRef<[u8]>) -> Result<Self, InvalidToken> {
        let bytes = secret.as_ref();
        let body_end = bytes
            .iter()
            .position(|byte| *byte == b'=')
            .unwrap_or(bytes.len());
        if body_end == 0
            || !bytes[..body_end]
                .iter()
                .all(|byte| byte.is_ascii_alphanumeric() || b"-._~+/".contains(byte))
            || !bytes[body_end..].iter().all(|byte| *byte == b'=')
        {
            return Err(InvalidToken);
        }
        Ok(Self(bytes.to_vec()))
    }

    pub fn expose_secret(&self) -> &[u8] {
        &self.0
    }

    pub fn digest(&self) -> TokenDigest {
        TokenDigest(Sha256::digest(&self.0).into())
    }
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Token([REDACTED])")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidToken;

impl fmt::Display for InvalidToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid bearer token syntax")
    }
}
impl std::error::Error for InvalidToken {}

impl std::str::FromStr for Token {
    type Err = InvalidToken;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}
impl TryFrom<&[u8]> for Token {
    type Error = InvalidToken;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}
