//! Dependency injection container for grelsolar.

use std::sync::Arc;

use crate::config::Config;
use crate::integration::{homeassistant, solarlog};
use crate::services;

pub struct Container {
    pub solarlog: Arc<solarlog::Client>,
    pub homeassistant: Arc<homeassistant::Client>,
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
        Self {
            solarlog,
            homeassistant,
            solar_service,
        }
    }
}
