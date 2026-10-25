#[derive(Debug, thiserror::Error)]
pub enum BuildError {
    #[error(transparent)]
    Oidc(#[from] authn_oidc::prelude::OidcError),
    #[error(transparent)]
    TokenStore(#[from] authn_token_postgres::prelude::PostgresTokenError),
}
