pub struct AuthError;

pub enum AuthOutcome<T> {
    Authenticated(T),
    NotApplicable,
    Rejected,
}

pub trait Authenticate {
    type Context;
    type Identity;

    fn authenticate(
        &self,
        ctx: &Self::Context,
    ) -> impl Future<Output = Result<AuthOutcome<Self::Identity>, AuthError>>;
}
