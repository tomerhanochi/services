use std::fmt;

use authn_oidc::prelude::{OidcConfig, Sessions};
use authn_oidc_axum::prelude::{CookieSettings, SuccessPath};
use authn_static::prelude::{Password, Username};

#[derive(Clone)]
pub struct StaticAuthConfig {
    pub username: Username,
    pub password: Password,
}

impl fmt::Debug for StaticAuthConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StaticAuthConfig")
            .field("username", &self.username)
            .field("password", &"[REDACTED]")
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct OidcAuthConfig {
    pub client: OidcConfig,
    pub sessions: Sessions,
    pub cookies: CookieSettings,
    pub success_path: SuccessPath,
}

impl OidcAuthConfig {
    pub fn new(client: OidcConfig) -> Self {
        Self {
            client,
            sessions: Sessions::default(),
            cookies: CookieSettings::default(),
            success_path: SuccessPath::new("/").expect("default success path is valid"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TokenAuthConfig<S> {
    pub store: S,
}

#[derive(Debug, Clone)]
pub struct AuthConfig<S> {
    pub r#static: StaticAuthConfig,
    pub oidc: Option<OidcAuthConfig>,
    pub token: Option<TokenAuthConfig<S>>,
}
