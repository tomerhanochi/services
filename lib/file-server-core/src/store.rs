//! The file store port.

use std::io;

use thiserror::Error;
use tokio::io::AsyncRead;

use crate::model::{Entry, Location, Metadata, MountName};

#[derive(Debug, Error)]
pub enum FsError {
    #[error("no such file or directory")]
    NotFound,
    #[error("something with that name already exists")]
    AlreadyExists,
    #[error("not a directory")]
    NotADirectory,
    #[error("is a directory")]
    IsADirectory,
    /// The source and destination are on different filesystems inside one mount.
    #[error("can't move across filesystems")]
    CrossDevice,
    #[error(transparent)]
    Io(io::Error),
}

/// Where the files are. Paths are confined to their mount: an implementation must not
/// follow anything (`..`, symlinks) out of a mount's root.
pub trait FileStore: Send + Sync {
    /// A file's bytes.
    type Content: AsyncRead + Send + Unpin + 'static;

    /// The configured mounts, in display order.
    fn mounts(&self) -> &[MountName];

    /// A directory's entries, in no particular order.
    fn list(&self, dir: &Location) -> impl Future<Output = Result<Vec<Entry>, FsError>> + Send;

    fn metadata(&self, at: &Location) -> impl Future<Output = Result<Metadata, FsError>> + Send;

    /// Opens a file for reading.
    fn open(
        &self,
        file: &Location,
    ) -> impl Future<Output = Result<(Metadata, Self::Content), FsError>> + Send;

    /// Moves `from` to `to`, both in the same mount. Fails with
    /// [`FsError::AlreadyExists`] rather than replacing anything, atomically: there is no
    /// window in which another writer's file could be overwritten.
    fn rename(
        &self,
        from: &Location,
        to: &Location,
    ) -> impl Future<Output = Result<(), FsError>> + Send;
}
