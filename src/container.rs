//! Dependency injection container for Grust.

use std::sync::Arc;

use crate::{config::Config, home_assistant,services, solarlog};



pub struct Container {
    pub solarlog: Arc<solarlog::Client>,
    pub home_assistant: Arc<home_assistant::Client>,
    pub solar_service: Arc<services::SolarBridgeBackgroundService>,
}

impl Container {
    /// Creates a new instance of the dependency injection container.
    pub fn new(config: &Config) -> Self {
        let solarlog = Arc::new(solarlog::Client::new(
            &config.solarlog_url,
            &config.solarlog_password,
        ));
        let home_assistant = Arc::new(home_assistant::Client::new(
            &config.home_assistant_url,
            &config.home_assistant_token,
        ));
        let solar_service = Arc::new({
            services::SolarBridgeBackgroundService::new(
                Arc::clone(&solarlog),
                Arc::clone(&home_assistant),
                config.solar_power_period,
                config.solar_energy_period,
                config.solar_status_period
            )
        });
        Self {
            solarlog,
            home_assistant,
            solar_service,
        }
    }
}
