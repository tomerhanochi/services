use authn_identity::prelude::{Identity, Machine, MachineId};
use authn_token::prelude::{Token, TokenStore};
use authn_token_postgres::prelude::{PostgresTokenConfig, PostgresTokens};

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL pointing to a disposable PostgreSQL database"]
async fn postgres_tokens_enforce_revocation_and_expiry() {
    let url = std::env::var("TEST_DATABASE_URL").unwrap();
    let store = PostgresTokens::connect(&PostgresTokenConfig {
        database_url: url.clone(),
        max_connections: 2.try_into().unwrap(),
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
    assert_eq!(store.lookup(&digest).await.unwrap(), Some(identity));
    sqlx::query("UPDATE authn_api_tokens SET revoked = true WHERE digest = $1")
        .bind(digest.0.as_slice())
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(store.lookup(&digest).await.unwrap(), None);
    sqlx::query("UPDATE authn_api_tokens SET revoked = false, expires_at = CURRENT_TIMESTAMP - INTERVAL '1 second' WHERE digest = $1")
        .bind(digest.0.as_slice())
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(store.lookup(&digest).await.unwrap(), None);
    sqlx::query("DELETE FROM authn_api_tokens WHERE digest = $1")
        .bind(digest.0.as_slice())
        .execute(&pool)
        .await
        .unwrap();
}
