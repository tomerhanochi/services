//! One page per module. Each returns the whole document, built with [`page`](crate::page::page).

mod actions;
mod directory;
mod error;
mod home;
mod move_picker;
mod rename;
mod select;

pub use actions::actions;
pub use directory::directory;
pub use error::error;
pub use home::home;
pub use move_picker::move_picker;
pub use rename::rename;
pub use select::select;
