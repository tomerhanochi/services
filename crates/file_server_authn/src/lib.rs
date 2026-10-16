//! Authentication composition for a file server: break-glass Basic, OIDC cookie, API bearer.

mod authenticator;
mod config;
mod error;
mod setup;
mod token_store;

pub mod prelude {
    pub use crate::authenticator::{
        AppAuthenticator, OidcAuthenticator, StaticAuthenticator, TokenAuthenticator,
    };
    pub use crate::config::{AuthConfig, OidcAuthConfig, StaticAuthConfig, TokenAuthConfig};
    pub use crate::error::BuildError;
    pub use crate::setup::{AuthSetup, configure};
    pub use crate::token_store::PostgresTokens;
    pub use authn_identity::prelude::{Identity, Machine, MachineId, User, UserId};
}
