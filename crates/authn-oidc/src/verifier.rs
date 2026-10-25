use crate::prelude::{OidcError, OidcIdentityStore, OidcUsers, SessionId, Sessions};
use authn::prelude::{Either, VerifyCredentials};
use authn_identity::prelude::Identity;

#[derive(Debug, Clone)]
pub struct OidcVerifier<S = OidcUsers> {
    sessions: Sessions,
    store: S,
}
impl<S> OidcVerifier<S> {
    pub fn new(sessions: Sessions, store: S) -> Self {
        Self { sessions, store }
    }
    pub fn store(&self) -> &S {
        &self.store
    }
}
impl<S: OidcIdentityStore + Sync> VerifyCredentials for OidcVerifier<S> {
    type Credentials = SessionId;
    type Identity = Identity;
    type Error = Either<OidcError, S::Error>;

    #[hotpath::measure(impl_type = "OidcVerifier", future = true)]
    async fn verify(&self, id: &SessionId) -> Result<Option<Identity>, Self::Error> {
        let Some(claims) = self.sessions.lookup(id).map_err(Either::Left)? else {
            return Ok(None);
        };
        self.store.lookup(&claims).await.map_err(Either::Right)
    }
}
