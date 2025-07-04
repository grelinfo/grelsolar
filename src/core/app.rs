//! Core application logic for grelsolar
use super::config::{Config, configure_logger};
use super::container::Container;

pub async fn app() -> Result<(), anyhow::Error> {
    dotenvy::dotenv().ok();
    configure_logger();
    let config = Config::from_env()?;
    let container = Container::new(&config);

    log::info!("{} (v{}) started", config.app_name, config.app_version);
    container.solar_service.run().await;
    log::info!("{} stopped", config.app_name);
    Ok(())
}
