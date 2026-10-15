//! OIDC authorization-code login using `openidconnect`, followed by opaque sessions.

mod client;
mod error;
mod session;

pub use client::{LoginRequest, OidcClient, OidcConfig};
pub use error::OidcError;
pub use session::{OidcIdentity, OidcVerifier, SessionId, Sessions};
