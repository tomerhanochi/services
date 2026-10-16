use authn_oidc::prelude::OidcIdentity;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Identity {
    BreakGlass { username: String },
    Oidc(OidcIdentity),
    ApiToken { subject: String },
}

impl From<OidcIdentity> for Identity {
    fn from(identity: OidcIdentity) -> Self {
        Self::Oidc(identity)
    }
}
