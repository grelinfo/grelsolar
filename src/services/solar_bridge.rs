//! Solar Bridge Background Service.
//! This service bridges SolarLog and Home Assistant, enabling automatic synchronization of solar production data between the two systems.

use chrono::{DateTime, NaiveDate};
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
            self.sync_solar_power_task(self.power_period),
            self.sync_solar_energy_task(self.energy_period),
            self.sync_solar_status_task(self.status_period)
        );
    }

    /// Periodically retrieves the current power from SolarLog and updates Home Assistant if it changes.
    /// This method runs in a loop, polling the SolarLog API at the specified interval.
    /// # Arguments
    /// * `period` - The interval at which to poll SolarLog for current power data.
    async fn sync_solar_power_task(&self, period: Duration) {
        let mut last_power = None;
        let mut interval = interval(period);

        loop {
            interval.tick().await;
            last_power = self.sync_solar_power(last_power).await;
        }
    }

    /// Periodically retrieves the inverter status from SolarLog and updates Home Assistant if it changes.
    /// This method runs in a loop, polling the SolarLog API at the specified interval.
    /// # Arguments
    /// * `period` - The interval at which to poll SolarLog for inverter status data.
    async fn sync_solar_energy_task(&self, period: Duration) {
        let mut last_energy_today = None;
        let mut last_day = chrono::Local::now().date_naive();
        let mut interval = interval(period);

        loop {
            interval.tick().await;
            let day = chrono::Local::now().date_naive();
            if day != last_day {
                last_day = day;
                last_energy_today = None; // Reset energy for a new day
            }
            last_energy_today = self.sync_solar_energy(day, last_energy_today).await;
        }
    }

    /// Periodically retrieves the inverter status from SolarLog and updates Home Assistant if it changes.
    /// This method runs in a loop, polling the SolarLog API at the specified interval.
    /// # Arguments
    /// * `period` - The interval at which to poll SolarLog for inverter status data.
    async fn sync_solar_status_task(&self, period: Duration) {
        let mut last_status = None;
        let mut interval = interval(period);
        loop {
            interval.tick().await;
            last_status = self.sync_solar_status(last_status).await;
        }
    }

    /// Synchronizes the current solar power with Home Assistant.
    async fn sync_solar_power(&self, last_power: Option<i64>) -> Option<i64> {
        match self.solarlog.get_current_power().await {
            Ok(Some(power)) if last_power != Some(power) => {
                if let Err(e) = self.home_assistant.set_solar_current_power(power).await {
                    log::error!("Cannot update current power in Home Assistant: {e}");
                } else {
                    log::debug!("Current power updated in Home Assistant: {power} W");
                }
                Some(power)
            }
            Ok(Some(_)) => last_power,
            Ok(None) => {
                log::warn!("Cannot retrieve current power from SolarLog");
                last_power
            }
            Err(e) => {
                log::error!("Cannot retrieve current power from SolarLog: {e}");
                last_power
            }
        }
    }

    /// Synchronizes the solar energy produced today with Home Assistant.
    async fn sync_solar_energy(
        &self,
        day: NaiveDate,
        last_energy_today: Option<i64>,
    ) -> Option<i64> {
        match self.solarlog.get_energy_of_day(day).await {
            Ok(Some(energy_today)) if last_energy_today != Some(energy_today) => {
                let day_midnight = Self::day_midnight(&day);
                if let Err(e) = self
                    .home_assistant
                    .set_solar_energy(energy_today, &day_midnight)
                    .await
                {
                    log::error!("Cannot update energy today in Home Assistant: {e}");
                } else {
                    log::debug!("Energy today updated in Home Assistant: {energy_today} Wh");
                }
                Some(energy_today)
            }
            Ok(Some(_)) => last_energy_today,
            Ok(None) => {
                log::warn!("Cannot retrieve energy today from SolarLog");
                last_energy_today
            }
            Err(e) => {
                log::error!("Cannot retrieve energy today from SolarLog: {e}");
                last_energy_today
            }
        }
    }

    /// Synchronizes the SolarLog device status with Home Assistant.
    async fn sync_solar_status(
        &self,
        last_status: Option<solarlog::InverterStatus>,
    ) -> Option<solarlog::InverterStatus> {
        match self.solarlog.get_status().await {
            Ok(Some(status)) if last_status.as_ref() != Some(&status) => {
                let status_str = status.to_string();
                if let Err(e) = self.home_assistant.set_solar_status(&status_str).await {
                    log::error!("Cannot update inverter status in Home Assistant: {e}");
                } else {
                    log::debug!("Inverter status updated in Home Assistant: {status_str}");
                }
                Some(status)
            }
            Ok(Some(_)) => last_status,
            Ok(None) => {
                log::warn!("Cannot retrieve inverter status from SolarLog");
                last_status
            }
            Err(e) => {
                log::error!("Cannot retrieve inverter status from SolarLog: {e}");
                last_status
            }
        }
    }

    fn day_midnight(day: &NaiveDate) -> DateTime<chrono::Local> {
        day.and_hms_opt(0, 0, 0)
            .and_then(|d| d.and_local_timezone(chrono::Local).single())
            .expect("cannot create midnight time")
    }
}

#[cfg(test)]
mod tests {
    use super::SolarBridgeBackgroundService;
    use chrono::{Local, Timelike};

    #[test]
    fn test_today_midnight_is_midnight() {
        let today = Local::now().date_naive();
        let midnight = SolarBridgeBackgroundService::day_midnight(&today);
        let now = Local::now();

        // Should be today
        assert_eq!(midnight.date_naive(), now.date_naive());

        // Should be at 00:00:00
        assert_eq!(midnight.hour(), 0);
        assert_eq!(midnight.minute(), 0);
        assert_eq!(midnight.second(), 0);
    }
}
