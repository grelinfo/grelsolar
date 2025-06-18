//! Grust Application

use grust::{config::Config, container::Container};


#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let env = env_logger::Env::default()
        .filter_or("APP_LOG", "info")
        .write_style_or("APP_LOG_STYLE", "always");

    env_logger::init_from_env(env);

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
    log::info!("{} gracefull shutdown", config.app_name);
    std::process::exit(0);
}