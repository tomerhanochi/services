use crate::{authenticator::AppAuthenticator, config::AuthConfig, error::BuildError};
use authn::prelude::Authenticator;
use authn_oidc::prelude::{
    ClientSecret, OidcClient, OidcConfig, OidcUsers, OidcVerifier, PositiveDuration, Scope,
    Sessions,
};
use authn_oidc_axum::prelude::{CookieSettings, OidcHttp, SuccessPath};
use authn_token::prelude::TokenVerifier;
use authn_token_axum::prelude::BearerTokenExtractor;
use authn_token_postgres::prelude::PostgresTokens;
use std::time::Duration;

pub struct AuthSetup {
    pub authenticator: AppAuthenticator,
    pub oidc: OidcHttp,
}
/// Constructs stores, sessions, discovery clients and request authentication from config values.
#[hotpath::measure(future = true)]
pub async fn configure(config: &AuthConfig) -> Result<AuthSetup, BuildError> {
    let oidc_config = &config.oidc;
    let mut client_config = OidcConfig::new(
        &oidc_config.issuer,
        &oidc_config.client_id,
        &oidc_config.redirect_uri,
    )?;
    client_config.ca_certificate = oidc_config
        .tls_ca_file
        .as_ref()
        .map(std::fs::read)
        .transpose()
        .map_err(|error| authn_oidc::prelude::OidcError::Provider(Box::new(error)))?;
    client_config.client_secret = oidc_config.client_secret.clone().map(ClientSecret::new);
    client_config.scopes = oidc_config.scopes.iter().cloned().map(Scope::new).collect();
    client_config.session_ttl =
        PositiveDuration::new(Duration::from_secs(oidc_config.session_ttl_secs))?;
    client_config.login_ttl =
        PositiveDuration::new(Duration::from_secs(oidc_config.login_ttl_secs))?;
    let cookies = CookieSettings::new(&oidc_config.cookie_name)?;
    let success = SuccessPath::new(&oidc_config.success_path)?;
    let mut users = OidcUsers::default();
    for (subject, user) in &oidc_config.users {
        users.insert(&client_config.issuer, subject, user.clone());
    }
    let sessions = Sessions::default();
    let client = OidcClient::discover(client_config, sessions.clone()).await?;
    let oidc_auth = Authenticator::new(cookies.extractor(), OidcVerifier::new(sessions, users));
    let oidc = OidcHttp::new(client, cookies, success);
    let token = match &config.token {
        Some(config) => Some(Authenticator::new(
            BearerTokenExtractor,
            TokenVerifier::new(PostgresTokens::connect(config).await?),
        )),
        None => None,
    };
    Ok(AuthSetup {
        authenticator: (oidc_auth, token),
        oidc,
    })
}
