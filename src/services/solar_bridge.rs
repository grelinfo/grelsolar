//! Solar Bridge Background Service.
//! This service bridges SolarLog and Home Assistant, enabling automatic synchronization of solar production data between the two systems.

use chrono::{DateTime, TimeZone};
use std::sync::Arc;
use tokio::time::{Duration, interval};

use crate::{home_assistant, solarlog};

pub struct SolarBridgeBackgroundService {
    solarlog: Arc<solarlog::Client>,
    home_assistant: Arc<home_assistant::Client>,
    power_period: Duration,
    energy_period: Duration,
    status_period: Duration,
}

impl SolarBridgeBackgroundService {
    /// Creates a new instance of `SolarService`.
    pub fn new(
        solarlog: Arc<solarlog::Client>,
        home_assistant: Arc<home_assistant::Client>,
        power_period: Duration,
        energy_period: Duration,
        status_period: Duration,
    ) -> Self {
        SolarBridgeBackgroundService {
            solarlog,
            home_assistant,
            power_period,
            energy_period,
            status_period,
        }
    }

    /// Run the background service to synchronize data between SolarLog and Home Assistant.
    pub async fn run(&self) {
        tokio::join!(
            self.sync_solar_power(self.power_period),
            self.sync_solar_energy(self.energy_period),
            self.sync_solar_status(self.status_period)
        );
    }

    /// Periodically retrieves the current power from SolarLog and updates Home Assistant if it changes.
    /// This method runs in a loop, polling the SolarLog API at the specified interval.
    /// # Arguments
    /// * `period` - The interval at which to poll SolarLog for current power data.
    async fn sync_solar_power(&self, period: Duration) {
        let mut last_power = None;
        let mut interval = interval(period);

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
    /// This method runs in a loop, polling the SolarLog API at the specified interval.
    /// # Arguments
    /// * `period` - The interval at which to poll SolarLog for inverter status data.
    async fn sync_solar_energy(&self, period: Duration) {
        let mut last_energy_today = None;
        let mut interval = interval(period);
        let mut today_midnight = Self::today_midnight();

        loop {
            interval.tick().await;

            let now = Self::today_midnight();
            if now != today_midnight {
                today_midnight = now;
                last_energy_today = None;
            }

            let day = today_midnight.date_naive();
            let energy_today = self.solarlog.get_energy_of_day(day).await;

            match energy_today {
                Ok(Some(energy_today)) => {
                    if last_energy_today != Some(energy_today) {
                        if let Err(e) = self
                            .home_assistant
                            .set_solar_energy(energy_today, &today_midnight)
                            .await
                        {
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
    /// This method runs in a loop, polling the SolarLog API at the specified interval.
    /// # Arguments
    /// * `period` - The interval at which to poll SolarLog for inverter status data.
    async fn sync_solar_status(&self, period: Duration) {
        let mut last_status = None;
        let mut interval = interval(period);
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

    fn today_midnight() -> DateTime<chrono::Local> {
        chrono::Local::now()
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .and_then(|midnight| chrono::Local.from_local_datetime(&midnight).single())
            .expect("cannot create midnight datetime")
    }
}

#[cfg(test)]
mod tests {
    use super::SolarBridgeBackgroundService;
    use chrono::{Local, Timelike};

    #[test]
    fn test_today_midnight_is_midnight() {
        let midnight = SolarBridgeBackgroundService::today_midnight();
        let now = Local::now();

        // Should be today
        assert_eq!(midnight.date_naive(), now.date_naive());

        // Should be at 00:00:00
        assert_eq!(midnight.hour(), 0);
        assert_eq!(midnight.minute(), 0);
        assert_eq!(midnight.second(), 0);
    }
}
