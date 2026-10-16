use authn::prelude::VerifyCredentials;
use authn_identity::prelude::*;
use authn_static::prelude::*;
use authn_static_directory::prelude::*;
use std::{io, sync::Arc};

fn credentials(value: &str) -> BasicCredentials {
    BasicCredentials::parse(value).unwrap()
}

#[tokio::test]
async fn directory_accounts_follow_password_changes_and_removals() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("alice");
    std::fs::write(&path, encoded_hash("first:password")).unwrap();
    std::fs::write(directory.path().join("bob"), encoded_hash("second")).unwrap();
    let verifier = StaticVerifier::new(Arc::new(DirectoryStore::new(directory.path())));
    let expected = Identity::User(User {
        id: UserId::new("alice").unwrap(),
    });
    assert_eq!(
        verifier
            .verify(&credentials("alice:first:password"))
            .await
            .unwrap(),
        Some(expected.clone())
    );
    assert!(
        verifier
            .verify(&credentials("alice:wrong"))
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        verifier
            .verify(&credentials("unknown:first:password"))
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        verifier
            .verify(&credentials("bob:second"))
            .await
            .unwrap()
            .is_some()
    );
    std::fs::write(&path, encoded_hash("new password ")).unwrap();
    assert!(
        verifier
            .verify(&credentials("alice:first:password"))
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        verifier
            .verify(&credentials("alice:new password"))
            .await
            .unwrap()
            .is_none()
    );
    assert_eq!(
        verifier
            .verify(&credentials("alice:new password "))
            .await
            .unwrap(),
        Some(expected)
    );
    std::fs::remove_file(&path).unwrap();
    assert!(
        verifier
            .verify(&credentials("alice:new password "))
            .await
            .unwrap()
            .is_none()
    );
}

#[tokio::test]
async fn invalid_hash_files_and_read_failures_are_operational_errors() {
    let directory = tempfile::tempdir().unwrap();
    let store = DirectoryStore::new(directory.path());
    let username = Username::new("alice").unwrap();
    for contents in [
        vec![],
        b"plaintext-password".to_vec(),
        vec![255],
        vec![b'x'; 1025],
    ] {
        std::fs::write(directory.path().join("alice"), contents).unwrap();
        assert_eq!(
            store.lookup(&username).await.unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
    }
    std::fs::create_dir(directory.path().join("bob")).unwrap();
    assert!(store.lookup(&Username::new("bob").unwrap()).await.is_err());
}

#[tokio::test]
async fn directory_lookups_use_the_current_configured_path() {
    let parent = tempfile::tempdir().unwrap();
    let original = parent.path().join("accounts");
    let moved = parent.path().join("moved-accounts");
    std::fs::create_dir(&original).unwrap();
    std::fs::write(original.join("alice"), encoded_hash("original")).unwrap();
    let verifier = StaticVerifier::new(DirectoryStore::new(&original));
    std::fs::rename(&original, &moved).unwrap();
    std::fs::create_dir(&original).unwrap();
    std::fs::write(original.join("alice"), encoded_hash("replacement")).unwrap();
    assert!(
        verifier
            .verify(&credentials("alice:original"))
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        verifier
            .verify(&credentials("alice:replacement"))
            .await
            .unwrap()
            .is_some()
    );
}

#[tokio::test]
async fn hash_files_accept_a_single_text_line_ending() {
    let directory = tempfile::tempdir().unwrap();
    let store = DirectoryStore::new(directory.path());
    let hash = encoded_hash("password");
    for suffix in ["", "\n", "\r\n"] {
        std::fs::write(directory.path().join("alice"), format!("{hash}{suffix}")).unwrap();
        assert!(
            store
                .lookup(&Username::new("alice").unwrap())
                .await
                .unwrap()
                .is_some()
        );
    }
    std::fs::write(directory.path().join("alice"), format!("{hash}\n\n")).unwrap();
    assert!(
        store
            .lookup(&Username::new("alice").unwrap())
            .await
            .is_err()
    );
}

#[tokio::test]
async fn requests_cannot_select_paths_or_kubernetes_metadata() {
    let directory = tempfile::tempdir().unwrap();
    let verifier = StaticVerifier::new(DirectoryStore::new(directory.path()));
    for username in [
        "../alice",
        "/etc/passwd",
        "sub/alice",
        "sub\\alice",
        ".",
        "..",
        "..data",
        "..2026_01",
    ] {
        let credentials = BasicCredentials::new(
            Username::new(username).unwrap(),
            Password::new("password").unwrap(),
        );
        assert!(
            verifier.verify(&credentials).await.unwrap().is_none(),
            "{username}"
        );
    }
}

#[cfg(unix)]
#[tokio::test]
async fn kubernetes_symlink_rotation_is_read_on_each_lookup() {
    use std::os::unix::fs::symlink;
    let directory = tempfile::tempdir().unwrap();
    for (revision, password) in [("..first", "old"), ("..second", "new")] {
        std::fs::create_dir(directory.path().join(revision)).unwrap();
        std::fs::write(
            directory.path().join(revision).join("alice"),
            encoded_hash(password),
        )
        .unwrap();
    }
    symlink("..first", directory.path().join("..data")).unwrap();
    symlink("..data/alice", directory.path().join("alice")).unwrap();
    let verifier = StaticVerifier::new(DirectoryStore::new(directory.path()));
    assert!(
        verifier
            .verify(&credentials("alice:old"))
            .await
            .unwrap()
            .is_some()
    );
    symlink("..second", directory.path().join("..data_tmp")).unwrap();
    std::fs::rename(
        directory.path().join("..data_tmp"),
        directory.path().join("..data"),
    )
    .unwrap();
    assert!(
        verifier
            .verify(&credentials("alice:old"))
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        verifier
            .verify(&credentials("alice:new"))
            .await
            .unwrap()
            .is_some()
    );
}

fn encoded_hash(password: &str) -> String {
    use argon2::{Argon2, PasswordHasher, password_hash::SaltString};
    // Deterministic test salt; real credentials must use a fresh random salt per hash.
    let salt = SaltString::encode_b64(b"test-salt-16bytes").unwrap();
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .unwrap()
        .to_string()
}
