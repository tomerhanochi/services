use authn_oidc::prelude::OidcClient;
use authn_oidc_axum::prelude::OidcHttp;
use authn_token::prelude::TokenStore;
use axum::Router;

use crate::{
    authenticator::{AppAuthenticator, authenticator},
    config::AuthConfig,
    error::BuildError,
    identity::Identity,
};

pub struct AuthSetup<S> {
    pub authenticator: AppAuthenticator<S>,
    pub oidc: Option<OidcHttp>,
}

impl<S> AuthSetup<S> {
    /// Mount with `.nest("/auth", setup.routes())` and use `/auth/callback` as redirect URI.
    pub fn routes<State: Clone + Send + Sync + 'static>(&self) -> Router<State> {
        self.oidc
            .as_ref()
            .map_or_else(Router::new, OidcHttp::router)
    }
}

/// Initializes OIDC discovery and returns request authentication plus its login routes.
pub async fn configure<S>(config: &AuthConfig<S>) -> Result<AuthSetup<S>, BuildError>
where
    S: TokenStore<Identity = Identity> + Clone + Sync,
{
    let authenticator = authenticator(config)?;
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
