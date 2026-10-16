//! Authentication composition for a file server: break-glass Basic, OIDC cookie, API bearer.

mod authenticator;
mod config;
mod error;
mod identity;
mod setup;

pub mod prelude {
    pub use crate::authenticator::{
        AppAuthenticator, OidcAuthenticator, StaticAuthenticator, TokenAuthenticator, authenticator,
    };
    pub use crate::config::{AuthConfig, OidcAuthConfig, StaticAuthConfig, TokenAuthConfig};
    pub use crate::error::BuildError;
    pub use crate::identity::Identity;
    pub use crate::setup::{AuthSetup, configure};
}
