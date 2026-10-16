use crate::token_store::PostgresTokens;
use authn::prelude::Authenticator;
use authn_oidc::prelude::OidcVerifier;
use authn_oidc_axum::prelude::SessionCookieExtractor;
use authn_static::prelude::StaticVerifier;
use authn_static_axum::prelude::BasicAuthExtractor;
use authn_static_directory::prelude::DirectoryStore;
use authn_token::prelude::TokenVerifier;
use authn_token_axum::prelude::BearerTokenExtractor;

pub type StaticAuthenticator = Authenticator<BasicAuthExtractor, StaticVerifier<DirectoryStore>>;
pub type OidcAuthenticator = Authenticator<SessionCookieExtractor, OidcVerifier>;
pub type TokenAuthenticator = Authenticator<BearerTokenExtractor, TokenVerifier<PostgresTokens>>;
pub type AppAuthenticator = (
    Option<StaticAuthenticator>,
    Option<OidcAuthenticator>,
    Option<TokenAuthenticator>,
);
