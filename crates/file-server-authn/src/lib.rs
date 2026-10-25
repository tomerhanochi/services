//! Authentication composition for a file server: OIDC cookie and API bearer.

mod authenticator;
mod config;
mod error;
mod setup;

pub mod prelude {
    pub use crate::authenticator::{AppAuthenticator, OidcAuthenticator, TokenAuthenticator};
    pub use crate::config::{AuthConfig, OidcAuthConfig};
    pub use crate::error::BuildError;
    pub use crate::setup::{AuthSetup, configure};
    pub use authn_identity::prelude::{Identity, Machine, MachineId, User, UserId};
}
