mod components;
mod icons;
mod page;
mod pages;
mod urls;

pub mod prelude {
    pub use crate::components::{Date, Done, Moving, Size};
    pub use crate::icons::Icons;
    pub use crate::page::page;
    pub use crate::pages::{
        actions, directory, error, home, move_picker, rename, select, signed_out,
    };
    pub use crate::urls::{AUTH_CALLBACK, Asset, HOME, Href, LOGIN, LOGOUT, LoginHref, Route};
}
