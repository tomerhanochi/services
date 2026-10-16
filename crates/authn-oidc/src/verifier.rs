use crate::prelude::{OidcError, OidcIdentity, SessionId, Sessions};
use authn::prelude::VerifyCredentials;
use std::marker::PhantomData;
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
