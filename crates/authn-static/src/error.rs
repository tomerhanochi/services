use std::{error::Error, fmt};

#[derive(Debug)]
pub enum PasswordVerificationError {
    Hash(argon2::password_hash::Error),
    Worker(tokio::task::JoinError),
}
impl fmt::Display for PasswordVerificationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Hash(_) => f.write_str("password hash verification failed"),
            Self::Worker(_) => f.write_str("password verification worker failed"),
        }
    }
}
impl Error for PasswordVerificationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Hash(error) => Some(error),
            Self::Worker(error) => Some(error),
        }
    }
}
