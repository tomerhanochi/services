use std::fs;
use std::os::unix::fs::symlink;

use file_server_core::prelude::{EntryKind, FileStore, FsError, Location, MountName, RelPath};
use file_server_fs::prelude::TokioStore;
use tokio::io::AsyncReadExt;

/// A mount at `root/mount`, with a secret next to it at `root/secret.txt`.
async fn setup() -> (tempfile::TempDir, TokioStore) {
    let tmp = tempfile::tempdir().unwrap();
    let mount = tmp.path().join("mount");
    fs::create_dir_all(mount.join("docs/inner")).unwrap();
    fs::write(mount.join("docs/a.txt"), "hello").unwrap();
    fs::write(mount.join("b.txt"), "bee").unwrap();
    fs::write(tmp.path().join("secret.txt"), "secret").unwrap();
    symlink(tmp.path().join("secret.txt"), mount.join("escape")).unwrap();
    symlink("../secret.txt", mount.join("escape-relative")).unwrap();
    symlink("docs/a.txt", mount.join("inside")).unwrap();
    let store = TokioStore::new([(MountName::new("m").unwrap(), mount)])
        .await
        .unwrap();
    (tmp, store)
}

fn at(path: &str) -> Location {
    Location::new(MountName::new("m").unwrap(), RelPath::parse(path).unwrap())
}

#[tokio::test]
async fn lists_and_hides_escaping_symlinks() {
    let (_tmp, store) = setup().await;
    let mut names: Vec<_> = store
        .list(&at(""))
        .await
        .unwrap()
        .into_iter()
        .map(|e| e.name.to_string())
        .collect();
    names.sort();
    assert_eq!(names, ["b.txt", "docs", "inside"]);

    let docs = store.list(&at("docs")).await.unwrap();
    assert_eq!(docs.len(), 2);
    let a = docs.iter().find(|e| e.name.as_str() == "a.txt").unwrap();
    assert_eq!((a.metadata.kind, a.metadata.size), (EntryKind::File, 5));
    assert!(matches!(
        store.list(&at("b.txt")).await,
        Err(FsError::NotADirectory)
    ));
    assert!(matches!(
        store.list(&at("nope")).await,
        Err(FsError::NotFound)
    ));
}

#[tokio::test]
async fn opens_files_but_not_escapes_or_directories() {
    let (_tmp, store) = setup().await;
    let (meta, mut file) = store.open(&at("inside")).await.unwrap();
    assert_eq!(meta.size, 5);
    let mut text = String::new();
    file.read_to_string(&mut text).await.unwrap();
    assert_eq!(text, "hello");

    assert!(store.open(&at("escape")).await.is_err());
    assert!(store.open(&at("escape-relative")).await.is_err());
    assert!(matches!(
        store.open(&at("docs")).await,
        Err(FsError::IsADirectory)
    ));
}

#[tokio::test]
async fn renames_without_replacing() {
    let (tmp, store) = setup().await;
    store.rename(&at("b.txt"), &at("c.txt")).await.unwrap();
    assert!(tmp.path().join("mount/c.txt").exists());

    let clash = store.rename(&at("c.txt"), &at("docs")).await;
    assert!(matches!(clash, Err(FsError::AlreadyExists)), "{clash:?}");
    assert_eq!(
        fs::read_to_string(tmp.path().join("mount/c.txt")).unwrap(),
        "bee"
    );

    store
        .rename(&at("c.txt"), &at("docs/inner/c.txt"))
        .await
        .unwrap();
    assert!(tmp.path().join("mount/docs/inner/c.txt").exists());
    store.rename(&at("docs/inner"), &at("inner")).await.unwrap();
    assert!(tmp.path().join("mount/inner/c.txt").exists());

    assert!(matches!(
        store.rename(&at("missing"), &at("x")).await,
        Err(FsError::NotFound)
    ));
}

#[tokio::test]
async fn unknown_mount_is_not_found() {
    let (_tmp, store) = setup().await;
    let other = Location::root(MountName::new("other").unwrap());
    assert!(matches!(store.list(&other).await, Err(FsError::NotFound)));
}
