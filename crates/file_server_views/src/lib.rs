mod format;
mod pages;
mod urls;

pub mod prelude {
    pub use crate::format::{Date, Size};
    pub use crate::pages::{Done, Page, directory, error, home, move_picker, rename, signed_out};
    pub use crate::urls::{ASSETS, AUTH_CALLBACK, HOME, Href, LOGIN, LOGOUT, LoginHref, Route};
}
