use authn_identity::prelude::{MachineId, UserId};
use figment::{
    Figment,
    providers::{Env, Format, Toml},
};
use file_server_authn::prelude::AuthConfig;
use file_server_core::prelude::{Grant, MountName};
use file_server_postgres::prelude::DatabaseConfig;
use serde::Deserialize;
use std::{
    collections::{BTreeMap, HashSet},
    net::SocketAddr,
    path::{Path, PathBuf},
};

/// Parsed application config, containing deployment values only.
#[derive(Debug, Deserialize)]
#[serde(try_from = "RawConfig")]
pub struct Config {
    pub listen: SocketAddr,
    pub public_origin: String,
    pub database: DatabaseConfig,
    pub auth: AuthConfig,
    pub mounts: Vec<Mount>,
    pub roles: BTreeMap<String, BTreeMap<MountName, Grant>>,
    pub users: BTreeMap<UserId, Vec<String>>,
    pub machines: BTreeMap<MachineId, Vec<String>>,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawConfig {
    #[serde(default = "listen")]
    listen: SocketAddr,
    public_url: String,
    database: DatabaseConfig,
    auth: AuthConfig,
    mounts: Vec<Mount>,
    #[serde(default)]
    roles: BTreeMap<String, BTreeMap<MountName, Grant>>,
    #[serde(default)]
    users: BTreeMap<UserId, Vec<String>>,
    #[serde(default)]
    machines: BTreeMap<MachineId, Vec<String>>,
}
fn listen() -> SocketAddr {
    "127.0.0.1:8080".parse().unwrap()
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mount {
    pub name: MountName,
    pub path: PathBuf,
}

impl TryFrom<RawConfig> for Config {
    type Error = String;
    fn try_from(raw: RawConfig) -> Result<Self, String> {
        let public = url::Url::parse(&raw.public_url).map_err(|_| "invalid public_url")?;
        if !matches!(public.scheme(), "http" | "https")
            || public.host_str().is_none()
            || !public.username().is_empty()
            || public.password().is_some()
            || public.path() != "/"
            || public.query().is_some()
            || public.fragment().is_some()
        {
            return Err(
                "public_url must be an HTTP(S) origin without credentials, path, query or fragment"
                    .into(),
            );
        }
        let mut names = HashSet::new();
        for mount in &raw.mounts {
            if !names.insert(&mount.name) {
                return Err(format!("duplicate mount: {}", mount.name));
            }
        }
        for (role, grants) in &raw.roles {
            for mount in grants.keys() {
                if !names.contains(mount) {
                    return Err(format!("role {role} refers to unknown mount {mount}"));
                }
            }
        }
        for role in raw.users.values().chain(raw.machines.values()).flatten() {
            if !raw.roles.contains_key(role) {
                return Err(format!("unknown role: {role}"));
            }
        }
        Ok(Self {
            listen: raw.listen,
            public_origin: public.origin().ascii_serialization(),
            database: raw.database,
            auth: raw.auth,
            mounts: raw.mounts,
            roles: raw.roles,
            users: raw.users,
            machines: raw.machines,
        })
    }
}
impl Config {
    /// TOML followed by FILE_SERVER_* overrides; double underscores separate nested keys.
    #[hotpath::measure(impl_type = "Config")]
    pub fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let contents = std::fs::read_to_string(path)?;
        let mut config: Self = Figment::new()
            .merge(Toml::string(&contents))
            .merge(
                Env::prefixed("FILE_SERVER_")
                    .ignore(&["CONFIG"])
                    .split("__"),
            )
            .extract()?;
        let directory = path.parent().unwrap_or(Path::new("."));
        for mount in &mut config.mounts {
            if mount.path.is_relative() {
                mount.path = directory.join(&mount.path);
            }
        }
        if let Some(basic) = &mut config.auth.r#static
            && basic.directory.is_relative()
        {
            basic.directory = directory.join(&basic.directory);
        }
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn example_config_parses_and_rejects_dangling_references() {
        let example = include_str!("../file_server.example.toml");
        let config: Config = toml::from_str(example).unwrap();
        assert_eq!(config.public_origin, "http://localhost:8080");
        assert!(
            toml::from_str::<Config>(
                &example.replace(r#"documents = "editor""#, r#"unknown = "editor""#)
            )
            .is_err()
        );
        assert!(
            toml::from_str::<Config>(
                &example.replace(r#"alice = ["admin"]"#, r#"alice = ["missing"]"#)
            )
            .is_err()
        );
    }
}
