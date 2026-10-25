use crate::token_store::PostgresTokens;
use authn::prelude::Authenticator;
use authn_oidc::prelude::OidcVerifier;
use authn_oidc_axum::prelude::SessionCookieExtractor;
use authn_token::prelude::TokenVerifier;
use authn_token_axum::prelude::BearerTokenExtractor;

pub type OidcAuthenticator = Authenticator<SessionCookieExtractor, OidcVerifier>;
pub type TokenAuthenticator = Authenticator<BearerTokenExtractor, TokenVerifier<PostgresTokens>>;
pub type AppAuthenticator = (Option<OidcAuthenticator>, Option<TokenAuthenticator>);
