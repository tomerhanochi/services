use authn::prelude::Authenticate;
use authn_identity::prelude::Identity;
use axum::http::request::Parts;
use file_server_core::prelude::{AccessStore, FileService, FileStore};
use file_server_views::prelude::Page;
use std::{fmt::Display, sync::Arc};

pub trait Ports: Send + Sync + 'static {
    type Files: FileStore + 'static;
    type Access: AccessStore + 'static;
    type Auth: Authenticate<Context = Parts, Identity = Identity, Error: Display>
        + Send
        + Sync
        + 'static;
    fn files(&self) -> &FileService<Self::Files, Self::Access>;
    fn authenticator(&self) -> &Self::Auth;
}
pub struct AppState<P> {
    pub ports: Arc<P>,
    pub css: Arc<String>,
    pub public_origin: String,
    pub oidc_enabled: bool,
}
impl<P> Clone for AppState<P> {
    fn clone(&self) -> Self {
        Self {
            ports: self.ports.clone(),
            css: self.css.clone(),
            public_origin: self.public_origin.clone(),
            oidc_enabled: self.oidc_enabled,
        }
    }
}
impl<P> AppState<P> {
    pub(crate) fn page<'a>(&self, user: Option<&'a Identity>) -> Page<'a> {
        Page {
            stylesheet: "/assets/app.css",
            logout: self.oidc_enabled,
            user: user.map(|identity| match identity {
                Identity::User(user) => user.id.as_str(),
                Identity::Machine(machine) => machine.id.as_str(),
            }),
        }
    }
}
