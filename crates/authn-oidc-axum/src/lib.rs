mod cookie_name;
mod cookies;
mod routes;
mod success_path;

pub mod prelude {
    pub use crate::cookie_name::CookieName;
    pub use crate::cookies::{CookieSettings, SessionCookieExtractor};
    pub use crate::routes::{Callback, OidcHttp, OidcHttpError, callback, login, logout};
    pub use crate::success_path::SuccessPath;
}
