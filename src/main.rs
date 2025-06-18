//! Grust Application

use grust::{config::Config, container::Container};


#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    env_logger::init();

    let config = match Config::from_env() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Configuration error: {:?}", e);
            std::process::exit(1);
        }
    };
    let container = Container::new(&config);

    log::info!("Starting Solar Service...");
    container.solar_service.run().await;
    log::info!("Solar Service stopped gracefully.");
}