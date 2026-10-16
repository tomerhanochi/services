use std::{error::Error, fmt};

use authn_oidc::prelude::OidcError;

#[derive(Debug)]
pub enum BuildError {
    Oidc(OidcError),
}

impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Oidc(error) => error.fmt(f),
        }
    }
}

impl Error for BuildError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Oidc(error) => Some(error),
        }
    }
}

impl From<OidcError> for BuildError {
    fn from(error: OidcError) -> Self {
        Self::Oidc(error)
    }
}
