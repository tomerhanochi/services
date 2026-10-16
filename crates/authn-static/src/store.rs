use crate::{account::StaticAccount, username::Username};
use std::{error::Error, sync::Arc};

/// Application-owned accounts containing parsed secure password hashes, never plaintext.
/// Unknown or disabled users return `None`; operational failures return `Err`.
pub trait StaticStore {
    type Identity;
    type Error: Error + Send + 'static;

    fn lookup(
        &self,
        username: &Username,
    ) -> impl Future<Output = Result<Option<StaticAccount<Self::Identity>>, Self::Error>> + Send;
}

impl<S: StaticStore + Send + Sync> StaticStore for Arc<S> {
    type Identity = S::Identity;
    type Error = S::Error;
    async fn lookup(
        &self,
        username: &Username,
    ) -> Result<Option<StaticAccount<Self::Identity>>, Self::Error> {
        self.as_ref().lookup(username).await
    }
}
