//! Dependency injection container for grelsolar.

use std::sync::Arc;

use super::config::Config;
use crate::integration::{homeassistant, solarlog};
use crate::services;

/// Container for application dependencies.
pub struct Container {
    config: Arc<Config>,
    solarlog: Arc<solarlog::Client>,
    homeassistant: Arc<homeassistant::Client>,
    solar_service: Arc<services::SolarBridgeBackgroundService>,
}

impl Container {
    /// Creates a new instance of the dependency injection container.
    pub fn new(config: Config) -> Self {
        let config = Arc::new(config);

        let solarlog = Arc::new(solarlog::Client::new(
            config.solarlog_url.clone(),
            config.solarlog_password.clone(),
        ));

        let homeassistant = Arc::new(homeassistant::Client::new(
            config.homeassistant_url.clone(),
            config.homeassistant_token.clone(),
        ));

        let solar_service = Arc::new(services::SolarBridgeBackgroundService::new(
            Arc::clone(&solarlog),
            Arc::clone(&homeassistant),
            config.sync_power_interval.into(),
            config.sync_energy_interval.into(),
            config.sync_status_interval.into(),
        ));

        Self {
            config,
            solarlog,
            homeassistant,
            solar_service,
        }
    }

    /// Returns a reference to the application config.
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Returns a reference to the solar service.
    pub fn solar_service(&self) -> Arc<services::SolarBridgeBackgroundService> {
        Arc::clone(&self.solar_service)
    }

    /// Returns a reference to the SolarLog client.
    pub fn solarlog_client(&self) -> Arc<solarlog::Client> {
        Arc::clone(&self.solarlog)
    }

    /// Returns a reference to the HomeAssistant client.
    pub fn homeassistant_client(&self) -> Arc<homeassistant::Client> {
        Arc::clone(&self.homeassistant)
    }

    /// Shutdown the container and clean up resources.
    pub async fn shutdown(&self) {
        self.solarlog.logout().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use humantime::Duration;

    fn config() -> Config {
        Config {
            app_log: "info".into(),
            app_log_style: "auto".into(),
            solarlog_url: reqwest::Url::parse("http://localhost:1234").unwrap(),
            solarlog_password: "pw".into(),
            homeassistant_url: reqwest::Url::parse("http://localhost:2222").unwrap(),
            homeassistant_token: "token2".into(),
            sync_power_interval: Duration::from(std::time::Duration::from_secs(10)),
            sync_energy_interval: Duration::from(std::time::Duration::from_secs(2)),
            sync_status_interval: Duration::from(std::time::Duration::from_secs(3)),
        }
    }

    #[tokio::test]
    async fn test_container_init() {
        let config = config();
        let container = Container::new(config);

        container.shutdown().await;

        assert_eq!(container.config().app_log, "info");
        assert!(Arc::ptr_eq(
            &container.solarlog_client(),
            &container.solarlog_client()
        ));
        assert!(Arc::ptr_eq(
            &container.homeassistant_client(),
            &container.homeassistant_client()
        ));
        assert!(Arc::ptr_eq(
            &container.solar_service(),
            &container.solar_service()
        ));

        assert!(Arc::strong_count(&container.solarlog_client()) >= 1);
        assert!(Arc::strong_count(&container.homeassistant_client()) >= 1);
        assert!(Arc::strong_count(&container.solar_service()) >= 1);
    }
}
