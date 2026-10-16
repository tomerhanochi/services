use crate::password_hash::PasswordHash;
use std::fmt;

/// A stored password hash and the application identity it authenticates.
pub struct StaticAccount<I> {
    pub password_hash: PasswordHash,
    pub identity: I,
}
impl<I> StaticAccount<I> {
    pub fn new(password_hash: PasswordHash, identity: I) -> Self {
        Self {
            password_hash,
            identity,
        }
    }
}
impl<I> fmt::Debug for StaticAccount<I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("StaticAccount([REDACTED])")
    }
}
