use crate::prelude::OidcError;
use openidconnect::{ClientId, ClientSecret, IssuerUrl, RedirectUrl, Scope};
use std::{fmt, str::FromStr, time::Duration};

macro_rules! https_url {
    ($name:ident, $inner:ident) => {
        #[derive(Debug, Clone)]
        pub struct $name($inner);

        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self, OidcError> {
                let url = $inner::new(value.into())
                    .map_err(|_| OidcError::Configuration("invalid OIDC URL"))?;
                if url.url().scheme() != "https" {
                    return Err(OidcError::Configuration("OIDC URLs must use HTTPS"));
                }
                Ok(Self(url))
            }
            pub fn as_str(&self) -> &str {
                self.0.as_str()
            }
            pub(crate) fn inner(&self) -> &$inner {
                &self.0
            }
        }
        impl FromStr for $name {
            type Err = OidcError;
            fn from_str(value: &str) -> Result<Self, Self::Err> {
                Self::new(value)
            }
        }
    };
}
https_url!(HttpsIssuerUrl, IssuerUrl);
https_url!(HttpsRedirectUrl, RedirectUrl);

#[derive(Debug, Clone)]
pub struct OidcClientId(ClientId);
impl OidcClientId {
    pub fn new(value: impl Into<String>) -> Result<Self, OidcError> {
        let value = value.into();
        if value.is_empty() {
            return Err(OidcError::Configuration("client ID is required"));
        }
        Ok(Self(ClientId::new(value)))
    }
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
    pub(crate) fn inner(&self) -> &ClientId {
        &self.0
    }
}
impl FromStr for OidcClientId {
    type Err = OidcError;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}

/// A lifetime that can be added to the current system time.
#[derive(Debug, Clone, Copy)]
pub struct PositiveDuration(Duration);
impl PositiveDuration {
    pub fn new(value: Duration) -> Result<Self, OidcError> {
        if value.is_zero() || std::time::SystemTime::now().checked_add(value).is_none() {
            return Err(OidcError::Configuration(
                "a positive, representable lifetime is required",
            ));
        }
        Ok(Self(value))
    }
    pub fn get(self) -> Duration {
        self.0
    }
}
impl TryFrom<Duration> for PositiveDuration {
    type Error = OidcError;
    fn try_from(value: Duration) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

/// Configuration whose local syntax and lifetime invariants are established at construction.
#[derive(Clone)]
pub struct OidcConfig {
    pub issuer: HttpsIssuerUrl,
    pub client_id: OidcClientId,
    pub client_secret: Option<ClientSecret>,
    pub redirect_uri: HttpsRedirectUrl,
    pub scopes: Vec<Scope>,
    pub session_ttl: PositiveDuration,
    pub login_ttl: PositiveDuration,
}

impl OidcConfig {
    pub fn new(
        issuer: impl Into<String>,
        client_id: impl Into<String>,
        redirect_uri: impl Into<String>,
    ) -> Result<Self, OidcError> {
        Ok(Self {
            issuer: HttpsIssuerUrl::new(issuer)?,
            client_id: OidcClientId::new(client_id)?,
            redirect_uri: HttpsRedirectUrl::new(redirect_uri)?,
            client_secret: None,
            scopes: vec![Scope::new("email".into()), Scope::new("profile".into())],
            session_ttl: PositiveDuration(Duration::from_secs(3600)),
            login_ttl: PositiveDuration(Duration::from_secs(600)),
        })
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
