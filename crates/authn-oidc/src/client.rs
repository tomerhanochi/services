use std::{
    collections::HashMap,
    fmt,
    sync::{Arc, Mutex},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use openidconnect::{
    AccessTokenHash, AuthorizationCode, ClaimsVerificationError, ClientId, ClientSecret, CsrfToken,
    EndpointMaybeSet, EndpointNotSet, EndpointSet, IssuerUrl, JsonWebKey, JwsSigningAlgorithm,
    Nonce, OAuth2TokenResponse, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope,
    SignatureVerificationError, TokenResponse,
    core::{CoreAuthenticationFlow, CoreClient, CoreJsonWebKey, CoreProviderMetadata},
    reqwest,
};
use subtle::ConstantTimeEq;

use crate::{OidcError, OidcIdentity, SessionId, Sessions};

type DiscoveredClient = CoreClient<
    EndpointSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointMaybeSet,
    EndpointMaybeSet,
>;

#[derive(Clone)]
pub struct OidcConfig {
    pub issuer: String,
    pub client_id: String,
    pub client_secret: Option<String>,
    pub redirect_uri: String,
    pub scopes: Vec<String>,
    pub session_ttl: Duration,
    pub login_ttl: Duration,
}

impl OidcConfig {
    pub fn new(
        issuer: impl Into<String>,
        client_id: impl Into<String>,
        redirect_uri: impl Into<String>,
    ) -> Self {
        Self {
            issuer: issuer.into(),
            client_id: client_id.into(),
            redirect_uri: redirect_uri.into(),
            client_secret: None,
            scopes: vec!["email".into(), "profile".into()],
            session_ttl: Duration::from_secs(3600),
            login_ttl: Duration::from_secs(600),
        }
    }
}

impl fmt::Debug for OidcConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OidcConfig")
            .field("issuer", &self.issuer)
            .field("client_id", &self.client_id)
            .field("redirect_uri", &self.redirect_uri)
            .field("client_secret", &"[REDACTED]")
            .finish_non_exhaustive()
    }
}

struct PendingLogin {
    nonce: Nonce,
    pkce: PkceCodeVerifier,
    expires_at: SystemTime,
}

/// The state must be stored in a browser-bound cookie by the HTTP adapter.
pub struct LoginRequest {
    pub url: String,
    pub state: CsrfToken,
}

#[derive(Clone)]
pub struct OidcClient {
    inner: Arc<Inner>,
}

struct Inner {
    client: Mutex<DiscoveredClient>,
    http: reqwest::Client,
    config: OidcConfig,
    sessions: Sessions,
    pending: Mutex<HashMap<String, PendingLogin>>,
}

