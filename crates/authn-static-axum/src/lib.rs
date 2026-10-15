//! HTTP Basic credential extraction for Axum.

use std::convert::Infallible;

use authn::ExtractCredentials;
use authn_static::BasicCredentials;
use axum::http::request::Parts;
use base64::{Engine, engine::general_purpose::STANDARD};

#[derive(Debug, Clone, Copy, Default)]
pub struct BasicAuthExtractor;

impl ExtractCredentials for BasicAuthExtractor {
    type Context = Parts;
    type Credentials = BasicCredentials;
    type Error = Infallible;

    fn extract(&self, parts: &Parts) -> Result<Option<BasicCredentials>, Infallible> {
        let Some(encoded) = authn_axum::authorization(parts, "Basic") else {
            return Ok(None);
        };
        let credentials = STANDARD
            .decode(encoded)
            .ok()
            .and_then(|bytes| String::from_utf8(bytes).ok())
            .and_then(|value| {
                value
                    .split_once(':')
                    .map(|(username, password)| BasicCredentials::new(username, password))
            })
            .unwrap_or(BasicCredentials::Malformed);
        Ok(Some(credentials))
    }
}
