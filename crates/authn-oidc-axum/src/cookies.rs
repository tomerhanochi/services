use std::{convert::Infallible, time::Duration};

use crate::cookie_name::CookieName;
use authn::prelude::{ExtractCredentials, Extracted};
use authn_oidc::prelude::{OidcError, SessionId};
use axum::http::{HeaderMap, header::COOKIE, request::Parts};
use cookie::{Cookie, SameSite};

#[derive(Debug, Clone)]
pub struct CookieSettings {
    session_name: CookieName,
    login_name: CookieName,
}

impl Default for CookieSettings {
    fn default() -> Self {
        Self::new("__Host-oidc-session").expect("default cookie name is valid")
    }
}

impl CookieSettings {
    pub fn new(session_name: impl Into<String>) -> Result<Self, OidcError> {
        let session_name = CookieName::new(session_name)?;
        let login_name = CookieName::new(format!("{}-login", session_name.as_str()))?;
        Ok(Self {
            session_name,
            login_name,
        })
    }

    pub fn session_name(&self) -> &str {
        self.session_name.as_str()
    }

    pub fn extractor(&self) -> SessionCookieExtractor {
        SessionCookieExtractor {
            name: self.session_name.clone(),
        }
    }

    pub(crate) fn login_value<'a>(&self, headers: &'a HeaderMap) -> Extracted<&'a str> {
        cookie_value(headers, self.login_name.as_str())
    }

    pub(crate) fn session_value<'a>(&self, headers: &'a HeaderMap) -> Extracted<&'a str> {
        cookie_value(headers, self.session_name.as_str())
    }

    pub(crate) fn session_header(&self, value: &str, ttl: Duration) -> String {
        cookie_header(self.session_name.as_str(), value, ttl)
    }

    pub(crate) fn login_header(&self, value: &str, ttl: Duration) -> String {
        cookie_header(self.login_name.as_str(), value, ttl)
    }
}

fn cookie_header(name: &str, value: &str, ttl: Duration) -> String {
    Cookie::build((name, value))
        .path("/")
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Lax)
        .max_age(cookie::time::Duration::seconds(
            ttl.as_secs().min(i64::MAX as u64) as i64,
        ))
        .build()
        .to_string()
}

/// A duplicate or malformed cookie is an invalid credential, represented explicitly.
fn cookie_value<'a>(headers: &'a HeaderMap, name: &str) -> Extracted<&'a str> {
    let mut value = None;
    for header in headers.get_all(COOKIE) {
        let Ok(header) = header.to_str() else {
            return Extracted::Malformed;
        };
        for pair in header.split(';').map(str::trim) {
            let Some((key, candidate)) = pair.split_once('=') else {
                if pair == name {
                    return Extracted::Malformed;
                }
                continue;
            };
            if key != name {
                continue;
            }
            if value.is_some() {
                return Extracted::Malformed;
            }
            value = Some(candidate);
        }
    }
    value.map_or(Extracted::Absent, Extracted::Present)
}

#[derive(Debug, Clone)]
pub struct SessionCookieExtractor {
    name: CookieName,
}

impl Default for SessionCookieExtractor {
    fn default() -> Self {
        CookieSettings::default().extractor()
    }
}

impl SessionCookieExtractor {
    pub fn new(name: impl Into<String>) -> Result<Self, OidcError> {
        Ok(CookieSettings::new(name)?.extractor())
    }
}

impl ExtractCredentials for SessionCookieExtractor {
    type Context = Parts;
    type Credentials = SessionId;
    type Error = Infallible;

    fn extract(&self, parts: &Parts) -> Result<Extracted<SessionId>, Infallible> {
        Ok(cookie_value(&parts.headers, self.name.as_str()).parse(SessionId::from_cookie))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_and_login_cookies_are_secure_and_can_be_cleared() {
        let settings = CookieSettings::default();
        for header in [
            settings.session_header("session", Duration::from_secs(60)),
            settings.login_header("state", Duration::from_secs(60)),
        ] {
            let cookie = Cookie::parse(header).unwrap();
            assert_eq!(cookie.secure(), Some(true));
            assert_eq!(cookie.http_only(), Some(true));
            assert_eq!(cookie.same_site(), Some(SameSite::Lax));
            assert_eq!(cookie.path(), Some("/"));
            assert!(cookie.domain().is_none());
            assert_eq!(cookie.max_age().unwrap().whole_seconds(), 60);
        }
        let cookie = Cookie::parse(settings.session_header("", Duration::ZERO)).unwrap();
        assert_eq!(cookie.max_age().unwrap().whole_seconds(), 0);
    }

    #[test]
    fn unsafe_cookie_names_are_rejected() {
        for name in ["", "a=b", "a;b", "a b", "a\r\nb", "é"] {
            assert!(CookieSettings::new(name).is_err());
        }
    }
}
