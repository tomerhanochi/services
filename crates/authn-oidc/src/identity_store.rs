use crate::{config::HttpsIssuerUrl, session::OidcIdentity};
use authn_identity::prelude::{Identity, User, UserId};
use std::{collections::HashMap, convert::Infallible, error::Error, sync::Arc};

/// Maps verified provider claims to an application principal. Unknown users return `None`.
pub trait OidcIdentityStore {
    type Error: Error + Send + 'static;
    fn lookup(
        &self,
        claims: &OidcIdentity,
    ) -> impl Future<Output = Result<Option<Identity>, Self::Error>> + Send;
}
impl<S: OidcIdentityStore + Send + Sync> OidcIdentityStore for Arc<S> {
    type Error = S::Error;
    async fn lookup(&self, claims: &OidcIdentity) -> Result<Option<Identity>, Self::Error> {
        self.as_ref().lookup(claims).await
    }
}

/// Explicit issuer/subject mappings for applications without a user database.
#[derive(Debug, Clone, Default)]
pub struct OidcUsers(HashMap<(String, String), UserId>);
impl OidcUsers {
    pub fn insert(&mut self, issuer: &HttpsIssuerUrl, subject: impl Into<String>, user: UserId) {
        self.0
            .insert((issuer.as_str().to_owned(), subject.into()), user);
    }
}
impl OidcIdentityStore for OidcUsers {
    type Error = Infallible;
    async fn lookup(&self, claims: &OidcIdentity) -> Result<Option<Identity>, Self::Error> {
        Ok(self
            .0
            .get(&(claims.issuer.clone(), claims.subject.clone()))
            .cloned()
            .map(|id| User { id }.into()))
    }
}
