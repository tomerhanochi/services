use crate::{AuthOutcome, Authenticate, authenticate::AuthError};

impl<A: Authenticate> Authenticate for Option<A> {
    type Context = A::Context;
    type Identity = A::Identity;

    async fn authenticate(
        &self,
        ctx: &Self::Context,
    ) -> Result<AuthOutcome<Self::Identity>, AuthError> {
        match self {
            Some(auth) => auth.authenticate(ctx).await,
            None => Ok(AuthOutcome::NotApplicable),
        }
    }
}

