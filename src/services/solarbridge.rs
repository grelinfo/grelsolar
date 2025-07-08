//! Solar Bridge Background Service.
//! This service bridges SolarLog and Home Assistant, enabling automatic synchronization of solar production data between the two systems.

use chrono::{DateTime, NaiveDate};
use std::sync::Arc;
use tokio::time::{Duration, interval};

use crate::integration::{homeassistant, solarlog};

pub struct SolarBridgeBackgroundService {
    solarlog: Arc<solarlog::Client>,
    homeassistant: Arc<homeassistant::Client>,
    sync_power_interval: Duration,
    sync_energy_interval: Duration,
    sync_status_interval: Duration,
}

impl SolarBridgeBackgroundService {
    /// Creates a new instance of `SolarService`.
    pub fn new(
        solarlog: Arc<solarlog::Client>,
        homeassistant: Arc<homeassistant::Client>,
        sync_power_interval: Duration,
        sync_energy_interval: Duration,
        sync_status_interval: Duration,
    ) -> Self {
        SolarBridgeBackgroundService {
            solarlog,
            homeassistant,
            sync_power_interval,
            sync_energy_interval,
            sync_status_interval,
        }
    }

    /// Run the background service to synchronize data between SolarLog and Home Assistant.
    pub async fn run(&self) {
        tokio::join!(
            self.sync_solar_power_task(self.sync_power_interval),
            self.sync_solar_energy_task(self.sync_energy_interval),
            self.sync_solar_status_task(self.sync_status_interval)
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
            match self.sync_solar_power(last_power).await {
                Ok(power) => last_power = power,
                Err(e) => log::error!("Error syncing solar power: {e}"),
            }
        }
    }

    /// Periodically retrieves the inverter status from SolarLog and updates Home Assistant if it changes.
    /// This method runs in a loop, polling the SolarLog API at the specified interval.
    /// # Arguments
    /// * `period` - The interval at which to poll SolarLog for inverter status data.
    async fn sync_solar_energy_task(&self, period: Duration) {
        let mut last_value: Option<(NaiveDate, i64)> = None;
        let mut interval = interval(period);

        loop {
            interval.tick().await;
            match self.sync_solar_energy(last_value).await {
                Ok(energy) => last_value = energy,
                Err(e) => log::error!("Error syncing solar energy: {e}"),
            }
        }
    }

    /// Periodically retrieves the inverter status from SolarLog and updates Home Assistant if it changes.
    /// This method runs in a loop, polling the SolarLog API at the specified interval.
    /// # Arguments
    /// * `period` - The interval at which to poll SolarLog for inverter status data.
    async fn sync_solar_status_task(&self, period: Duration) {
        let mut last_status: Option<solarlog::InverterStatus> = None;
        let mut interval = interval(period);
        loop {
            interval.tick().await;
            match self.sync_solar_status(last_status.as_ref()).await {
                Ok(status) => last_status = status,
                Err(e) => log::error!("Error syncing solar status: {e}"),
            }
        }
    }

    /// Synchronizes the current solar power with Home Assistant.
    pub async fn sync_solar_power(
        &self,
        last_power: Option<i64>,
    ) -> Result<Option<i64>, anyhow::Error> {
        let power = self.solarlog.get_current_power().await?;
        if last_power == Some(power) {
            return Ok(Some(power));
        }
        // Only reach
        self.homeassistant.set_solar_current_power(power).await?;
        Ok(Some(power))
    }

    /// Synchronizes the solar energy produced today with Home Assistant.
    pub async fn sync_solar_energy(
        &self,
        last_value: Option<(NaiveDate, i64)>,
    ) -> Result<Option<(NaiveDate, i64)>, anyhow::Error> {
        let value = self.solarlog.get_energy_of_last_day().await?;
        if last_value == Some(value) {
            return Ok(Some(value));
        }
        self.set_solar_energy(value).await?;
        Ok(Some(value))
    }

    /// Synchronizes the SolarLog device status with Home Assistant.
    pub async fn sync_solar_status(
        &self,
        last_status: Option<&solarlog::InverterStatus>,
    ) -> Result<Option<solarlog::InverterStatus>, anyhow::Error> {
        let status = self.solarlog.get_status().await?;
        if last_status == Some(&status) {
            return Ok(Some(status));
        }
        let status_str = status.to_string();
        self.homeassistant.set_solar_status(&status_str).await?;
        Ok(Some(status))
    }

    async fn set_solar_energy(&self, value: (NaiveDate, i64)) -> Result<(), homeassistant::Error> {
        let day_midnight = Self::day_midnight(&value.0);
        self.homeassistant
            .set_solar_energy(value.1, &day_midnight)
            .await
    }

    pub fn day_midnight(day: &NaiveDate) -> DateTime<chrono::Local> {
        day.and_hms_opt(0, 0, 0)
            .expect("invalid time")
            .and_local_timezone(chrono::Local)
            .single()
            .expect("ambiguous timezone")
    }
}

#[cfg(test)]
mod tests {
    use super::SolarBridgeBackgroundService;
    use chrono::{Datelike, NaiveDate, Timelike};

    #[test]
    fn test_day_midnight() {
        let static_date = NaiveDate::from_ymd_opt(2024, 6, 1).unwrap();
        let midnight = SolarBridgeBackgroundService::day_midnight(&static_date);

        assert_eq!(midnight.year(), 2024);
        assert_eq!(midnight.month(), 6);
        assert_eq!(midnight.day(), 1);
        assert_eq!(midnight.hour(), 0);
        assert_eq!(midnight.minute(), 0);
        assert_eq!(midnight.second(), 0);
        assert_eq!(midnight.nanosecond(), 0);
    }
}
