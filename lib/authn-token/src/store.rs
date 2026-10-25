use crate::digest::TokenDigest;
use std::{error::Error, sync::Arc};
pub trait TokenStore {
    type Identity;
    type Error: Error + Send + 'static;

    /// Returns only active, unexpired, unrevoked tokens. Unknown tokens return `None`.
    fn lookup(
        &self,
        digest: &TokenDigest,
    ) -> impl Future<Output = Result<Option<Self::Identity>, Self::Error>> + Send;
}

impl<S: TokenStore + Send + Sync> TokenStore for Arc<S> {
    type Identity = S::Identity;
    type Error = S::Error;

    async fn lookup(&self, digest: &TokenDigest) -> Result<Option<Self::Identity>, Self::Error> {
        self.as_ref().lookup(digest).await
    }
}
