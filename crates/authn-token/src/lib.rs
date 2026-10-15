//! Opaque API tokens verified against an application-owned store.

use std::{error::Error, fmt, sync::Arc};

use authn::VerifyCredentials;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use rand::{RngCore, rngs::OsRng};
use sha2::{Digest, Sha256};

/// An opaque bearer credential. Debug output never includes the secret.
#[derive(Clone)]
pub struct Token(Vec<u8>);

impl Token {
    /// Generates a bearer token with 256 bits of OS-provided randomness.
    pub fn generate() -> Result<Self, rand::Error> {
        let mut bytes = [0; 32];
        OsRng.try_fill_bytes(&mut bytes)?;
        Ok(Self::new(URL_SAFE_NO_PAD.encode(bytes)))
    }

    pub fn new(secret: impl AsRef<[u8]>) -> Self {
        Self(secret.as_ref().to_vec())
    }

    pub fn expose_secret(&self) -> &[u8] {
        &self.0
    }

    pub fn digest(&self) -> TokenDigest {
        TokenDigest(Sha256::digest(&self.0).into())
    }

    /// Whether this is a nonempty HTTP bearer token (token68 syntax).
    pub fn is_valid(&self) -> bool {
        let body_end = self
            .0
            .iter()
            .position(|byte| *byte == b'=')
            .unwrap_or(self.0.len());
        body_end > 0
            && self.0[..body_end]
                .iter()
                .all(|byte| byte.is_ascii_alphanumeric() || b"-._~+/".contains(byte))
            && self.0[body_end..].iter().all(|byte| *byte == b'=')
    }
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Token([REDACTED])")
    }
}

/// SHA-256 of a high-entropy API token, suitable for an indexed store lookup.
/// Generate tokens using cryptographically random secrets, not passwords.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TokenDigest(pub [u8; 32]);

pub trait TokenStore {
    type Identity;
    type Error: Error + Send + 'static;

    /// Returns only active, unexpired, unrevoked tokens. Unknown tokens return `None`.
    fn lookup(
        &self,
        digest: &TokenDigest,
    ) -> impl Future<Output = Result<Option<Self::Identity>, Self::Error>> + Send;
}

impl<S: TokenStore + Send + Sync> TokenStore for Arc<S> {
    type Identity = S::Identity;
    type Error = S::Error;

    async fn lookup(&self, digest: &TokenDigest) -> Result<Option<Self::Identity>, Self::Error> {
        self.as_ref().lookup(digest).await
    }
}

#[derive(Debug, Clone)]
pub struct TokenVerifier<S> {
    store: S,
}

impl<S> TokenVerifier<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }

    pub fn store(&self) -> &S {
        &self.store
    }
}

impl<S: TokenStore + Sync> VerifyCredentials for TokenVerifier<S> {
    type Credentials = Token;
    type Identity = S::Identity;
    type Error = S::Error;

    async fn verify(&self, token: &Token) -> Result<Option<Self::Identity>, Self::Error> {
        if !token.is_valid() {
            return Ok(None);
        }
        self.store.lookup(&token.digest()).await
    }
}
