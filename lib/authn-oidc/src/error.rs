use std::error::Error;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OidcError {
    #[error("{0}")]
    Configuration(&'static str),
    #[error("oidc")]
    Callback(#[from] OidcCallbackError),
    #[error("invalid session token")]
    InvalidSessionToken,
    #[error("oidc token verification: {0}")]
    InvalidToken(#[source] Box<dyn Error + Send + Sync>),
    #[error("oidc provider: {0}")]
    Provider(#[source] Box<dyn Error + Send + Sync>),
    #[error("oidc session store unavailable")]
    StoreUnavailable,
    #[error("oidc session or login capacity exceeded")]
    Capacity,
    #[error("secure random generation failed: {0}")]
    Random(#[source] Box<dyn Error + Send + Sync>),
}

#[derive(Debug, Error)]
pub enum OidcCallbackError {
    #[error("callback: provider rejected authorization")]
    ProviderRejected,
    #[error("callback: missing authorization code")]
    MissingAuthorizationCode,
    #[error("callback: missing state parameter")]
    MissingState,
    #[error("callback: missing login-state cookie")]
    MissingLoginStateCookie,
    #[error("callback: malformed login-state cookie")]
    MalformedLoginStateCookie,
    #[error("callback: empty authorization code or state")]
    EmptyCodeOrState,
    #[error("callback: state does not match browser cookie")]
    StateMismatch,
    #[error("callback: login state is unknown, expired, or already used")]
    InvalidLoginState,
    #[error("callback: provider response is missing an ID token")]
    MissingIdToken,
    #[error("callback: client secret required to verify access-token hash")]
    MissingClientSecret,
    #[error("callback: access-token hash does not match ID token")]
    AccessTokenHashMismatch,
    #[error("callback: invalid ID-token expiration")]
    InvalidTokenExpiration,
    #[error("callback: ID token is already expired")]
    SessionExpired,
}
