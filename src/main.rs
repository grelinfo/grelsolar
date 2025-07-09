//! grelsolar - A Rust application for solar energy management
//! The application is small enough to run on a single worker thread,
//! making it suitable for low-resource environments.
use envconfig::Envconfig;
use grelsolar::core::config::{Config, configure_logger};
use grelsolar::server::server;
use tokio::signal;
use tokio_util::sync::CancellationToken;

enum ExitCode {
    Success = 0,
    RuntimeError = 1,
    ConfigError = 2,
}

#[tokio::main(flavor = "multi_thread", worker_threads = 1)]
async fn main() {
    dotenvy::dotenv().ok();
    configure_logger();

    let config = match Config::init_from_env() {
        Ok(cfg) => cfg,
        Err(e) => {
            log::error!("Failed to load configuration: {e}");
            std::process::exit(ExitCode::ConfigError as i32);
        }
    };

    let shutdown_token = CancellationToken::new();
    let server_shutdown_token = shutdown_token.clone();

    let app = tokio::spawn(async move { server(config, server_shutdown_token).await });

    signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
    log::info!("Received Ctrl+C, waiting for graceful shutdown...");
    shutdown_token.cancel();

    match app.await {
        Ok(()) => {
            log::info!("Graceful shutdown completed");
            std::process::exit(ExitCode::Success as i32);
        }
        Err(e) => {
            log::error!("Application crashed: {e}");
            std::process::exit(ExitCode::RuntimeError as i32);
        }
    }
}
