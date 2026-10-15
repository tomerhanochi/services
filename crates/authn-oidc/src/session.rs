use std::{
    collections::HashMap,
    fmt,
    marker::PhantomData,
    sync::{Arc, Mutex},
    time::SystemTime,
};

use authn::VerifyCredentials;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use rand::{RngCore, rngs::OsRng};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::OidcError;

/// Stable identity is the pair `(issuer, subject)`, not the email address.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OidcIdentity {
    pub issuer: String,
    pub subject: String,
    pub email: Option<String>,
    pub email_verified: bool,
}

#[derive(Clone)]
pub struct SessionId(String);

impl SessionId {
    pub fn from_cookie(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn expose_secret(&self) -> &str {
        &self.0
    }

    fn digest(&self) -> [u8; 32] {
        Sha256::digest(self.0.as_bytes()).into()
    }
}

impl fmt::Debug for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SessionId([REDACTED])")
    }
}

struct Session {
    identity: OidcIdentity,
    expires_at: SystemTime,
}

/// Shared, bounded process-local sessions. A restart invalidates all sessions.
#[derive(Clone, Default)]
pub struct Sessions(Arc<Mutex<HashMap<[u8; 32], Session>>>);

impl Sessions {
    /// Trusted application code may issue sessions after establishing an OIDC identity.
    /// Request data must never be used as an identity without token verification.
    pub fn issue(
        &self,
        identity: OidcIdentity,
        expires_at: SystemTime,
    ) -> Result<SessionId, OidcError> {
        let now = SystemTime::now();
        if expires_at <= now {
            return Err(OidcError::InvalidCallback);
        }
        let mut bytes = [0; 32];
        OsRng
            .try_fill_bytes(&mut bytes)
            .map_err(|error| OidcError::Random(Box::new(error)))?;
        let id = SessionId(URL_SAFE_NO_PAD.encode(bytes));
        let mut sessions = self.0.lock().map_err(|_| OidcError::StoreUnavailable)?;
        sessions.retain(|_, session| session.expires_at > now);
        if sessions.len() >= 10_000 {
            return Err(OidcError::Capacity);
        }
        sessions.insert(
            id.digest(),
            Session {
                identity,
                expires_at,
            },
        );
        Ok(id)
    }

    pub fn lookup(&self, id: &SessionId) -> Result<Option<OidcIdentity>, OidcError> {
        let mut sessions = self.0.lock().map_err(|_| OidcError::StoreUnavailable)?;
        let digest = id.digest();
        let Some(session) = sessions.get(&digest) else {
            return Ok(None);
        };
        if session.expires_at <= SystemTime::now() {
            sessions.remove(&digest);
            return Ok(None);
        }
        Ok(Some(session.identity.clone()))
    }

    pub fn revoke(&self, id: &SessionId) -> Result<(), OidcError> {
        self.0
            .lock()
            .map_err(|_| OidcError::StoreUnavailable)?
            .remove(&id.digest());
        Ok(())
    }
}

impl fmt::Debug for Sessions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Sessions { .. }")
    }
}

#[derive(Debug, Clone)]
pub struct OidcVerifier<I = OidcIdentity> {
    sessions: Sessions,
    identity: PhantomData<fn() -> I>,
}

impl<I> OidcVerifier<I> {
    pub fn new(sessions: Sessions) -> Self {
        Self {
            sessions,
            identity: PhantomData,
        }
    }
}

impl<I: From<OidcIdentity>> VerifyCredentials for OidcVerifier<I> {
    type Credentials = SessionId;
    type Identity = I;
    type Error = OidcError;

    async fn verify(&self, id: &SessionId) -> Result<Option<I>, OidcError> {
        Ok(self.sessions.lookup(id)?.map(I::from))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn expired_sessions_are_rejected_and_removed() {
        let sessions = Sessions::default();
        let id = sessions
            .issue(
                OidcIdentity {
                    issuer: "https://issuer.example".into(),
                    subject: "subject".into(),
                    email: None,
                    email_verified: false,
                },
                SystemTime::now() + Duration::from_secs(60),
            )
            .unwrap();
        sessions
            .0
            .lock()
            .unwrap()
            .get_mut(&id.digest())
            .unwrap()
            .expires_at = SystemTime::UNIX_EPOCH;
        assert!(sessions.lookup(&id).unwrap().is_none());
        assert!(sessions.0.lock().unwrap().is_empty());
    }
}
