use authn_identity::prelude::*;
use file_server_core::prelude::*;
use std::{
    collections::BTreeMap,
    io::Cursor,
    sync::{Arc, Mutex},
};
use tokio::io::AsyncReadExt;

/// Paths (`mount/a/b`) to whether they're directories.
#[derive(Clone, Default)]
struct FakeStore {
    mounts: Vec<MountName>,
    entries: Arc<Mutex<BTreeMap<String, EntryKind>>>,
}

fn key(at: &Location) -> String {
    format!("{}/{}", at.mount, at.path)
        .trim_end_matches('/')
        .to_owned()
}

impl FakeStore {
    fn new(paths: &[&str]) -> Self {
        let mut entries = BTreeMap::new();
        let mut mounts = Vec::new();
        for path in paths {
            let kind = if path.ends_with('/') {
                EntryKind::Directory
            } else {
                EntryKind::File
            };
            entries.insert(path.trim_end_matches('/').to_owned(), kind);
            let mount = MountName::new(path.split('/').next().unwrap()).unwrap();
            if !mounts.contains(&mount) {
                mounts.push(mount);
            }
        }
        for mount in &mounts {
            entries.insert(mount.to_string(), EntryKind::Directory);
        }
        FakeStore {
            mounts,
            entries: Arc::new(Mutex::new(entries)),
        }
    }

    fn paths(&self) -> Vec<String> {
        self.entries.lock().unwrap().keys().cloned().collect()
    }
}

fn meta(kind: EntryKind) -> Metadata {
    Metadata {
        kind,
        size: 3,
        modified: None,
    }
}

impl FileStore for FakeStore {
    type Content = Cursor<Vec<u8>>;

    fn mounts(&self) -> &[MountName] {
        &self.mounts
    }

    async fn list(&self, dir: &Location) -> Result<Vec<Entry>, FsError> {
        let entries = self.entries.lock().unwrap();
        let dir = key(dir);
        match entries.get(&dir) {
            None => return Err(FsError::NotFound),
            Some(EntryKind::File) => return Err(FsError::NotADirectory),
            Some(EntryKind::Directory) => {}
        }
        let prefix = format!("{dir}/");
        Ok(entries
            .iter()
            .filter_map(|(path, &kind)| {
                let name = path.strip_prefix(&prefix)?;
                (!name.contains('/')).then(|| Entry {
                    name: EntryName::new(name).unwrap(),
                    metadata: meta(kind),
                })
            })
            .collect())
    }

    async fn metadata(&self, at: &Location) -> Result<Metadata, FsError> {
        self.entries
            .lock()
            .unwrap()
            .get(&key(at))
            .map(|&k| meta(k))
            .ok_or(FsError::NotFound)
    }

    async fn open(&self, file: &Location) -> Result<(Metadata, Self::Content), FsError> {
        match self.entries.lock().unwrap().get(&key(file)) {
            None => Err(FsError::NotFound),
            Some(EntryKind::Directory) => Err(FsError::IsADirectory),
            Some(EntryKind::File) => Ok((meta(EntryKind::File), Cursor::new(b"abc".to_vec()))),
        }
    }

    async fn rename(&self, from: &Location, to: &Location) -> Result<(), FsError> {
        let mut entries = self.entries.lock().unwrap();
        let (from, to) = (key(from), key(to));
        if entries.contains_key(&to) {
            return Err(FsError::AlreadyExists);
        }
        let moved: Vec<_> = entries
            .keys()
            .filter(|k| **k == from || k.starts_with(&format!("{from}/")))
            .cloned()
            .collect();
        if moved.is_empty() {
            return Err(FsError::NotFound);
        }
        for old in moved {
            let kind = entries.remove(&old).unwrap();
            entries.insert(format!("{to}{}", &old[from.len()..]), kind);
        }
        Ok(())
    }
}

fn mount(name: &str) -> MountName {
    MountName::new(name).unwrap()
}

fn at(mount_name: &str, path: &str) -> Location {
    Location::new(mount(mount_name), RelPath::parse(path).unwrap())
}

