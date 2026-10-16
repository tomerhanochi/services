use authn::prelude::{ExtractCredentials, Extracted};
use authn_axum::prelude::authorization;
use authn_token::prelude::Token;
use axum::http::request::Parts;
use std::convert::Infallible;

#[derive(Debug, Clone, Copy, Default)]
pub struct BearerTokenExtractor;

impl ExtractCredentials for BearerTokenExtractor {
    type Context = Parts;
    type Credentials = Token;
    type Error = Infallible;

    fn extract(&self, ctx: &Parts) -> Result<Extracted<Token>, Infallible> {
        Ok(authorization(ctx, "Bearer").parse(Token::new))
    }
}
