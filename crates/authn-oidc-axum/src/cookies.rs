use std::{convert::Infallible, time::Duration};

use authn::ExtractCredentials;
use authn_oidc::{OidcError, SessionId};
use axum::http::{HeaderMap, header::COOKIE, request::Parts};
use cookie::{Cookie, SameSite};

#[derive(Debug, Clone)]
pub struct CookieSettings {
    session_name: String,
    login_name: String,
}

impl Default for CookieSettings {
    fn default() -> Self {
        Self::new("__Host-oidc-session").expect("default cookie name is valid")
    }
}

impl CookieSettings {
    pub fn new(session_name: impl Into<String>) -> Result<Self, OidcError> {
        let session_name = session_name.into();
        if session_name.is_empty()
            || session_name.len() > 128
            || !session_name
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b"!#$%&'*+-.^_`|~".contains(&byte))
        {
            return Err(OidcError::Configuration("invalid session cookie name"));
        }
        let login_name = format!("{session_name}-login");
        Ok(Self {
            session_name,
            login_name,
        })
    }

    pub fn session_name(&self) -> &str {
        &self.session_name
    }

    pub fn extractor(&self) -> SessionCookieExtractor {
        SessionCookieExtractor {
            name: self.session_name.clone(),
        }
    }

    pub(crate) fn login_value<'a>(&self, headers: &'a HeaderMap) -> Option<&'a str> {
        cookie_value(headers, &self.login_name)
    }

    pub(crate) fn session_value<'a>(&self, headers: &'a HeaderMap) -> Option<&'a str> {
        cookie_value(headers, &self.session_name)
    }

    pub(crate) fn session_header(&self, value: &str, ttl: Duration) -> String {
        cookie_header(&self.session_name, value, ttl)
    }

    pub(crate) fn login_header(&self, value: &str, ttl: Duration) -> String {
        cookie_header(&self.login_name, value, ttl)
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

/// A duplicate or malformed cookie is an invalid credential, represented by `""`.
fn cookie_value<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    let mut value = None;
    for header in headers.get_all(COOKIE) {
        let Ok(header) = header.to_str() else {
            return Some("");
        };
        for pair in header.split(';').map(str::trim) {
            let Some((key, candidate)) = pair.split_once('=') else {
                if pair == name {
                    return Some("");
                }
                continue;
            };
            if key != name {
                continue;
            }
            if value.is_some() {
                return Some("");
            }
            value = Some(candidate);
        }
    }
    value
}

#[derive(Debug, Clone)]
pub struct SessionCookieExtractor {
    name: String,
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

    fn extract(&self, parts: &Parts) -> Result<Option<SessionId>, Infallible> {
        Ok(cookie_value(&parts.headers, &self.name).map(SessionId::from_cookie))
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
