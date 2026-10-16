use crate::{files::Capabilities, model::MountName};
use authn_identity::prelude::Identity;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Grant {
    Viewer,
    Editor,
}
impl Grant {
    pub fn capabilities(self) -> Capabilities {
        Capabilities {
            list: true,
            download: true,
            rename: self == Self::Editor,
            moves: self == Self::Editor,
        }
    }
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Viewer => "viewer",
            Self::Editor => "editor",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Permission {
    List,
    Download,
    Rename,
    Move,
}

/// Access to mounts is evaluated against the application principal, independently of login method.
pub trait AccessStore: Send + Sync {
    fn capabilities(
        &self,
        identity: &Identity,
        mount: &MountName,
    ) -> impl Future<Output = Result<Capabilities, Box<dyn std::error::Error + Send + Sync>>> + Send;
}
