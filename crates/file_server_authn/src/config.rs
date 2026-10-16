use authn_oidc::prelude::{OidcConfig, OidcUsers, Sessions};
use authn_oidc_axum::prelude::{CookieSettings, SuccessPath};
use authn_static_directory::prelude::DirectoryStore;

#[derive(Debug, Clone)]
pub struct StaticAuthConfig<S = DirectoryStore> {
    pub store: S,
}

#[derive(Debug, Clone)]
pub struct OidcAuthConfig<S = OidcUsers> {
    pub client: OidcConfig,
    pub store: S,
    pub sessions: Sessions,
    pub cookies: CookieSettings,
    pub success_path: SuccessPath,
}
impl<S> OidcAuthConfig<S> {
    pub fn new(client: OidcConfig, store: S) -> Self {
        Self {
            client,
            store,
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
pub struct AuthConfig<T, S = DirectoryStore, O = OidcUsers> {
    pub r#static: StaticAuthConfig<S>,
    pub oidc: Option<OidcAuthConfig<O>>,
    pub token: Option<TokenAuthConfig<T>>,
}
