use authn::prelude::{AuthOutcome, Authenticate};
use authn_identity::prelude::{Identity, Machine, MachineId};
use authn_token::prelude::Token;
use axum::http::Request;
use file_server_authn::prelude::*;

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL pointing to a disposable PostgreSQL database"]
async fn configured_postgres_tokens_enforce_revocation_and_expiry() {
    let url = std::env::var("TEST_DATABASE_URL").unwrap();
    let setup = configure(&AuthConfig {
        token: Some(TokenAuthConfig {
            database_url: url.clone(),
            max_connections: 2.try_into().unwrap(),
        }),
        ..Default::default()
    })
    .await
    .unwrap();
    let pool = sqlx::PgPool::connect(&url).await.unwrap();
    let token = Token::generate().unwrap();
    let digest = token.digest();
    let identity = Identity::Machine(Machine {
        id: MachineId::new("config-test").unwrap(),
    });
    sqlx::query("INSERT INTO authn_api_tokens (digest, identity) VALUES ($1, $2)")
        .bind(digest.0.as_slice())
        .bind(sqlx::types::Json(&identity))
        .execute(&pool)
        .await
        .unwrap();
    let parts = Request::builder()
        .header(
            "authorization",
            format!(
                "Bearer {}",
                std::str::from_utf8(token.expose_secret()).unwrap()
            ),
        )
        .body(())
        .unwrap()
        .into_parts()
        .0;
    assert_eq!(
        setup.authenticator.authenticate(&parts).await.unwrap(),
        AuthOutcome::Authenticated(identity)
    );
    sqlx::query("UPDATE authn_api_tokens SET revoked = true WHERE digest = $1")
        .bind(digest.0.as_slice())
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(
        setup.authenticator.authenticate(&parts).await.unwrap(),
        AuthOutcome::Rejected
    );
    sqlx::query("UPDATE authn_api_tokens SET revoked = false, expires_at = CURRENT_TIMESTAMP - INTERVAL '1 second' WHERE digest = $1")
        .bind(digest.0.as_slice()).execute(&pool).await.unwrap();
    assert_eq!(
        setup.authenticator.authenticate(&parts).await.unwrap(),
        AuthOutcome::Rejected
    );
    sqlx::query("DELETE FROM authn_api_tokens WHERE digest = $1")
        .bind(digest.0.as_slice())
        .execute(&pool)
        .await
        .unwrap();
}
