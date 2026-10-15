use std::{error::Error, fmt};

#[derive(Debug)]
pub enum OidcError {
    Configuration(&'static str),
    InvalidCallback,
    InvalidToken(Box<dyn Error + Send + Sync>),
    Provider(Box<dyn Error + Send + Sync>),
    StoreUnavailable,
    Capacity,
    Random(Box<dyn Error + Send + Sync>),
}

impl fmt::Display for OidcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Configuration(message) => message,
            Self::InvalidCallback => "invalid or expired OIDC callback",
            Self::InvalidToken(_) => "OIDC token rejected",
            Self::Provider(_) => "OIDC provider request failed",
            Self::StoreUnavailable => "OIDC session store unavailable",
            Self::Capacity => "OIDC session or login capacity exceeded",
            Self::Random(_) => "secure random generation failed",
        })
    }
}

impl Error for OidcError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidToken(error) | Self::Provider(error) | Self::Random(error) => {
                Some(error.as_ref())
            }
            _ => None,
        }
    }
}
