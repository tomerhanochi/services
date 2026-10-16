use authn_identity::prelude::UserId;
use serde::Deserialize;
use std::{collections::BTreeMap, fmt, path::PathBuf};

/// Only serializable deployment inputs; setup creates all runtime resources.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthConfig {
    pub r#static: Option<StaticAuthConfig>,
    pub oidc: Option<OidcAuthConfig>,
    pub token: Option<TokenAuthConfig>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StaticAuthConfig {
    pub directory: PathBuf,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OidcAuthConfig {
    pub issuer: String,
    pub client_id: String,
    pub client_secret: Option<String>,
    pub redirect_uri: String,
    #[serde(default = "default_scopes")]
    pub scopes: Vec<String>,
    #[serde(default = "session_ttl")]
    pub session_ttl_secs: u64,
    #[serde(default = "login_ttl")]
    pub login_ttl_secs: u64,
    #[serde(default = "cookie_name")]
    pub cookie_name: String,
    #[serde(default = "success_path")]
    pub success_path: String,
    /// Provider subject -> application-local user ID; issuer is the configured provider.
    #[serde(default)]
    pub users: BTreeMap<String, UserId>,
}
fn default_scopes() -> Vec<String> {
    vec!["email".into(), "profile".into()]
}
fn session_ttl() -> u64 {
    3600
}
fn login_ttl() -> u64 {
    600
}
fn cookie_name() -> String {
    "__Host-oidc-session".into()
}
fn success_path() -> String {
    "/".into()
}
impl fmt::Debug for OidcAuthConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OidcAuthConfig")
            .field("issuer", &self.issuer)
            .field("client_id", &self.client_id)
            .field("client_secret", &"[REDACTED]")
            .field("redirect_uri", &self.redirect_uri)
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TokenAuthConfig {
    pub database_url: String,
    #[serde(default = "max_connections")]
    pub max_connections: u32,
}
fn max_connections() -> u32 {
    5
}
impl fmt::Debug for TokenAuthConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TokenAuthConfig")
            .field("database_url", &"[REDACTED]")
            .field("max_connections", &self.max_connections)
            .finish()
    }
}
