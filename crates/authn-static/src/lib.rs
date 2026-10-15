//! A configured username/password pair for emergency access.
//! The configured password remains a secret; this is not a password database.

use std::{convert::Infallible, fmt};

use authn::VerifyCredentials;
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

#[derive(Clone)]
pub enum BasicCredentials {
    Parsed { username: String, password: String },
    Malformed,
}

impl BasicCredentials {
    pub fn new(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self::Parsed {
            username: username.into(),
            password: password.into(),
        }
    }
}

impl fmt::Debug for BasicCredentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("BasicCredentials([REDACTED])")
    }
}

#[derive(Clone)]
pub struct StaticVerifier<I = String> {
    username: [u8; 32],
    password: [u8; 32],
    identity: I,
}

impl StaticVerifier<String> {
    pub fn new(username: impl Into<String>, password: impl AsRef<[u8]>) -> Self {
        let username = username.into();
        Self::with_identity(username.as_bytes(), password, username.clone())
    }
}

impl<I> StaticVerifier<I> {
    pub fn with_identity(
        username: impl AsRef<[u8]>,
        password: impl AsRef<[u8]>,
        identity: I,
    ) -> Self {
        Self {
            username: Sha256::digest(username.as_ref()).into(),
            password: Sha256::digest(password.as_ref()).into(),
            identity,
        }
    }
}

impl<I> fmt::Debug for StaticVerifier<I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("StaticVerifier([REDACTED])")
    }
}

impl<I: Clone + Sync> VerifyCredentials for StaticVerifier<I> {
    type Credentials = BasicCredentials;
    type Identity = I;
    type Error = Infallible;

    async fn verify(&self, credentials: &BasicCredentials) -> Result<Option<I>, Infallible> {
        let BasicCredentials::Parsed { username, password } = credentials else {
            return Ok(None);
        };
        let username: [u8; 32] = Sha256::digest(username.as_bytes()).into();
        let password: [u8; 32] = Sha256::digest(password.as_bytes()).into();
        let matches = self.username.ct_eq(&username) & self.password.ct_eq(&password);
        Ok(bool::from(matches).then(|| self.identity.clone()))
    }
}
