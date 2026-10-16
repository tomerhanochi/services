use std::error::Error;

use crate::{AuthOutcome, Authenticate, Either};

macro_rules! tuple_or_single {
    ($single:expr $(,)?) => {
        $single
    };
    ($first:expr, $($rest:expr),+ $(,)?) => {
        ($first, $($rest),+)
    };
}

macro_rules! tuple_or_single_type {
    ($single:ident $(,)?) => {
        $single
    };
    ($first:ident, $($rest:ident),+ $(,)?) => {
        ($first, $($rest),+)
    };
}

macro_rules! tuples {
    ($first:ident $($t:ident)*) => {
        #[allow(non_snake_case)]
        impl<$first, $($t,)*> Authenticate for ($first, $($t,)*)
        where
            $first: Authenticate + Sync,
            $first::Context: Sync,
            $first::Identity: Send,
            $first::Error: Error + Send,
            $(
                $t: Authenticate<Context = $first::Context, Identity = $first::Identity> + Sync,
                $t::Error: Error + Send,
            )*
         {
            type Context = $first::Context;
            type Identity = $first::Identity;
            type Error = Either<$first::Error, <tuple_or_single_type!($($t,)*) as Authenticate>::Error>;

            async fn authenticate(
                &self,
                ctx: &Self::Context,
            ) -> Result<AuthOutcome<Self::Identity>, Self::Error> {
                let (_, $($t,)*) = self;
                let rest = tuple_or_single!($($t,)*);
                match self.0.authenticate(ctx).await.map_err(Either::Left)? {
                    AuthOutcome::NotApplicable => rest
                        .authenticate(ctx)
                        .await
                        .map_err(Either::Right),
                    outcome => Ok(outcome),
                }
            }
        }
    };
}

macro_rules! all_tuples {
    ($first:ident $($rest:ident)+) => {
        tuples!($first $($rest)*);
        all_tuples!($($rest)*);
    };
    ($first:ident) => {};
    () => {};
}

all_tuples!(A5 A4 A3 A2 A1);