/// alice edits photos; bob views photos; nobody sees `secret`.
struct Access;
impl AccessStore for Access {
    async fn capabilities(
        &self,
        identity: &Identity,
        mount: &MountName,
    ) -> Result<Capabilities, Box<dyn std::error::Error + Send + Sync>> {
        let Identity::User(user) = identity else {
            return Ok(Capabilities::default());
        };
        Ok(match (mount.as_str(), user.id.as_str()) {
            ("photos", "alice") => Grant::Editor.capabilities(),
            ("photos", "bob") => Grant::Viewer.capabilities(),
            _ => Capabilities::default(),
        })
    }
}
fn user(id: &str) -> Identity {
    User {
        id: UserId::new(id).unwrap(),
    }
    .into()
}
fn service(store: FakeStore) -> FileService<FakeStore, Access> {
    FileService::new(store, Access)
}

fn store() -> FakeStore {
    FakeStore::new(&[
        "photos/2024/",
        "photos/2024/b.jpg",
        "photos/2024/a10.jpg",
        "photos/2024/a2.jpg",
        "photos/old/",
        "secret/x",
    ])
}

#[tokio::test]
async fn lists_in_order_with_capabilities() {
    let files = service(store());
    let dir = files
        .list(&user("bob"), &at("photos", "2024"))
        .await
        .unwrap();
    let names: Vec<_> = dir.entries.iter().map(|e| e.name.to_string()).collect();
    assert_eq!(names, ["a2.jpg", "a10.jpg", "b.jpg"]);
    assert!(dir.capabilities.download && !dir.capabilities.rename);

    let root = files.list(&user("alice"), &at("photos", "")).await.unwrap();
    assert_eq!(root.entries[0].metadata.kind, EntryKind::Directory);
    assert!(root.capabilities.rename && root.capabilities.moves);
}

#[tokio::test]
async fn hidden_mounts_are_not_found() {
    let files = service(store());
    assert_eq!(
        files.mounts(&user("alice")).await.unwrap(),
        [mount("photos")]
    );
    assert!(matches!(
        files.list(&user("alice"), &at("secret", "")).await,
        Err(FilesError::NotFound)
    ));
    assert!(matches!(
        files.list(&user("mallory"), &at("photos", "")).await,
        Err(FilesError::NotFound)
    ));
    assert!(matches!(
        files.list(&user("alice"), &at("nope", "")).await,
        Err(FilesError::NotFound)
    ));
}

#[tokio::test]
async fn viewers_can_download_but_not_rename() {
    let files = service(store());
    let bob = user("bob");
    let (_, mut content) = files
        .download(&bob, &at("photos", "2024/b.jpg"))
        .await
        .unwrap();
    let mut bytes = Vec::new();
    content.read_to_end(&mut bytes).await.unwrap();
    assert_eq!(bytes, b"abc");
    assert!(matches!(
        files.download(&bob, &at("photos", "2024")).await,
        Err(FilesError::IsADirectory)
    ));
    let name = EntryName::new("c.jpg").unwrap();
    assert!(matches!(
        files.rename(&bob, &at("photos", "2024/b.jpg"), name).await,
        Err(FilesError::Forbidden)
    ));
}

#[tokio::test]
async fn rename_and_move() {
    let store = store();
    let files = service(store.clone());
    let alice = user("alice");

    let renamed = files
        .rename(
            &alice,
            &at("photos", "2024/b.jpg"),
            EntryName::new("c.jpg").unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(renamed, at("photos", "2024/c.jpg"));
    let clash = files
        .rename(&alice, &renamed, EntryName::new("a2.jpg").unwrap())
        .await;
    assert!(matches!(clash, Err(FilesError::AlreadyExists)));

    let moved = files
        .move_into(&alice, &at("photos", "2024"), &at("photos", "old"))
        .await
        .unwrap();
    assert_eq!(moved, at("photos", "old/2024"));
    assert!(store.paths().contains(&"photos/old/2024/c.jpg".to_owned()));

    let into_itself = files
        .move_into(&alice, &at("photos", "old"), &at("photos", "old/2024"))
        .await;
    assert!(matches!(into_itself, Err(FilesError::IntoItself)));
    let into_file = files
        .move_into(
            &alice,
            &at("photos", "old/2024/c.jpg"),
            &at("photos", "old/2024/a2.jpg"),
        )
        .await;
    assert!(matches!(into_file, Err(FilesError::NotADirectory)));
    let root = files
        .rename(&alice, &at("photos", ""), EntryName::new("x").unwrap())
        .await;
    assert!(matches!(root, Err(FilesError::Root)));
    let across = files
        .move_into(&alice, &at("photos", "old"), &at("secret", ""))
        .await;
    assert!(matches!(across, Err(FilesError::Forbidden)));
}
