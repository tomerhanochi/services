mod config;
mod server;

pub mod prelude {
    pub use crate::config::{Config, Mount};
    pub use crate::server::{Server, run, shutdown_signal};
}
