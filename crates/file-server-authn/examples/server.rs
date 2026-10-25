use authn_axum::prelude::Authn;
use axum::{Router, routing::get};
use file_server_authn::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = AuthConfig {
        token: Some(TokenAuthConfig {
            database_url: std::env::var("DATABASE_URL")?,
            max_connections: std::num::NonZeroU32::new(5).unwrap(),
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
