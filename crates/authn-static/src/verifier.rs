use crate::{credentials::BasicCredentials, error::PasswordVerificationError, store::StaticStore};
use authn::prelude::{Either, VerifyCredentials};

#[derive(Debug, Clone)]
pub struct StaticVerifier<S> {
    store: S,
}
impl<S> StaticVerifier<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }
    pub fn store(&self) -> &S {
        &self.store
    }
}
impl<S: StaticStore + Sync> VerifyCredentials for StaticVerifier<S>
where
    S::Identity: Send,
{
    type Credentials = BasicCredentials;
    type Identity = S::Identity;
    type Error = Either<S::Error, PasswordVerificationError>;

    #[hotpath::measure(impl_type = "StaticVerifier", future = true)]
    async fn verify(
        &self,
        credentials: &BasicCredentials,
    ) -> Result<Option<Self::Identity>, Self::Error> {
        let Some(account) = self
            .store
            .lookup(credentials.username())
            .await
            .map_err(Either::Left)?
        else {
            return Ok(None);
        };
        let password = credentials.password().clone();
        let matches = tokio::task::spawn_blocking(move || account.password_hash.verify(&password))
            .await
            .map_err(|error| Either::Right(PasswordVerificationError::Worker(error)))?
            .map_err(|error| Either::Right(PasswordVerificationError::Hash(error)))?;
        Ok(matches.then_some(account.identity))
    }
}
