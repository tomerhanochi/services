use crate::config::Config;
use authn_identity::prelude::{Identity, Machine, User};
use axum::Router;
use file_server_authn::prelude::{AppAuthenticator, configure};
use file_server_core::prelude::{FileService, Grant, MountName};
use file_server_fs::prelude::TokioStore;
use file_server_postgres::prelude::PostgresAccess;
use file_server_web::prelude::{AppState, Ports};
use std::{error::Error, path::Path, sync::Arc};

type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;
struct AppPorts {
    files: FileService<TokioStore, PostgresAccess>,
    auth: AppAuthenticator,
}
impl Ports for AppPorts {
    type Files = TokioStore;
    type Access = PostgresAccess;
    type Auth = AppAuthenticator;
    fn files(&self) -> &FileService<Self::Files, Self::Access> {
        &self.files
    }
    fn authenticator(&self) -> &Self::Auth {
        &self.auth
    }
}
pub struct Server {
    listen: std::net::SocketAddr,
    router: Router,
}
impl Server {
    #[hotpath::measure(future = true)]
    pub async fn new(config: Config) -> Result<Self> {
        let access = PostgresAccess::connect(&config.database).await?;
        let grants: Vec<(String, MountName, Grant)> = config
            .roles
            .iter()
            .flat_map(|(role, grants)| {
                grants
                    .iter()
                    .map(move |(mount, grant)| (role.clone(), mount.clone(), *grant))
            })
            .collect();
        let principals = config
            .users
            .into_iter()
            .map(|(id, roles)| (Identity::User(User { id }), roles))
            .chain(
                config
                    .machines
                    .into_iter()
                    .map(|(id, roles)| (Identity::Machine(Machine { id }), roles)),
            )
            .collect::<Vec<_>>();
        let store = TokioStore::new(
            config
                .mounts
                .into_iter()
                .map(|mount| (mount.name, mount.path)),
        )
        .await?;
        let setup = configure(&config.auth).await?;
        access.sync_access(&grants, &principals).await?;
        let auth_routes = setup.routes();
        let state = AppState {
            ports: Arc::new(AppPorts {
                files: FileService::new(store, access),
                auth: setup.authenticator,
            }),
            css: Arc::new(htmlb_components::css::css(&htmlb_components_theme::THEME)),
            public_origin: config.public_origin,
            oidc_enabled: setup.oidc.is_some(),
        };
        Ok(Self {
            listen: config.listen,
            router: file_server_web::prelude::router(state, auth_routes),
        })
    }
    pub fn router(self) -> Router {
        self.router
    }
    pub fn listen(&self) -> std::net::SocketAddr {
        self.listen
    }
}

#[hotpath::measure(future = true)]
pub async fn run(path: &Path, shutdown: impl Future<Output = ()> + Send + 'static) -> Result<()> {
    let config = Config::load(path)?;
    let server = Server::new(config).await?;
    let listener = tokio::net::TcpListener::bind(server.listen()).await?;
    tracing::info!(address = %listener.local_addr()?, "file server listening");
    axum::serve(listener, server.router())
        .with_graceful_shutdown(shutdown)
        .await?;
    Ok(())
}
pub async fn shutdown_signal() {
    #[cfg(unix)]
    {
        let mut terminate =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .expect("install SIGTERM handler");
        tokio::select! { _ = tokio::signal::ctrl_c() => {}, _ = terminate.recv() => {} }
    }
    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
    }
}
