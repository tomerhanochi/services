use file_server_core::prelude::*;
use std::{collections::HashMap, io, path::PathBuf};

/// Async path-based mounts. Roots and symlinks must be controlled by trusted code;
/// canonical containment checks do not sandbox concurrent hostile filesystem writers.
pub struct TokioStore {
    order: Vec<MountName>,
    roots: HashMap<MountName, PathBuf>,
}
impl TokioStore {
    #[hotpath::measure(impl_type = "TokioStore", future = true)]
    pub async fn new(
        mounts: impl IntoIterator<Item = (MountName, PathBuf)>,
    ) -> Result<Self, FsError> {
        let mut order = Vec::new();
        let mut roots = HashMap::new();
        for (name, path) in mounts {
            if roots.contains_key(&name) {
                return Err(FsError::Io(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "duplicate mount",
                )));
            }
            let path = tokio::fs::canonicalize(path).await.map_err(fs_error)?;
            order.push(name.clone());
            roots.insert(name, path);
        }
        Ok(Self { order, roots })
    }
    #[hotpath::measure(impl_type = "TokioStore", future = true)]
    async fn resolve(&self, at: &Location) -> Result<PathBuf, FsError> {
        let root = self.roots.get(&at.mount).ok_or(FsError::NotFound)?;
        let path = root.join(relative(&at.path));
        let path = tokio::fs::canonicalize(path).await.map_err(fs_error)?;
        if !path.starts_with(root) {
            return Err(FsError::NotFound);
        }
        Ok(path)
    }
    #[hotpath::measure(impl_type = "TokioStore", future = true)]
    async fn parent(&self, at: &Location) -> Result<PathBuf, FsError> {
        let parent = at.parent().ok_or(FsError::NotFound)?;
        let name = at.path.name().ok_or(FsError::NotFound)?;
        Ok(self.resolve(&parent).await?.join(name.as_str()))
    }
}
fn relative(path: &RelPath) -> PathBuf {
    path.components().iter().map(EntryName::as_str).collect()
}
fn metadata(meta: &std::fs::Metadata) -> Metadata {
    Metadata {
        kind: if meta.is_dir() {
            EntryKind::Directory
        } else {
            EntryKind::File
        },
        size: if meta.is_dir() { 0 } else { meta.len() },
        modified: meta.modified().ok(),
    }
}
fn fs_error(error: io::Error) -> FsError {
    match error.kind() {
        io::ErrorKind::NotFound => FsError::NotFound,
        io::ErrorKind::AlreadyExists | io::ErrorKind::DirectoryNotEmpty => FsError::AlreadyExists,
        io::ErrorKind::NotADirectory => FsError::NotADirectory,
        io::ErrorKind::IsADirectory => FsError::IsADirectory,
        io::ErrorKind::CrossesDevices => FsError::CrossDevice,
        _ => FsError::Io(error),
    }
}
impl FileStore for TokioStore {
    type Content = tokio::fs::File;
    fn mounts(&self) -> &[MountName] {
        &self.order
    }
    #[hotpath::measure(impl_type = "TokioStore", future = true)]
    async fn list(&self, at: &Location) -> Result<Vec<Entry>, FsError> {
        let mut directory = tokio::fs::read_dir(self.resolve(at).await?)
            .await
            .map_err(fs_error)?;
        let mut entries = Vec::new();
        while let Some(entry) = directory.next_entry().await.map_err(fs_error)? {
            let Ok(name) = entry.file_name().into_string() else {
                continue;
            };
            let Ok(name) = EntryName::new(name) else {
                continue;
            };
            match self.metadata(&at.join(name.clone())).await {
                Ok(metadata) => entries.push(Entry { name, metadata }),
                Err(FsError::NotFound) => {}
                Err(error) => return Err(error),
            }
        }
        Ok(entries)
    }
    #[hotpath::measure(impl_type = "TokioStore", future = true)]
    async fn metadata(&self, at: &Location) -> Result<Metadata, FsError> {
        tokio::fs::metadata(self.resolve(at).await?)
            .await
            .map(|meta| metadata(&meta))
            .map_err(fs_error)
    }
    #[hotpath::measure(impl_type = "TokioStore", future = true)]
    async fn open(&self, at: &Location) -> Result<(Metadata, Self::Content), FsError> {
        let file = tokio::fs::File::open(self.resolve(at).await?)
            .await
            .map_err(fs_error)?;
        let meta = file.metadata().await.map_err(fs_error)?;
        if meta.is_dir() {
            return Err(FsError::IsADirectory);
        }
        if !meta.is_file() {
            return Err(FsError::NotFound);
        }
        Ok((metadata(&meta), file))
    }
    #[hotpath::measure(impl_type = "TokioStore", future = true)]
    async fn rename(&self, from: &Location, to: &Location) -> Result<(), FsError> {
        if from.mount != to.mount {
            return Err(FsError::CrossDevice);
        }
        self.resolve(from).await?;
        let (from, to) = (self.parent(from).await?, self.parent(to).await?);
        rename_no_replace(from, to).await
    }
}
#[cfg(unix)]
async fn rename_no_replace(from: PathBuf, to: PathBuf) -> Result<(), FsError> {
    tokio::task::spawn_blocking(move || {
        rustix::fs::renameat_with(
            rustix::fs::CWD,
            from,
            rustix::fs::CWD,
            to,
            rustix::fs::RenameFlags::NOREPLACE,
        )
        .map_err(|error| fs_error(error.into()))
    })
    .await
    .map_err(|error| FsError::Io(io::Error::other(error)))?
}
#[cfg(not(unix))]
async fn rename_no_replace(_from: PathBuf, _to: PathBuf) -> Result<(), FsError> {
    Err(FsError::Io(io::Error::new(
        io::ErrorKind::Unsupported,
        "atomic no-replace rename unsupported on this platform",
    )))
}
