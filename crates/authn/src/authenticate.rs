#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthOutcome<T> {
    Authenticated(T),
    /// This mechanism found no credentials; another mechanism may be tried.
    NotApplicable,
    /// Credentials were present but invalid; authentication must stop.
    Rejected,
}

/// Establishes an identity from a context, using a future that can move between threads.
pub trait Authenticate {
    type Context;
    type Identity;
    type Error;

    fn authenticate(
        &self,
        ctx: &Self::Context,
    ) -> impl Future<Output = Result<AuthOutcome<Self::Identity>, Self::Error>> + Send;
}
