use crate::config::AuthConfig;
use authn::prelude::Authenticator;
use authn_identity::prelude::Identity;
use authn_oidc::prelude::{OidcIdentityStore, OidcUsers, OidcVerifier};
use authn_oidc_axum::prelude::SessionCookieExtractor;
use authn_static::prelude::{StaticStore, StaticVerifier};
use authn_static_axum::prelude::BasicAuthExtractor;
use authn_static_directory::prelude::DirectoryStore;
use authn_token::prelude::{TokenStore, TokenVerifier};
use authn_token_axum::prelude::BearerTokenExtractor;

pub type StaticAuthenticator<S = DirectoryStore> =
    Authenticator<BasicAuthExtractor, StaticVerifier<S>>;
pub type OidcAuthenticator<S = OidcUsers> = Authenticator<SessionCookieExtractor, OidcVerifier<S>>;
pub type TokenAuthenticator<S> = Authenticator<BearerTokenExtractor, TokenVerifier<S>>;
pub type AppAuthenticator<T, S = DirectoryStore, O = OidcUsers> = (
    StaticAuthenticator<S>,
    Option<OidcAuthenticator<O>>,
    Option<TokenAuthenticator<T>>,
);

/// Builds the request authenticator without I/O. Stores already contain parsed configuration.
pub fn authenticator<T, S, O>(config: &AuthConfig<T, S, O>) -> AppAuthenticator<T, S, O>
where
    T: TokenStore<Identity = Identity> + Clone + Sync,
    S: StaticStore<Identity = Identity> + Clone + Sync,
    O: OidcIdentityStore + Clone + Sync,
{
    let static_auth = Authenticator::new(
        BasicAuthExtractor,
        StaticVerifier::new(config.r#static.store.clone()),
    );
    let oidc = config.oidc.as_ref().map(|config| {
        Authenticator::new(
            config.cookies.extractor(),
            OidcVerifier::new(config.sessions.clone(), config.store.clone()),
        )
    });
    let token = config.token.as_ref().map(|config| {
        Authenticator::new(
            BearerTokenExtractor,
            TokenVerifier::new(config.store.clone()),
        )
    });
    (static_auth, oidc, token)
}
