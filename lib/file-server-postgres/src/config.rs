use serde::Deserialize;
use std::{fmt, num::NonZeroU32};

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatabaseConfig {
    pub url: String,
    #[serde(default = "connections")]
    pub max_connections: NonZeroU32,
}
fn connections() -> NonZeroU32 {
    NonZeroU32::new(10).unwrap()
}
impl fmt::Debug for DatabaseConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DatabaseConfig")
            .field("url", &"[REDACTED]")
            .field("max_connections", &self.max_connections)
            .finish()
    }
}
