use authn_identity::prelude::Identity;
use authn_token::prelude::{TokenDigest, TokenStore};
use sqlx::{PgPool, types::Json};

/// Application token records. Only SHA-256 digests are persisted.
#[derive(Debug, Clone)]
pub struct PostgresTokens(pub(crate) PgPool);
impl TokenStore for PostgresTokens {
    type Identity = Identity;
    type Error = sqlx::Error;
    #[hotpath::measure(future = true)]
    async fn lookup(&self, digest: &TokenDigest) -> Result<Option<Identity>, sqlx::Error> {
        let identity = sqlx::query_scalar::<_, Json<Identity>>(
            "SELECT identity FROM authn_api_tokens WHERE digest = $1 AND NOT revoked AND (expires_at IS NULL OR expires_at > CURRENT_TIMESTAMP)"
        ).bind(digest.0.as_slice()).fetch_optional(&self.0).await?;
        Ok(identity.map(|Json(identity)| identity))
    }
}

#[hotpath::measure(future = true)]
pub(crate) async fn connect(
    config: &crate::config::TokenAuthConfig,
) -> Result<PostgresTokens, sqlx::Error> {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(config.max_connections.get())
        .acquire_timeout(std::time::Duration::from_secs(10))
        .connect(&config.database_url)
        .await?;
    sqlx::query("CREATE TABLE IF NOT EXISTS authn_api_tokens (digest BYTEA PRIMARY KEY CHECK (octet_length(digest) = 32), identity JSONB NOT NULL, expires_at TIMESTAMPTZ, revoked BOOLEAN NOT NULL DEFAULT FALSE)")
        .execute(&pool).await?;
    Ok(PostgresTokens(pool))
}
