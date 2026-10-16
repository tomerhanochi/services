use crate::config::DatabaseConfig;
use authn_identity::prelude::Identity;
use file_server_core::prelude::{AccessStore, Capabilities, Grant, MountName};
use sqlx::{PgPool, postgres::PgPoolOptions};

static MIGRATIONS: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

#[derive(Debug, Clone)]
pub struct PostgresAccess {
    pool: PgPool,
}
impl PostgresAccess {
    pub async fn connect(
        config: &DatabaseConfig,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections.get())
            .acquire_timeout(std::time::Duration::from_secs(10))
            .connect(&config.url)
            .await?;
        MIGRATIONS.run(&pool).await?;
        Ok(Self { pool })
    }
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Replace only config-owned access records atomically. API tokens are unaffected.
    /// Transactions are local SQLx transactions, with no unit-of-work abstraction.
    pub async fn sync_access(
        &self,
        grants: &[(String, MountName, Grant)],
        principals: &[(Identity, Vec<String>)],
    ) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("SELECT pg_advisory_xact_lock(739104608142)")
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM file_server_principal_roles")
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM file_server_role_grants")
            .execute(&mut *tx)
            .await?;
        for (role, mount, grant) in grants {
            sqlx::query(
                "INSERT INTO file_server_role_grants (role, mount, permission) VALUES ($1, $2, $3)",
            )
            .bind(role)
            .bind(mount.as_str())
            .bind(grant.as_str())
            .execute(&mut *tx)
            .await?;
        }
        for (identity, roles) in principals {
            let (kind, id) = principal(identity);
            for role in roles {
                sqlx::query("INSERT INTO file_server_principal_roles (kind, principal_id, role) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING")
                    .bind(kind).bind(id).bind(role).execute(&mut *tx).await?;
            }
        }
        tx.commit().await
    }
}
fn principal(identity: &Identity) -> (&'static str, &str) {
    match identity {
        Identity::User(user) => ("user", user.id.as_str()),
        Identity::Machine(machine) => ("machine", machine.id.as_str()),
    }
}
impl AccessStore for PostgresAccess {
    async fn capabilities(
        &self,
        identity: &Identity,
        mount: &MountName,
    ) -> Result<Capabilities, Box<dyn std::error::Error + Send + Sync>> {
        let (kind, id) = principal(identity);
        let (read, write) = sqlx::query_as::<_, (bool, bool)>(
            "SELECT COALESCE(bool_or(g.permission IN ('viewer', 'editor')), false), COALESCE(bool_or(g.permission = 'editor'), false) FROM file_server_principal_roles p JOIN file_server_role_grants g ON g.role = p.role WHERE p.kind = $1 AND p.principal_id = $2 AND g.mount = $3"
        ).bind(kind).bind(id).bind(mount.as_str()).fetch_one(&self.pool).await?;
        Ok(Capabilities {
            list: read,
            download: read,
            rename: write,
            moves: write,
        })
    }
}
