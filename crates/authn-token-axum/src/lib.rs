//! Bearer token extraction for Axum request parts.

use std::convert::Infallible;

use authn::ExtractCredentials;
use authn_token::Token;
use axum::http::request::Parts;

#[derive(Debug, Clone, Copy, Default)]
pub struct BearerTokenExtractor;

impl ExtractCredentials for BearerTokenExtractor {
    type Context = Parts;
    type Credentials = Token;
    type Error = Infallible;

    fn extract(&self, ctx: &Self::Context) -> Result<Option<Self::Credentials>, Self::Error> {
        Ok(authn_axum::authorization(ctx, "Bearer").map(Token::new))
    }
}
