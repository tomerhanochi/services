use crate::{AuthOutcome, Authenticate};

impl<T: Authenticate> Authenticate for &T {
    type Context = T::Context;
    type Identity = T::Identity;
    type Error = T::Error;

    fn authenticate(
        &self,
        ctx: &Self::Context,
    ) -> impl Future<Output = Result<AuthOutcome<Self::Identity>, Self::Error>> + Send {
        (**self).authenticate(ctx)
    }
}
