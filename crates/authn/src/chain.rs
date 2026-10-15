use std::error::Error;

use crate::{AuthOutcome, Authenticate, Either};

/// Tries the second mechanism only when the first is not applicable.
#[derive(Debug, Clone)]
pub struct Chain<A, B>(pub A, pub B);

impl<A, B> Authenticate for Chain<A, B>
where
    A: Authenticate + Sync,
    A::Context: Sync,
    A::Identity: Send,
    A::Error: Error + Send + 'static,
    B: Authenticate<Context = A::Context, Identity = A::Identity> + Sync,
    B::Error: Error + Send + 'static,
{
    type Context = A::Context;
    type Identity = A::Identity;
    type Error = Either<A::Error, B::Error>;

    async fn authenticate(
        &self,
        ctx: &Self::Context,
    ) -> Result<AuthOutcome<Self::Identity>, Self::Error> {
        match self.0.authenticate(ctx).await.map_err(Either::Left)? {
            AuthOutcome::NotApplicable => self.1.authenticate(ctx).await.map_err(Either::Right),
            outcome => Ok(outcome),
        }
    }
}
