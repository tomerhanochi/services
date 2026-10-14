use crate::{AuthOutcome, Authenticate, authenticate::AuthError};

pub struct Chain<A, B>(pub A, pub B);

impl<A, B> Authenticate for Chain<A, B>
where
    A: Authenticate,
    B: Authenticate<Context = A::Context, Identity = A::Identity>,
{
    type Context = A::Context;
    type Identity = A::Identity;

    async fn authenticate(
        &self,
        ctx: &Self::Context,
    ) -> Result<AuthOutcome<Self::Identity>, AuthError> {
        match self.0.authenticate(ctx).await? {
            AuthOutcome::NotApplicable => {
                self.1.authenticate(ctx).await
            }
            outcome => Ok(outcome),
        }
    }
}
