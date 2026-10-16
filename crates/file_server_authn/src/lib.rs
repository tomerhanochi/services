//! Authentication composition for a file server: break-glass Basic, OIDC cookie, API bearer.

use std::{error::Error, fmt};

use authn::prelude::Authenticator;
use authn_oidc::prelude::{
    OidcClient, OidcConfig, OidcError, OidcIdentity, OidcVerifier, Sessions,
};
use authn_oidc_axum::prelude::{CookieSettings, OidcHttp, SessionCookieExtractor, SuccessPath};
use authn_static::prelude::{Password, StaticVerifier, Username};
use authn_static_axum::prelude::BasicAuthExtractor;
use authn_token::prelude::{TokenStore, TokenVerifier};
use authn_token_axum::prelude::BearerTokenExtractor;
use axum::Router;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Identity {
    BreakGlass { username: String },
    Oidc(OidcIdentity),
    ApiToken { subject: String },
}

impl From<OidcIdentity> for Identity {
    fn from(identity: OidcIdentity) -> Self {
        Self::Oidc(identity)
    }
}

pub type StaticAuthenticator = Authenticator<BasicAuthExtractor, StaticVerifier<Identity>>;
pub type OidcAuthenticator = Authenticator<SessionCookieExtractor, OidcVerifier<Identity>>;
pub type TokenAuthenticator<S> = Authenticator<BearerTokenExtractor, TokenVerifier<S>>;
pub type AppAuthenticator<S> = (
    StaticAuthenticator,
    Option<OidcAuthenticator>,
    Option<TokenAuthenticator<S>>,
);

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

#[derive(Debug)]
pub enum BuildError {
    StaticCredentials(&'static str),
    Oidc(OidcError),
}

impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StaticCredentials(message) => f.write_str(message),
            Self::Oidc(error) => error.fmt(f),
        }
    }
}

impl Error for BuildError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Oidc(error) => Some(error),
            _ => None,
        }
    }
}

impl From<OidcError> for BuildError {
    fn from(error: OidcError) -> Self {
        Self::Oidc(error)
    }
}

/// Builds the request authenticator without network access.
/// For OIDC login routes, use `configure` with the same config and shared sessions.
pub fn authenticator<S>(config: &AuthConfig<S>) -> Result<AppAuthenticator<S>, BuildError>
where
    S: TokenStore<Identity = Identity> + Clone + Sync,
{
    let credentials = &config.r#static;
    let static_auth = Authenticator::new(
        BasicAuthExtractor,
        StaticVerifier::with_identity(
            &credentials.username,
            &credentials.password,
            Identity::BreakGlass {
                username: credentials.username.as_str().to_owned(),
            },
        ),
    );
    let oidc = config.oidc.as_ref().map(|config| {
        Authenticator::new(
            config.cookies.extractor(),
            OidcVerifier::<Identity>::new(config.sessions.clone()),
        )
    });
    let token = config.token.as_ref().map(|config| {
        Authenticator::new(
            BearerTokenExtractor,
            TokenVerifier::new(config.store.clone()),
        )
    });
    Ok((static_auth, oidc, token))
}

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
