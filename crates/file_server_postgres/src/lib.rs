mod config;
mod store;

pub mod prelude {
    pub use crate::config::DatabaseConfig;
    pub use crate::store::PostgresAccess;
}
