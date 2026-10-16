//! A composition example. Replace ExampleTokenStore with your persistent TokenStore.

use cap_std::{ambient_authority, fs::Dir};

use authn_static_directory::prelude::DirectoryStore;

use std::{collections::HashMap, convert::Infallible, env, sync::Arc};

use authn_axum::prelude::Authn;
use authn_oidc::prelude::{ClientSecret, OidcConfig, OidcUsers};
use authn_token::prelude::{Token, TokenDigest, TokenStore};
use axum::{Router, routing::get};
use file_server_authn::prelude::{
    AppAuthenticator, AuthConfig, Identity, Machine, MachineId, OidcAuthConfig, StaticAuthConfig,
    TokenAuthConfig, UserId, configure,
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
    let token = match env::var("API_TOKEN") {
        Ok(secret) => Some(TokenAuthConfig {
            store: ExampleTokenStore(Arc::new(HashMap::from([(
                Token::new(secret)?.digest(),
                Identity::Machine(Machine {
                    id: MachineId::new("example-client")?,
                }),
            )]))),
        }),
        Err(env::VarError::NotPresent) => None,
        Err(error) => return Err(error.into()),
    };
    let oidc = match env::var("OIDC_ISSUER") {
        Err(env::VarError::NotPresent) => None,
        Err(error) => return Err(error.into()),
        Ok(issuer) => {
            let mut client = OidcConfig::new(
                issuer,
                env::var("OIDC_CLIENT_ID")?,
                env::var("OIDC_REDIRECT_URI")?,
            )?;
            client.client_secret = env::var("OIDC_CLIENT_SECRET").ok().map(ClientSecret::new);
            let mut users = OidcUsers::default();
            users.insert(
                &client.issuer,
                env::var("OIDC_SUBJECT")?,
                UserId::new(env::var("OIDC_USER_ID")?)?,
            );
            Some(OidcAuthConfig::new(client, users))
        }
    };
    let config = AuthConfig {
        r#static: StaticAuthConfig {
            store: DirectoryStore::new(Dir::open_ambient_dir(
                env::var("STATIC_AUTH_DIRECTORY")?,
                ambient_authority(),
            )?),
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
