use crate::{authenticator::AppAuthenticator, config::AuthConfig, error::BuildError, token_store};
use authn::prelude::Authenticator;
use authn_oidc::prelude::{
    ClientSecret, OidcClient, OidcConfig, OidcUsers, OidcVerifier, PositiveDuration, Scope,
    Sessions,
};
use authn_oidc_axum::prelude::{CookieSettings, OidcHttp, SuccessPath};
use authn_token::prelude::TokenVerifier;
use authn_token_axum::prelude::BearerTokenExtractor;
use axum::Router;
use std::time::Duration;

pub struct AuthSetup {
    pub authenticator: AppAuthenticator,
    pub oidc: Option<OidcHttp>,
}
impl AuthSetup {
    pub fn routes<State: Clone + Send + Sync + 'static>(&self) -> Router<State> {
        self.oidc
            .as_ref()
            .map_or_else(Router::new, OidcHttp::router)
    }
}

/// Constructs stores, sessions, discovery clients and request authentication from config values.
#[hotpath::measure(future = true)]
pub async fn configure(config: &AuthConfig) -> Result<AuthSetup, BuildError> {
    if config.oidc.is_none() && config.token.is_none() {
        return Err(BuildError::NoMechanisms);
    }
    let (oidc_auth, oidc) = if let Some(config) = &config.oidc {
        let mut client_config =
            OidcConfig::new(&config.issuer, &config.client_id, &config.redirect_uri)?;
        client_config.ca_certificate =
            config
                .tls_ca_file
                .as_ref()
                .map(std::fs::read)
                .transpose()
                .map_err(|error| authn_oidc::prelude::OidcError::Provider(Box::new(error)))?;
        client_config.client_secret = config.client_secret.clone().map(ClientSecret::new);
        client_config.scopes = config.scopes.iter().cloned().map(Scope::new).collect();
        client_config.session_ttl =
            PositiveDuration::new(Duration::from_secs(config.session_ttl_secs))?;
        client_config.login_ttl =
            PositiveDuration::new(Duration::from_secs(config.login_ttl_secs))?;
        let cookies = CookieSettings::new(&config.cookie_name)?;
        let success = SuccessPath::new(&config.success_path)?;
        let mut users = OidcUsers::default();
        for (subject, user) in &config.users {
            users.insert(&client_config.issuer, subject, user.clone());
        }
        let sessions = Sessions::default();
        let client = OidcClient::discover(client_config, sessions.clone()).await?;
        (
            Some(Authenticator::new(
                cookies.extractor(),
                OidcVerifier::new(sessions, users),
            )),
            Some(OidcHttp::new(client, cookies, success)),
        )
    } else {
        (None, None)
    };
    let token = match &config.token {
        Some(config) => Some(Authenticator::new(
            BearerTokenExtractor,
            TokenVerifier::new(token_store::connect(config).await?),
        )),
        None => None,
    };
    Ok(AuthSetup {
        authenticator: (oidc_auth, token),
        oidc,
    })
}
