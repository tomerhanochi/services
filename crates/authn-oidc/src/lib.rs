mod client;
mod config;
mod error;
mod session;
mod session_id;
mod verifier;

pub mod prelude {
    pub use crate::client::{LoginRequest, OidcClient};
    pub use crate::config::{
        HttpsIssuerUrl, HttpsRedirectUrl, OidcClientId, OidcConfig, PositiveDuration,
    };
    pub use crate::error::OidcError;
    pub use crate::session::{OidcIdentity, Sessions};
    pub use crate::session_id::SessionId;
    pub use crate::verifier::OidcVerifier;
    pub use openidconnect::{ClientSecret, Scope};
}
