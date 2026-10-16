use authn_identity::prelude::{Identity, User, UserId};
use authn_static::prelude::{PasswordHash, StaticAccount, StaticStore, Username};
use cap_std::fs::Dir;
use std::{
    io::{self, Read},
    sync::Arc,
};

const MAX_HASH_BYTES: u64 = 1024;

/// A capability-scoped directory of Argon2id PHC hashes, keyed by local user ID.
/// Each lookup reopens the file so mounted secret rotations and removals take effect.
#[derive(Debug, Clone)]
pub struct DirectoryStore {
    dir: Arc<Dir>,
}
impl DirectoryStore {
    /// Use an already-open directory capability; opening ambient paths belongs to the caller.
    pub fn new(dir: Dir) -> Self {
        Self { dir: Arc::new(dir) }
    }
}
impl StaticStore for DirectoryStore {
    type Identity = Identity;
    type Error = io::Error;

    async fn lookup(&self, username: &Username) -> io::Result<Option<StaticAccount<Identity>>> {
        let Some(user_id) = directory_user_id(username) else {
            return Ok(None);
        };
        let dir = self.dir.clone();
        tokio::task::spawn_blocking(move || {
            // cap-std resolves relative symlinks within the supplied directory capability.
            // No path canonicalization or preflight metadata checks precede the open.
            let file = match dir.open(user_id.as_str()) {
                Ok(file) => file,
                Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
                Err(error) => return Err(error),
            };
            let mut bytes = Vec::new();
            file.take(MAX_HASH_BYTES + 1).read_to_end(&mut bytes)?;
            if bytes.len() as u64 > MAX_HASH_BYTES {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "password hash file exceeds 1 KiB",
                ));
            }
            let value = std::str::from_utf8(&bytes).map_err(|_| {
                io::Error::new(io::ErrorKind::InvalidData, "password hash must be UTF-8")
            })?;
            // Accept a text file's single line ending, without trimming any hash contents.
            let value = value
                .strip_suffix('\n')
                .map(|v| v.strip_suffix('\r').unwrap_or(v))
                .unwrap_or(value);
            let password_hash = PasswordHash::new(value)
                .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
            Ok(Some(StaticAccount::new(
                password_hash,
                User { id: user_id }.into(),
            )))
        })
        .await
        .map_err(io::Error::other)?
    }
}

/// Parse a request username into a local user ID that is also a single secret filename.
fn directory_user_id(username: &Username) -> Option<UserId> {
    let name = username.as_str();
    if name.len() > 255
        || name == "."
        || name.starts_with("..")
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"-_.".contains(&byte))
    {
        return None;
    }
    UserId::new(name).ok()
}
