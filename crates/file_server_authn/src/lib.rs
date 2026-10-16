//! Authentication composition for a file server: break-glass Basic, OIDC cookie, API bearer.

mod authenticator;
mod config;
mod error;
mod setup;

pub mod prelude {
    pub use crate::authenticator::{
        AppAuthenticator, OidcAuthenticator, StaticAuthenticator, TokenAuthenticator, authenticator,
    };
    pub use crate::config::{AuthConfig, OidcAuthConfig, StaticAuthConfig, TokenAuthConfig};
    pub use crate::error::BuildError;
    pub use crate::setup::{AuthSetup, configure};
    pub use authn_identity::prelude::{Identity, Machine, MachineId, User, UserId};
}
