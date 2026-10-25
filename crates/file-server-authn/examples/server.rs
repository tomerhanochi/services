use authn_axum::prelude::Authn;
use authn_oidc_axum::prelude::{callback, login, logout};
use axum::{
    Router,
    routing::{get, post},
};
use file_server_authn::prelude::*;

#[derive(Clone, Copy)]
enum Route {
    Home,
    OidcLogin,
    OidcCallback,
    OidcLogout,
}

impl Route {
    const fn path(self) -> &'static str {
        match self {
            Self::Home => "/",
            Self::OidcLogin => "/auth/login",
            Self::OidcCallback => "/auth/callback",
            Self::OidcLogout => "/auth/logout",
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config_path = std::env::var("AUTH_CONFIG").unwrap_or_else(|_| "auth.toml".into());
    let config: AuthConfig = toml::from_str(&std::fs::read_to_string(config_path)?)?;
    let setup = configure(&config).await?;
    let app = Router::new()
        .route(
            Route::Home.path(),
            get(|Authn(identity): Authn<AppAuthenticator>| async move { format!("{identity:?}") }),
        )
        .with_state(setup.authenticator);
    let oidc = Router::new()
        .route(Route::OidcLogin.path(), get(login))
        .route(Route::OidcCallback.path(), get(callback))
        .route(Route::OidcLogout.path(), post(logout))
        .with_state(setup.oidc);
    let app = app.merge(oidc);
    axum::serve(tokio::net::TcpListener::bind("127.0.0.1:3000").await?, app).await?;
    Ok(())
}