impl OidcClient {
    pub async fn discover(config: OidcConfig, sessions: Sessions) -> Result<Self, OidcError> {
        if config.client_id.is_empty() || config.session_ttl.is_zero() || config.login_ttl.is_zero()
        {
            return Err(OidcError::Configuration(
                "client ID and positive session/login lifetimes are required",
            ));
        }
        let issuer = IssuerUrl::new(config.issuer.clone())
            .map_err(|_| OidcError::Configuration("invalid OIDC issuer URL"))?;
        let redirect = RedirectUrl::new(config.redirect_uri.clone())
            .map_err(|_| OidcError::Configuration("invalid OIDC callback URL"))?;
        if issuer.url().scheme() != "https" || redirect.url().scheme() != "https" {
            return Err(OidcError::Configuration(
                "OIDC issuer and callback URLs must use HTTPS",
            ));
        }
        let http = reqwest::Client::builder()
            .https_only(true)
            .redirect(reqwest::redirect::Policy::none())
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|error| OidcError::Provider(Box::new(error)))?;
        let metadata = CoreProviderMetadata::discover_async(issuer, &http)
            .await
            .map_err(|error| OidcError::Provider(Box::new(error)))?;
        if metadata.authorization_endpoint().url().scheme() != "https"
            || metadata
                .token_endpoint()
                .is_none_or(|url| url.url().scheme() != "https")
            || metadata.jwks_uri().url().scheme() != "https"
        {
            return Err(OidcError::Configuration(
                "OIDC provider endpoints must use HTTPS",
            ));
        }
        let client = CoreClient::from_provider_metadata(
            metadata,
            ClientId::new(config.client_id.clone()),
            config.client_secret.clone().map(ClientSecret::new),
        )
        .set_redirect_uri(redirect);
        Ok(Self {
            inner: Arc::new(Inner {
                client: Mutex::new(client),
                http,
                config,
                sessions,
                pending: Mutex::new(HashMap::new()),
            }),
        })
    }

    fn client(&self) -> Result<DiscoveredClient, OidcError> {
        Ok(self
            .inner
            .client
            .lock()
            .map_err(|_| OidcError::StoreUnavailable)?
            .clone())
    }

    /// Refreshes discovery metadata and signing keys, preserving existing sessions and logins.
    pub async fn refresh_provider(&self) -> Result<(), OidcError> {
        let refreshed = Self::discover(self.inner.config.clone(), self.sessions()).await?;
        *self
            .inner
            .client
            .lock()
            .map_err(|_| OidcError::StoreUnavailable)? = refreshed.client()?;
        Ok(())
    }

    pub fn sessions(&self) -> Sessions {
        self.inner.sessions.clone()
    }

    pub fn session_ttl(&self) -> Duration {
        self.inner.config.session_ttl
    }

    pub fn login_ttl(&self) -> Duration {
        self.inner.config.login_ttl
    }

    pub fn begin_login(&self) -> Result<LoginRequest, OidcError> {
        let (challenge, pkce) = PkceCodeChallenge::new_random_sha256();
        let client = self.client()?;
        let mut request = client
            .authorize_url(
                CoreAuthenticationFlow::AuthorizationCode,
                CsrfToken::new_random,
                Nonce::new_random,
            )
            .set_pkce_challenge(challenge);
        for scope in &self.inner.config.scopes {
            request = request.add_scope(Scope::new(scope.clone()));
        }
        let (url, state, nonce) = request.url();
        let now = SystemTime::now();
        let expires_at = now
            .checked_add(self.login_ttl())
            .ok_or(OidcError::Configuration("login lifetime too large"))?;
        let mut pending = self
            .inner
            .pending
            .lock()
            .map_err(|_| OidcError::StoreUnavailable)?;
        pending.retain(|_, login| login.expires_at > now);
        if pending.len() >= 1024 {
            return Err(OidcError::Capacity);
        }
        pending.insert(
            state.secret().clone(),
            PendingLogin {
                nonce,
                pkce,
                expires_at,
            },
        );
        Ok(LoginRequest {
            url: url.to_string(),
            state,
        })
    }

    pub async fn complete_login(
        &self,
        code: &str,
        state: &str,
        browser_state: &str,
    ) -> Result<SessionId, OidcError> {
        if code.is_empty()
            || state.is_empty()
            || !bool::from(state.as_bytes().ct_eq(browser_state.as_bytes()))
        {
            return Err(OidcError::InvalidCallback);
        }
        // Consume the transaction before the network request; it can only be used once.
        let login = self
            .inner
            .pending
            .lock()
            .map_err(|_| OidcError::StoreUnavailable)?
            .remove(state)
            .filter(|login| login.expires_at > SystemTime::now())
            .ok_or(OidcError::InvalidCallback)?;
        let client = self.client()?;
        let response = client
            .exchange_code(AuthorizationCode::new(code.to_owned()))
            .map_err(|_| OidcError::Configuration("OIDC token endpoint missing"))?
            .set_pkce_verifier(login.pkce)
            .request_async(&self.inner.http)
            .await
            .map_err(|error| {
                if matches!(&error, openidconnect::RequestTokenError::ServerResponse(_)) {
                    OidcError::InvalidToken(Box::new(error))
                } else {
                    OidcError::Provider(Box::new(error))
                }
            })?;
        let token = response.id_token().ok_or(OidcError::InvalidCallback)?;
        let refreshed_client;
        let mut verifier = client.id_token_verifier();
        let claims = match token.claims(&verifier, &login.nonce) {
            Ok(claims) => claims,
            Err(ClaimsVerificationError::SignatureVerification(
                SignatureVerificationError::NoMatchingKey,
            )) => {
                // A provider may have rotated its keys since discovery. Refresh once.
                self.refresh_provider().await?;
                refreshed_client = self.client()?;
                verifier = refreshed_client.id_token_verifier();
                token
                    .claims(&verifier, &login.nonce)
                    .map_err(|error| OidcError::InvalidToken(Box::new(error)))?
            }
            Err(error) => return Err(OidcError::InvalidToken(Box::new(error))),
        };
        if let Some(expected) = claims.access_token_hash() {
            let algorithm = token
                .signing_alg()
                .map_err(|error| OidcError::InvalidToken(Box::new(error)))?;
            let key = if algorithm.uses_shared_secret() {
                let secret = self
                    .inner
                    .config
                    .client_secret
                    .as_ref()
                    .ok_or(OidcError::InvalidCallback)?;
                CoreJsonWebKey::new_symmetric(secret.as_bytes().to_vec())
            } else {
                token
                    .signing_key(&verifier)
                    .map_err(|error| OidcError::InvalidToken(Box::new(error)))?
                    .clone()
            };
            let actual = AccessTokenHash::from_token(response.access_token(), algorithm, &key)
                .map_err(|error| OidcError::InvalidToken(Box::new(error)))?;
            if !bool::from(
                actual
                    .as_str()
                    .as_bytes()
                    .ct_eq(expected.as_str().as_bytes()),
            ) {
                return Err(OidcError::InvalidCallback);
            }
        }
        let identity = OidcIdentity {
            issuer: claims.issuer().as_str().to_owned(),
            subject: claims.subject().as_str().to_owned(),
            email: claims.email().map(|email| email.as_str().to_owned()),
            email_verified: claims.email_verified().unwrap_or(false),
        };
        let expires = u64::try_from(claims.expiration().timestamp())
            .map_err(|_| OidcError::InvalidCallback)?;
        let token_expiry = UNIX_EPOCH
            .checked_add(Duration::from_secs(expires))
            .ok_or(OidcError::InvalidCallback)?;
        let session_expiry = SystemTime::now()
            .checked_add(self.session_ttl())
            .ok_or(OidcError::Configuration("session lifetime too large"))?;
        self.inner
            .sessions
            .issue(identity, token_expiry.min(session_expiry))
    }
}

impl fmt::Debug for OidcClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OidcClient")
            .field("config", &self.inner.config)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests;
