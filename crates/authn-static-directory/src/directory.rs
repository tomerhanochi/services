use authn_identity::prelude::{Identity, User, UserId};
use authn_static::prelude::{PasswordHash, StaticAccount, StaticStore, Username};
use std::{io, path::PathBuf};
use tokio::io::AsyncReadExt;

const MAX_HASH_BYTES: u64 = 1024;

/// A trusted mounted directory of Argon2id PHC hashes, keyed by local user ID.
/// File operations use Tokio. The application controls the directory and its symlinks.
#[derive(Debug, Clone)]
pub struct DirectoryStore {
    directory: PathBuf,
}
impl DirectoryStore {
    pub fn new(directory: impl Into<PathBuf>) -> Self {
        Self {
            directory: directory.into(),
        }
    }
}
impl StaticStore for DirectoryStore {
    type Identity = Identity;
    type Error = io::Error;
    #[hotpath::measure(impl_type = "DirectoryStore", future = true)]
    async fn lookup(&self, username: &Username) -> io::Result<Option<StaticAccount<Identity>>> {
        let Some(user_id) = directory_user_id(username) else {
            return Ok(None);
        };
        let file = match tokio::fs::File::open(self.directory.join(user_id.as_str())).await {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error),
        };
        let mut bytes = Vec::new();
        hotpath::io!(file, label = "password-hash-file")
            .take(MAX_HASH_BYTES + 1)
            .read_to_end(&mut bytes)
            .await?;
        if bytes.len() as u64 > MAX_HASH_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "password hash file exceeds 1 KiB",
            ));
        }
        let value = std::str::from_utf8(&bytes).map_err(|_| {
            io::Error::new(io::ErrorKind::InvalidData, "password hash must be UTF-8")
        })?;
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
