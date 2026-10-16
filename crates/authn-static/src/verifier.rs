use crate::{credentials::BasicCredentials, password::Password, username::Username};
use authn::prelude::VerifyCredentials;
use sha2::{Digest, Sha256};
use std::{convert::Infallible, fmt};
use subtle::ConstantTimeEq;
#[derive(Clone)]
pub struct StaticVerifier<I = String> {
    username: [u8; 32],
    password: [u8; 32],
    identity: I,
}

impl StaticVerifier<String> {
    pub fn new(username: Username, password: Password) -> Self {
        let identity = username.as_str().to_owned();
        Self::with_identity(&username, &password, identity)
    }
}

impl<I> StaticVerifier<I> {
    pub fn with_identity(username: &Username, password: &Password, identity: I) -> Self {
        Self {
            username: Sha256::digest(username.as_str().as_bytes()).into(),
            password: Sha256::digest(password.expose_secret().as_bytes()).into(),
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
        let username: [u8; 32] = Sha256::digest(credentials.username().as_bytes()).into();
        let password: [u8; 32] = Sha256::digest(credentials.password().as_bytes()).into();
        let matches = self.username.ct_eq(&username) & self.password.ct_eq(&password);
        Ok(bool::from(matches).then(|| self.identity.clone()))
    }
}
