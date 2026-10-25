use authn_axum::prelude::Authn;
use axum::{Router, routing::get};
use file_server_authn::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = AuthConfig {
        r#static: Some(StaticAuthConfig {
            directory: std::env::var("STATIC_AUTH_DIRECTORY")?.into(),
        }),
        ..Default::default()
    };
    let setup = configure(&config).await?;
    let app = Router::new()
        .route(
            "/",
            get(|Authn(identity): Authn<AppAuthenticator>| async move { format!("{identity:?}") }),
        )
        .nest("/auth", setup.routes())
        .with_state(setup.authenticator);
    axum::serve(tokio::net::TcpListener::bind("127.0.0.1:3000").await?, app).await?;
    Ok(())
}
