//! Opaque session cookie extraction and OIDC login/callback/logout routes.

mod cookies;
mod routes;

pub use cookies::{CookieSettings, SessionCookieExtractor};
pub use routes::{OidcHttp, OidcHttpError};
