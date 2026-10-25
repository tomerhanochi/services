mod auth;
mod error;
mod files;
mod html;
mod routes;
mod security;
mod state;

pub mod prelude {
    pub use crate::routes::router;
    pub use crate::state::{AppState, Ports};
}
