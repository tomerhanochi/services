//! Browsing, downloading, renaming and moving, as a given user.

use std::error::Error;

use authn_identity::prelude::Identity;
use thiserror::Error;

use crate::access::{AccessStore, Permission};
use crate::model::{Entry, EntryKind, EntryName, Location, Metadata, MountName};
use crate::store::{FileStore, FsError};

/// Why an operation failed, in terms a user can act on.
#[derive(Debug, Error)]
pub enum FilesError {
    /// Also returned for mounts the user can't list, so their existence isn't revealed.
    #[error("not found")]
    NotFound,
    #[error("you can't do that here")]
    Forbidden,
    #[error("something named that already exists there")]
    AlreadyExists,
    #[error("that's not a folder")]
    NotADirectory,
    #[error("folders can't be downloaded")]
    IsADirectory,
    #[error("a folder can't be moved into itself")]
    IntoItself,
    #[error("the root of a mount can't be renamed or moved")]
    Root,
    #[error("can't move between storage devices")]
    CrossDevice,
    #[error("internal error")]
    Internal(#[source] Box<dyn Error + Send + Sync>),
}

impl From<FsError> for FilesError {
    fn from(error: FsError) -> Self {
        match error {
            FsError::NotFound => FilesError::NotFound,
            FsError::AlreadyExists => FilesError::AlreadyExists,
            FsError::NotADirectory => FilesError::NotADirectory,
            FsError::IsADirectory => FilesError::IsADirectory,
            FsError::CrossDevice => FilesError::CrossDevice,
            FsError::Io(e) => FilesError::Internal(e.into()),
        }
    }
}

/// What the user may do in one mount.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Capabilities {
    pub list: bool,
    pub download: bool,
    pub rename: bool,
    pub moves: bool,
}

impl Capabilities {
    pub fn allows(self, permission: Permission) -> bool {
        match permission {
            Permission::List => self.list,
            Permission::Download => self.download,
            Permission::Rename => self.rename,
            Permission::Move => self.moves,
        }
    }
}

/// A listed directory.
#[derive(Debug)]
pub struct Directory {
    /// In [`Entry::listing_order`].
    pub entries: Vec<Entry>,
    pub capabilities: Capabilities,
}

/// The file operations, each checked against the user's permissions on the mount.
pub struct FileService<S, R> {
    store: S,
    access: R,
}

impl<S: FileStore, R: AccessStore> FileService<S, R> {
    pub fn new(store: S, access: R) -> Self {
        FileService { store, access }
    }

    /// What `user` may do in `mount`.
    #[hotpath::measure(impl_type = "FileService", future = true)]
    pub async fn capabilities(
        &self,
        user: &Identity,
        mount: &MountName,
    ) -> Result<Capabilities, FilesError> {
        if !self.store.mounts().contains(mount) {
            return Ok(Capabilities::default());
        }
        self.access
            .capabilities(user, mount)
            .await
            .map_err(FilesError::Internal)
    }

    /// The mounts `user` may list, in configured order.
    #[hotpath::measure(impl_type = "FileService", future = true)]
    pub async fn mounts(&self, user: &Identity) -> Result<Vec<MountName>, FilesError> {
        let mut visible = Vec::new();
        for mount in self.store.mounts() {
            if self.capabilities(user, mount).await?.list {
                visible.push(mount.clone());
            }
        }
        Ok(visible)
    }

    #[hotpath::measure(impl_type = "FileService", future = true)]
    pub async fn list(&self, user: &Identity, dir: &Location) -> Result<Directory, FilesError> {
        let capabilities = self.require(user, &dir.mount, Permission::List).await?;
        let mut entries = self.store.list(dir).await?;
        entries.sort_by(Entry::listing_order);
        Ok(Directory {
            entries,
            capabilities,
        })
    }

    /// One entry's metadata, e.g. for a rename form.
    #[hotpath::measure(impl_type = "FileService", future = true)]
    pub async fn metadata(&self, user: &Identity, at: &Location) -> Result<Metadata, FilesError> {
        self.require(user, &at.mount, Permission::List).await?;
        Ok(self.store.metadata(at).await?)
    }

    #[hotpath::measure(impl_type = "FileService", future = true)]
    pub async fn download(
        &self,
        user: &Identity,
        file: &Location,
    ) -> Result<(Metadata, S::Content), FilesError> {
        self.require(user, &file.mount, Permission::Download)
            .await?;
        Ok(self.store.open(file).await?)
    }

    /// Renames `at` within its directory, returning its new location.
    #[hotpath::measure(impl_type = "FileService", future = true)]
    pub async fn rename(
        &self,
        user: &Identity,
        at: &Location,
        name: EntryName,
    ) -> Result<Location, FilesError> {
        self.require(user, &at.mount, Permission::Rename).await?;
        let parent = at.parent().ok_or(FilesError::Root)?;
        let to = parent.join(name);
        if to != *at {
            self.store.rename(at, &to).await?;
        }
        Ok(to)
    }

    /// Moves `at` into the directory `into`, in the same mount, returning its new location.
    #[hotpath::measure(impl_type = "FileService", future = true)]
    pub async fn move_into(
        &self,
        user: &Identity,
        at: &Location,
        into: &Location,
    ) -> Result<Location, FilesError> {
        if at.mount != into.mount {
            // Mounts are separate trees; a cross-mount move would be a copy.
            return Err(FilesError::Forbidden);
        }
        self.require(user, &at.mount, Permission::Move).await?;
        let name = at.path.name().ok_or(FilesError::Root)?.clone();
        if into.path.starts_with(&at.path) {
            return Err(FilesError::IntoItself);
        }
        if self.store.metadata(into).await?.kind != EntryKind::Directory {
            return Err(FilesError::NotADirectory);
        }
        let to = into.join(name);
        if to != *at {
            self.store.rename(at, &to).await?;
        }
        Ok(to)
    }

    /// Fails with [`FilesError::NotFound`] if the user can't even list the mount, and
    /// [`FilesError::Forbidden`] if they can but lack `permission`.
    #[hotpath::measure(impl_type = "FileService", future = true)]
    async fn require(
        &self,
        user: &Identity,
        mount: &MountName,
        permission: Permission,
    ) -> Result<Capabilities, FilesError> {
        let capabilities = self.capabilities(user, mount).await?;
        match (capabilities.list, capabilities.allows(permission)) {
            (false, _) => Err(FilesError::NotFound),
            (true, false) => Err(FilesError::Forbidden),
            (true, true) => Ok(capabilities),
        }
    }
}
