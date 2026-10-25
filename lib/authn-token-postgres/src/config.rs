use serde::Deserialize;
use std::{fmt, num::NonZeroU32};

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresTokenConfig {
    pub database_url: String,
    #[serde(default = "max_connections")]
    pub max_connections: NonZeroU32,
}

fn max_connections() -> NonZeroU32 {
    NonZeroU32::new(5).unwrap()
}

impl fmt::Debug for PostgresTokenConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PostgresTokenConfig")
            .field("database_url", &"[REDACTED]")
            .field("max_connections", &self.max_connections)
            .finish()
    }
}
