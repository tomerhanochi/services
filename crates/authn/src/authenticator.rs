use crate::{AuthOutcome, Authenticate, ExtractCredentials, VerifyCredentials, authenticate::AuthError};

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

impl<E: ExtractCredentials, V: VerifyCredentials<Credentials = E::Credentials>> Authenticate for Authenticator<E, V>
where AuthError: From<E::Error> + From<V::Error>{
    type Context = E::Context;
    type Identity = V::Identity;

    async fn authenticate(
        &self,
        ctx: &Self::Context,
    ) -> Result<AuthOutcome<Self::Identity>, AuthError> {
        let Some(creds) = self.extractor.extract(ctx)? else {
            return Ok(AuthOutcome::NotApplicable)
        };
        let Some(identity) = self.verifier.verify(&creds).await? else {
            return Ok(AuthOutcome::Rejected)
        };
        Ok(AuthOutcome::Authenticated(identity))
    }
}

