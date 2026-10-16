use crate::{
    authenticator::{AppAuthenticator, authenticator},
    config::AuthConfig,
    error::BuildError,
};
use authn_identity::prelude::Identity;
use authn_oidc::prelude::{OidcClient, OidcIdentityStore, OidcUsers};
use authn_oidc_axum::prelude::OidcHttp;
use authn_static::prelude::StaticStore;
use authn_static_directory::prelude::DirectoryStore;
use authn_token::prelude::TokenStore;
use axum::Router;

pub struct AuthSetup<T, S = DirectoryStore, O = OidcUsers> {
    pub authenticator: AppAuthenticator<T, S, O>,
    pub oidc: Option<OidcHttp>,
}
impl<T, S, O> AuthSetup<T, S, O> {
    /// Mount with `.nest("/auth", setup.routes())` and use `/auth/callback` as redirect URI.
    pub fn routes<State: Clone + Send + Sync + 'static>(&self) -> Router<State> {
        self.oidc
            .as_ref()
            .map_or_else(Router::new, OidcHttp::router)
    }
}

/// Initializes OIDC discovery and returns request authentication plus its login routes.
pub async fn configure<T, S, O>(
    config: &AuthConfig<T, S, O>,
) -> Result<AuthSetup<T, S, O>, BuildError>
where
    T: TokenStore<Identity = Identity> + Clone + Sync,
    S: StaticStore<Identity = Identity> + Clone + Sync,
    O: OidcIdentityStore + Clone + Sync,
{
    let authenticator = authenticator(config);
    let oidc = match &config.oidc {
        None => None,
        Some(config) => {
            let client =
                OidcClient::discover(config.client.clone(), config.sessions.clone()).await?;
            Some(OidcHttp::new(
                client,
                config.cookies.clone(),
                config.success_path.clone(),
            ))
        }
    };
    Ok(AuthSetup {
        authenticator,
        oidc,
    })
}
