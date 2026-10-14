use authn::ExtractCredentials;
use authn_token::Token;
use axum::http::{header::AUTHORIZATION, request::Parts};

pub struct BearerTokenExtractor;

pub enum Error {}

impl ExtractCredentials for BearerTokenExtractor {
    type Context = Parts;
    type Credentials = Token;
    type Error = Error;

    fn extract(&self, ctx: &Self::Context) -> Result<Option<Self::Credentials>, Self::Error> {
        let Some(authorization) = ctx.headers.get(AUTHORIZATION) else {
            return Ok(None);
        };

        match authorization.as_bytes().strip_prefix(b"Bearer ") {
            Some(token) => Ok(Some(Token)),
            None => Ok(None),
        }
    }
}
