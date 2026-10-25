//! Validated names and paths.

use std::cmp::Ordering;
use std::fmt;
use std::time::SystemTime;

use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum InvalidName {
    #[error("a name can't be empty")]
    Empty,
    #[error("a name can't be longer than {0} bytes")]
    TooLong(usize),
    #[error("a name can't contain `/` or NUL")]
    Separator,
    #[error("`.` and `..` aren't names")]
    Dots,
    #[error("a mount name may only contain a-z, 0-9, `-` and `_`")]
    MountCharacters,
}

/// A configured root directory's name, as it appears in URLs: `[a-z0-9_-]{1,64}`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Deserialize)]
#[serde(try_from = "String")]
pub struct MountName(String);

impl MountName {
    pub fn new(name: impl Into<String>) -> Result<Self, InvalidName> {
        let name = name.into();
        if name.is_empty() {
            return Err(InvalidName::Empty);
        }
        if name.len() > 64 {
            return Err(InvalidName::TooLong(64));
        }
        if !name
            .bytes()
            .all(|b| matches!(b, b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_'))
        {
            return Err(InvalidName::MountCharacters);
        }
        Ok(MountName(name))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for MountName {
    type Error = InvalidName;
    fn try_from(name: String) -> Result<Self, InvalidName> {
        MountName::new(name)
    }
}

impl fmt::Display for MountName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// One path component: a file or directory name. Never empty, `.`, `..`, or containing
/// `/` or NUL, so joining it to a path can't climb out of the path.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct EntryName(String);

impl EntryName {
    /// The longest name common filesystems accept, in bytes.
    pub const MAX_LEN: usize = 255;

    pub fn new(name: impl Into<String>) -> Result<Self, InvalidName> {
        let name = name.into();
        match name.as_str() {
            "" => Err(InvalidName::Empty),
            "." | ".." => Err(InvalidName::Dots),
            n if n.len() > Self::MAX_LEN => Err(InvalidName::TooLong(Self::MAX_LEN)),
            n if n.contains(['/', '\\', '\0']) => Err(InvalidName::Separator),
            _ => Ok(EntryName(name)),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for EntryName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A path inside a mount: zero or more [`EntryName`]s. The empty path is the mount's root.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct RelPath(Vec<EntryName>);

impl RelPath {
    pub fn root() -> Self {
        RelPath(Vec::new())
    }

    /// Parses `a/b/c`. Leading and trailing slashes are ignored; empty components (`a//b`),
    /// `.` and `..` are errors.
    pub fn parse(path: &str) -> Result<Self, InvalidName> {
        let path = path.trim_matches('/');
        if path.is_empty() {
            return Ok(RelPath::root());
        }
        path.split('/')
            .map(EntryName::new)
            .collect::<Result<_, _>>()
            .map(RelPath)
    }

    pub fn is_root(&self) -> bool {
        self.0.is_empty()
    }

    pub fn components(&self) -> &[EntryName] {
        &self.0
    }

    /// The last component; `None` for the root.
    pub fn name(&self) -> Option<&EntryName> {
        self.0.last()
    }

    /// `None` for the root.
    pub fn parent(&self) -> Option<RelPath> {
        let (_, parent) = self.0.split_last()?;
        Some(RelPath(parent.to_vec()))
    }

    pub fn join(&self, name: EntryName) -> RelPath {
        let mut components = self.0.clone();
        components.push(name);
        RelPath(components)
    }

    /// Whether `self` is `ancestor` or inside it.
    pub fn starts_with(&self, ancestor: &RelPath) -> bool {
        self.0.starts_with(&ancestor.0)
    }
}

/// `a/b/c`, empty for the root.
impl fmt::Display for RelPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, name) in self.0.iter().enumerate() {
            if i > 0 {
                f.write_str("/")?;
            }
            f.write_str(name.as_str())?;
        }
        Ok(())
    }
}

/// A file or directory: which mount, and where in it.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Location {
    pub mount: MountName,
    pub path: RelPath,
}

impl Location {
    pub fn new(mount: MountName, path: RelPath) -> Self {
        Location { mount, path }
    }

    pub fn root(mount: MountName) -> Self {
        Location {
            mount,
            path: RelPath::root(),
        }
    }

    /// `None` for a mount's root.
    pub fn parent(&self) -> Option<Location> {
        Some(Location {
            mount: self.mount.clone(),
            path: self.path.parent()?,
        })
    }

    pub fn join(&self, name: EntryName) -> Location {
        Location {
            mount: self.mount.clone(),
            path: self.path.join(name),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EntryKind {
    Directory,
    File,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Metadata {
    pub kind: EntryKind,
    /// Bytes, for files. 0 for directories.
    pub size: u64,
    pub modified: Option<SystemTime>,
}

/// A directory listing's row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub name: EntryName,
    pub metadata: Metadata,
}

impl Entry {
    /// Directories first, then by name, comparing digit runs as numbers (`2` < `10`) and
    /// letters case-insensitively.
    pub fn listing_order(a: &Entry, b: &Entry) -> Ordering {
        let dir = |e: &Entry| e.metadata.kind != EntryKind::Directory;
        dir(a)
            .cmp(&dir(b))
            .then_with(|| natural_cmp(a.name.as_str(), b.name.as_str()))
    }
}

/// Compares runs of ASCII digits by value and everything else by lower-cased character,
/// falling back to plain byte order for names equal under those rules.
pub fn natural_cmp(a: &str, b: &str) -> Ordering {
    let (mut x, mut y) = (a.chars().peekable(), b.chars().peekable());
    loop {
        match (x.peek().copied(), y.peek().copied()) {
            (None, None) => return a.cmp(b),
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(c), Some(d)) if c.is_ascii_digit() && d.is_ascii_digit() => {
                let run = |it: &mut std::iter::Peekable<std::str::Chars<'_>>| {
                    let mut digits = String::new();
                    while let Some(c) = it.next_if(char::is_ascii_digit) {
                        digits.push(c);
                    }
                    digits
                };
                let (m, n) = (run(&mut x), run(&mut y));
                let (m, n) = (m.trim_start_matches('0'), n.trim_start_matches('0'));
                let order = m.len().cmp(&n.len()).then_with(|| m.cmp(n));
                if order != Ordering::Equal {
                    return order;
                }
            }
            (Some(c), Some(d)) => {
                let order = c.to_lowercase().cmp(d.to_lowercase());
                if order != Ordering::Equal {
                    return order;
                }
                x.next();
                y.next();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entry_names() {
        assert!(EntryName::new("photo.jpg").is_ok());
        assert!(EntryName::new(".hidden").is_ok());
        assert_eq!(EntryName::new(""), Err(InvalidName::Empty));
        assert_eq!(EntryName::new(".."), Err(InvalidName::Dots));
        assert_eq!(EntryName::new("a/b"), Err(InvalidName::Separator));
        assert_eq!(EntryName::new("a\0"), Err(InvalidName::Separator));
        assert_eq!(
            EntryName::new("x".repeat(256)),
            Err(InvalidName::TooLong(255))
        );
    }

    #[test]
    fn paths() {
        let p = RelPath::parse("/a/b/").unwrap();
        assert_eq!(p.to_string(), "a/b");
        assert_eq!(p.parent().unwrap().to_string(), "a");
        assert!(p.starts_with(&RelPath::parse("a").unwrap()));
        assert!(
            !RelPath::parse("ab")
                .unwrap()
                .starts_with(&RelPath::parse("a").unwrap())
        );
        assert!(RelPath::parse("").unwrap().is_root());
        assert!(RelPath::parse("a/../b").is_err());
        assert!(RelPath::parse("a//b").is_err());
    }

    #[test]
    fn mount_names() {
        assert!(MountName::new("photos-2024").is_ok());
        assert_eq!(MountName::new("Photos"), Err(InvalidName::MountCharacters));
    }

    #[test]
    fn natural_order() {
        let mut names = ["file10", "File2", "file1", "a", "file02"];
        names.sort_by(|a, b| natural_cmp(a, b));
        assert_eq!(names, ["a", "file1", "File2", "file02", "file10"]);
    }
}
