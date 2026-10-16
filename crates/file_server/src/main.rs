use file_server::prelude::{run, shutdown_signal};
use std::{path::PathBuf, process::ExitCode};

#[tokio::main]
#[hotpath::main(percentiles = [50, 95, 99], functions_limit = 50)]
async fn main() -> ExitCode {
    use tracing_subscriber::{Layer, layer::SubscriberExt, util::SubscriberInitExt};
    let logs = tracing_subscriber::fmt::layer().with_filter(
        tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
    );
    let subscriber = tracing_subscriber::registry().with(logs);
    // Keep the log filter on its own layer: SQLx profiling needs DEBUG events
    // even when RUST_LOG only displays INFO messages.
    #[cfg(feature = "hotpath")]
    let subscriber = subscriber.with(hotpath::sqlx_tracing_layer());
    subscriber.init();
    hotpath::tokio_runtime!();
    let argument = std::env::args_os().nth(1);
    if argument
        .as_ref()
        .is_some_and(|arg| arg == "--help" || arg == "-h")
    {
        println!(
            "Usage: file_server [CONFIG]\nDefaults to FILE_SERVER_CONFIG, then file_server.toml.\nOverride nested TOML keys with FILE_SERVER_SECTION__KEY."
        );
        return ExitCode::SUCCESS;
    }
    let path = argument
        .or_else(|| std::env::var_os("FILE_SERVER_CONFIG"))
        .map(PathBuf::from)
        .unwrap_or_else(|| "file_server.toml".into());
    match run(&path, shutdown_signal()).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            tracing::error!(%error, "file server failed");
            ExitCode::FAILURE
        }
    }
}
