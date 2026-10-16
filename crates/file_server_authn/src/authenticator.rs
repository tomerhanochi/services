use authn::prelude::Authenticator;
use authn_oidc::prelude::OidcVerifier;
use authn_oidc_axum::prelude::SessionCookieExtractor;
use authn_static::prelude::StaticVerifier;
use authn_static_axum::prelude::BasicAuthExtractor;
use authn_token::prelude::{TokenStore, TokenVerifier};
use authn_token_axum::prelude::BearerTokenExtractor;

use crate::{config::AuthConfig, error::BuildError, identity::Identity};

pub type StaticAuthenticator = Authenticator<BasicAuthExtractor, StaticVerifier<Identity>>;
pub type OidcAuthenticator = Authenticator<SessionCookieExtractor, OidcVerifier<Identity>>;
pub type TokenAuthenticator<S> = Authenticator<BearerTokenExtractor, TokenVerifier<S>>;
pub type AppAuthenticator<S> = (
    StaticAuthenticator,
    Option<OidcAuthenticator>,
    Option<TokenAuthenticator<S>>,
);

/// Builds the request authenticator without network access.
/// For OIDC login routes, use `configure` with the same config and shared sessions.
pub fn authenticator<S>(config: &AuthConfig<S>) -> Result<AppAuthenticator<S>, BuildError>
where
    S: TokenStore<Identity = Identity> + Clone + Sync,
{
    let credentials = &config.r#static;
    let static_auth = Authenticator::new(
        BasicAuthExtractor,
        StaticVerifier::with_identity(
            &credentials.username,
            &credentials.password,
            Identity::BreakGlass {
                username: credentials.username.as_str().to_owned(),
            },
        ),
    );
    let oidc = config.oidc.as_ref().map(|config| {
        Authenticator::new(
            config.cookies.extractor(),
            OidcVerifier::<Identity>::new(config.sessions.clone()),
        )
    });
    let token = config.token.as_ref().map(|config| {
        Authenticator::new(
            BearerTokenExtractor,
            TokenVerifier::new(config.store.clone()),
        )
    });
    Ok((static_auth, oidc, token))
}
