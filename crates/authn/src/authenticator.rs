use std::error::Error;

use crate::prelude::{
    AuthOutcome, Authenticate, Either, ExtractCredentials, Extracted, VerifyCredentials,
};

#[derive(Debug, Clone)]
pub struct Authenticator<E, V> {
    extractor: E,
    verifier: V,
}

impl<E: ExtractCredentials, V: VerifyCredentials<Credentials = E::Credentials>>
    Authenticator<E, V>
{
    pub fn new(extractor: E, verifier: V) -> Self {
        Self {
            extractor,
            verifier,
        }
    }
}

impl<E, V> Authenticate for Authenticator<E, V>
where
    E: ExtractCredentials + Sync,
    E::Context: Sync,
    E::Credentials: Send,
    E::Error: Error + Send + 'static,
    V: VerifyCredentials<Credentials = E::Credentials> + Sync,
    V::Error: Error + Send + 'static,
{
    type Context = E::Context;
    type Identity = V::Identity;
    type Error = Either<E::Error, V::Error>;

    #[hotpath::measure(impl_type = "Authenticator", future = true)]
    async fn authenticate(
        &self,
        ctx: &Self::Context,
    ) -> Result<AuthOutcome<Self::Identity>, Self::Error> {
        let creds = match self.extractor.extract(ctx).map_err(Either::Left)? {
            Extracted::Absent => return Ok(AuthOutcome::NotApplicable),
            Extracted::Malformed => return Ok(AuthOutcome::Rejected),
            Extracted::Present(creds) => creds,
        };
        let Some(identity) = self.verifier.verify(&creds).await.map_err(Either::Right)? else {
            return Ok(AuthOutcome::Rejected);
        };
        Ok(AuthOutcome::Authenticated(identity))
    }
}
