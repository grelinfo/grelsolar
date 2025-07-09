//! Server
use crate::core::config::{APP_NAME, APP_VERSION, Config};
use crate::core::container::Container;
use tokio_util::sync::CancellationToken;

/// Run the server with the given configuration and shutdown token
pub async fn server(config: Config, shutdown_token: CancellationToken) {
    let container = Container::new(config);
    log::info!("{APP_NAME} v{APP_VERSION} started");
    container.solar_service().run(shutdown_token).await;
    container.shutdown().await;
}
