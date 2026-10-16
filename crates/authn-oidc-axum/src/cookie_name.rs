use authn_oidc::prelude::OidcError;
use std::str::FromStr;

/// An HTTP cookie name, limited to 128 bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CookieName(String);
impl CookieName {
    pub fn new(value: impl Into<String>) -> Result<Self, OidcError> {
        let value = value.into();
        if value.is_empty()
            || value.len() > 128
            || !value
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b"!#$%&'*+-.^_`|~".contains(&byte))
        {
            return Err(OidcError::Configuration("invalid cookie name"));
        }
        Ok(Self(value))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl FromStr for CookieName {
    type Err = OidcError;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}
