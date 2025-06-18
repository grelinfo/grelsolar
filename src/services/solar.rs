//! Solar Service.
//! This service bridges SolarLog and Home Assistant, enabling automatic synchronization of solar production data between the two systems.

use tokio::time::{interval, Duration};
use std::sync::Arc;

use crate::{home_assistant, solarlog};

pub struct SolarService {
    solarlog: Arc<solarlog::Client>,
    home_assistant: Arc<home_assistant::Client>,
}

impl SolarService {
    /// Creates a new instance of `SolarService`.
    pub fn new(
        solarlog: Arc<solarlog::Client>,
        home_assistant: Arc<home_assistant::Client>,
    ) -> Self {
        SolarService {
            solarlog,
            home_assistant,
        }
    }

    /// Runs all polling tasks concurrently.
    pub async fn run(&self) {
        tokio::join!(
            self.current_power_task(),
            self.energy_today_task(),
            self.inverter_status_task()
        );
    }

    /// Periodically polls SolarLog for current power and updates Home Assistant if changed.
    async fn current_power_task(&self) {
        let mut last_power = None;
        let mut interval = interval(Duration::from_secs(1));

        loop {
            interval.tick().await;
            match self.solarlog.get_current_power().await {
                Ok(Some(power)) if last_power != Some(power) => {
                    if let Err(e) = self.home_assistant.set_solar_current_power(power).await {
                        log::error!("Failed to update current power in Home Assistant: {e}");
                    } else {
                        last_power = Some(power);
                        log::debug!("Updated current power in Home Assistant: {power} W");
                    }
                }
                Ok(Some(_)) => {}
                Ok(None) => log::warn!("No current power data available from SolarLog"),
                Err(e) => log::error!("Error retrieving current power from SolarLog: {e}"),
            }
        }
    }

    /// Periodically retrieves the inverter status from SolarLog and updates Home Assistant if it changes.
    async fn energy_today_task(&self) {
        let mut last_energy_today = None;
        let mut interval = interval(Duration::from_secs(60));
        let mut today = chrono::Local::now().date_naive();

        loop {
            interval.tick().await;

            let now = chrono::Local::now().date_naive();
            if now != today {
                today = now;
                last_energy_today = None;
            }

            match self.solarlog.get_energy_today().await {
                Ok(Some(energy_today)) => {
                    if last_energy_today != Some(energy_today) {
                        if let Err(e) = self.home_assistant.set_solar_energy_today(energy_today).await {
                            log::error!("Failed to update energy today in Home Assistant: {e}");
                        }
                        last_energy_today = Some(energy_today);
                        log::debug!("Updated energy today in Home Assistant: {energy_today} Wh");
                    }
                }
                Ok(None) => log::warn!("No energy today data available from SolarLog"),
                Err(e) => log::error!("Error retrieving energy today from SolarLog: {e}"),
            }
        }
    }

    /// Periodically retrieves the inverter status from SolarLog and updates Home Assistant if it changes.
    async fn inverter_status_task(&self) {
        let mut last_status = None;
        let mut interval = interval(Duration::from_secs(60));

        loop {
            interval.tick().await;

            match self.solarlog.get_status().await {
                Ok(Some(status)) => {
                    if last_status.as_ref() != Some(&status) {
                        let status_str = status.to_string();
                        if let Err(e) = self.home_assistant.set_solar_status(&status_str).await {
                            log::error!("Failed to update inverter status in Home Assistant: {e}");
                        } else {
                            last_status = Some(status);
                            log::debug!("Updated inverter status in Home Assistant: {status_str}");
                        }
                    }
                }
                Ok(None) => log::warn!("No inverter status data available from SolarLog"),
                Err(e) => log::error!("Error retrieving inverter status from SolarLog: {e}"),
            }
        }
    }
}