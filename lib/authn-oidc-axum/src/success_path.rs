use authn_oidc::prelude::OidcError;
use axum::http::HeaderValue;
use std::str::FromStr;

/// A local absolute redirect path suitable for a Location header.
#[derive(Debug, Clone)]
pub struct SuccessPath(String);
impl SuccessPath {
    pub fn new(value: impl Into<String>) -> Result<Self, OidcError> {
        let value = value.into();
        if !value.starts_with('/')
            || value.starts_with("//")
            || value.contains('\\')
            || HeaderValue::from_str(&value).is_err()
        {
            return Err(OidcError::Configuration(
                "login success path must be a local absolute path",
            ));
        }
        Ok(Self(value))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl FromStr for SuccessPath {
    type Err = OidcError;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}
