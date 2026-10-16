use crate::{store::TokenStore, token::Token};
use authn::prelude::VerifyCredentials;
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
        self.store.lookup(&token.digest()).await
    }
}
