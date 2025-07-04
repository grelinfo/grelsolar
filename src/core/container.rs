//! Dependency injection container for grelsolar.

use std::sync::Arc;

use super::config::Config;
use crate::integration::{homeassistant, solarlog};
use crate::services;

pub struct Container {
    pub solar_service: Arc<services::SolarBridgeBackgroundService>,
}

impl Container {
    /// Creates a new instance of the dependency injection container.
    pub fn new(config: &Config) -> Self {
        let solarlog = Arc::new(solarlog::Client::new(
            config.solarlog_url.to_owned(),
            config.solarlog_password.to_owned(),
        ));
        let homeassistant = Arc::new(homeassistant::Client::new(
            config.homeassistant_url.to_owned(),
            config.homeassistant_token.to_owned(),
        ));
        let solar_service = Arc::new({
            services::SolarBridgeBackgroundService::new(
                Arc::clone(&solarlog),
                Arc::clone(&homeassistant),
                config.solar_power_sync_interval,
                config.solar_energy_sync_interval,
                config.solar_status_sync_interval,
            )
        });
        Self { solar_service }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_container_new_wires_dependencies() {
        let config = Config {
            app_name: "test_app".into(),
            app_version: "0.0.0".into(),
            app_log: "info".into(),
            app_log_style: "auto".into(),
            solarlog_url: reqwest::Url::parse("http://localhost:1234").unwrap(),
            solarlog_password: "pw".into(),
            homeassistant_url: reqwest::Url::parse("http://localhost:5678").unwrap(),
            homeassistant_token: "token".into(),
            solar_power_sync_interval: std::time::Duration::from_secs(1),
            solar_energy_sync_interval: std::time::Duration::from_secs(1),
            solar_status_sync_interval: std::time::Duration::from_secs(1),
        };
        let _container = Container::new(&config);
    }
}
