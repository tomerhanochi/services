mod authenticate;
mod authenticator;
mod either;
mod extract_credentials;
mod option;
mod refrence;
mod tuple;
mod verify_credentials;

pub mod prelude {
    pub use crate::authenticate::{AuthOutcome, Authenticate};
    pub use crate::authenticator::Authenticator;
    pub use crate::either::Either;
    pub use crate::extract_credentials::{ExtractCredentials, Extracted};
    pub use crate::verify_credentials::VerifyCredentials;
}
