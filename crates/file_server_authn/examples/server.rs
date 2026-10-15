//! A composition example. Replace ExampleTokenStore with your persistent TokenStore.

use std::{collections::HashMap, convert::Infallible, env, sync::Arc};

use authn_axum::Authn;
use authn_oidc::OidcConfig;
use authn_token::{Token, TokenDigest, TokenStore};
use axum::{Router, routing::get};
use file_server_authn::{
    AppAuthenticator, AuthConfig, Identity, OidcAuthConfig, StaticAuthConfig, TokenAuthConfig,
    configure,
};

#[derive(Clone)]
struct ExampleTokenStore(Arc<HashMap<TokenDigest, Identity>>);

impl TokenStore for ExampleTokenStore {
    type Identity = Identity;
    type Error = Infallible;

    async fn lookup(&self, digest: &TokenDigest) -> Result<Option<Identity>, Infallible> {
        Ok(self.0.get(digest).cloned())
    }
}

type Auth = AppAuthenticator<ExampleTokenStore>;

async fn whoami(Authn(identity): Authn<Auth>) -> String {
    format!("{identity:?}")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = env::var("API_TOKEN").ok().map(|secret| TokenAuthConfig {
        store: ExampleTokenStore(Arc::new(HashMap::from([(
            Token::new(secret).digest(),
            Identity::ApiToken {
                subject: "example-client".into(),
            },
        )]))),
    });
    let oidc = match env::var("OIDC_ISSUER") {
        Err(env::VarError::NotPresent) => None,
        Err(error) => return Err(error.into()),
        Ok(issuer) => {
            let mut client = OidcConfig::new(
                issuer,
                env::var("OIDC_CLIENT_ID")?,
                env::var("OIDC_REDIRECT_URI")?,
            );
            client.client_secret = env::var("OIDC_CLIENT_SECRET").ok();
            Some(OidcAuthConfig::new(client))
        }
    };
    let config = AuthConfig {
        r#static: StaticAuthConfig {
            username: env::var("ADMIN_USERNAME")?,
            password: env::var("ADMIN_PASSWORD")?,
        },
        oidc,
        token,
    };
    let setup = configure(&config).await?;
    let app = Router::new()
        .route("/", get(whoami))
        .nest("/auth", setup.routes())
        .with_state(setup.authenticator);
    // Terminate HTTPS at a trusted local reverse proxy. OIDC cookies are Secure.
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
