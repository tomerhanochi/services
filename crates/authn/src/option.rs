use crate::{AuthOutcome, Authenticate};

impl<A> Authenticate for Option<A>
where
    A: Authenticate + Sync,
    A::Context: Sync,
{
    type Context = A::Context;
    type Identity = A::Identity;
    type Error = A::Error;

    async fn authenticate(
        &self,
        ctx: &Self::Context,
    ) -> Result<AuthOutcome<Self::Identity>, Self::Error> {
        match self {
            Some(auth) => auth.authenticate(ctx).await,
            None => Ok(AuthOutcome::NotApplicable),
        }
    }
}
