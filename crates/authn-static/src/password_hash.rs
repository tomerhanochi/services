use crate::password::Password;
use argon2::{
    Argon2, Params, PasswordVerifier,
    password_hash::{Error, PasswordHashString},
};
use std::{fmt, str::FromStr};

/// A parsed Argon2id v19 PHC password hash with at least 19 MiB memory, two
/// iterations, a 16-byte salt, and a 32-byte output. Debug output is redacted.
#[derive(Clone)]
pub struct PasswordHash(PasswordHashString);

impl PasswordHash {
    pub fn new(encoded: &str) -> Result<Self, InvalidPasswordHash> {
        let encoded = PasswordHashString::new(encoded).map_err(|_| InvalidPasswordHash)?;
        let parsed = encoded.password_hash();
        if parsed.algorithm.as_str() != "argon2id" || parsed.version != Some(19) {
            return Err(InvalidPasswordHash);
        }
        let params = Params::try_from(&parsed).map_err(|_| InvalidPasswordHash)?;
        let mut salt = [0; 64];
        let salt = parsed
            .salt
            .ok_or(InvalidPasswordHash)?
            .decode_b64(&mut salt)
            .map_err(|_| InvalidPasswordHash)?;
        let output = parsed.hash.ok_or(InvalidPasswordHash)?;
        if params.m_cost() < Params::DEFAULT_M_COST
            || params.t_cost() < Params::DEFAULT_T_COST
            || salt.len() < 16
            || output.len() < 32
        {
            return Err(InvalidPasswordHash);
        }
        Ok(Self(encoded))
    }

    pub(crate) fn verify(&self, password: &Password) -> Result<bool, Error> {
        match Argon2::default()
            .verify_password(password.expose_secret().as_bytes(), &self.0.password_hash())
        {
            Ok(()) => Ok(true),
            Err(Error::Password) => Ok(false),
            Err(error) => Err(error),
        }
    }
}
impl FromStr for PasswordHash {
    type Err = InvalidPasswordHash;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}
impl fmt::Debug for PasswordHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("PasswordHash([REDACTED])")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidPasswordHash;
impl fmt::Display for InvalidPasswordHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("expected an Argon2id v19 PHC hash with at least 19 MiB memory, two iterations, a 16-byte salt and a 32-byte output")
    }
}
impl std::error::Error for InvalidPasswordHash {}
