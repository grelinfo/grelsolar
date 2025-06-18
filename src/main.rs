//! Grust Application

use grust::{
    config::{Config, configure_logger},
    container::Container,
};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    configure_logger();
    let config = match Config::from_env() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Configuration error: {:?}", e);
            std::process::exit(1);
        }
    };
    let container = Container::new(&config);

    log::info!("{} (v{}) started", config.app_name, config.app_version);
    container.solar_service.run().await;
    log::info!("{} stopped", config.app_name);
    std::process::exit(0);
}
