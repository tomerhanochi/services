use authn::prelude::{ExtractCredentials, Extracted};
use authn_axum::prelude::authorization;
use authn_static::prelude::BasicCredentials;
use axum::http::request::Parts;
use base64::{Engine, engine::general_purpose::STANDARD};
use std::convert::Infallible;

#[derive(Debug, Clone, Copy, Default)]
pub struct BasicAuthExtractor;

impl ExtractCredentials for BasicAuthExtractor {
    type Context = Parts;
    type Credentials = BasicCredentials;
    type Error = Infallible;

    fn extract(&self, parts: &Parts) -> Result<Extracted<BasicCredentials>, Infallible> {
        Ok(authorization(parts, "Basic").parse(|encoded| {
            let bytes = STANDARD.decode(encoded).map_err(|_| ())?;
            let value = String::from_utf8(bytes).map_err(|_| ())?;
            BasicCredentials::parse(&value).map_err(|_| ())
        }))
    }
}
