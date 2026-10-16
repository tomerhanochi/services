#[derive(Debug, thiserror::Error)]
pub enum BuildError {
    #[error(transparent)]
    Oidc(#[from] authn_oidc::prelude::OidcError),
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error("at least one authentication mechanism must be configured")]
    NoMechanisms,
}
