//! Solar Bridge Background Service.
//! This service bridges SolarLog and Home Assistant, enabling automatic synchronization of solar production data between the two systems.

use chrono::{DateTime, NaiveDate};
use std::sync::Arc;
use tokio::time::{Duration, interval};

use crate::integration::{homeassistant, solarlog};

pub struct SolarBridgeBackgroundService {
    solarlog: Arc<solarlog::Client>,
    homeassistant: Arc<homeassistant::Client>,
    power_period: Duration,
    energy_period: Duration,
    status_period: Duration,
}

impl SolarBridgeBackgroundService {
    /// Creates a new instance of `SolarService`.
    pub fn new(
        solarlog: Arc<solarlog::Client>,
        homeassistant: Arc<homeassistant::Client>,
        power_period: Duration,
        energy_period: Duration,
        status_period: Duration,
    ) -> Self {
        SolarBridgeBackgroundService {
            solarlog,
            homeassistant,
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
    pub async fn sync_solar_power(&self, last_power: Option<i64>) -> Option<i64> {
        let power = match self.solarlog.get_current_power().await {
            Ok(power) if last_power != Some(power) => power,
            Ok(power) => return Some(power),
            Err(e) => {
                log::error!("Solar Power: {e}");
                return last_power;
            }
        };

        // Only reach
        match self.homeassistant.set_solar_current_power(power).await {
            Ok(_) => log::debug!("Solar Power: {power} W"),
            Err(e) => log::error!("Solar Power update: {e}"),
        }

        Some(power)
    }

    /// Synchronizes the solar energy produced today with Home Assistant.
    pub async fn sync_solar_energy(
        &self,
        day: NaiveDate,
        last_energy_today: Option<i64>,
    ) -> Option<i64> {
        let energy = match self.solarlog.get_energy_of_day(day).await {
            Ok(energy) if last_energy_today != Some(energy) => energy,
            Ok(power) => return Some(power),
            Err(e) => {
                log::error!("Solar Energy: {e}");
                return last_energy_today;
            }
        };

        let day_midnight = Self::day_midnight(&day);
        match self
            .homeassistant
            .set_solar_energy(energy, &day_midnight)
            .await
        {
            Ok(_) => log::debug!("Solar Energy: {energy} Wh"),
            Err(e) => log::error!("Solar Energy update: {e}"),
        }

        Some(energy)
    }

    /// Synchronizes the SolarLog device status with Home Assistant.
    pub async fn sync_solar_status(
        &self,
        last_status: Option<solarlog::InverterStatus>,
    ) -> Option<solarlog::InverterStatus> {
        let status = match self.solarlog.get_status().await {
            Ok(status) if last_status.as_ref() != Some(&status) => status,
            Ok(status) => return Some(status),
            Err(e) => {
                log::error!("Solar Status: {e}");
                return last_status;
            }
        };

        let status_str = status.to_string();
        match self.homeassistant.set_solar_status(&status_str).await {
            Ok(_) => log::debug!("Solar Status: {status_str}"),
            Err(e) => log::error!("Solar Status update: {e}"),
        }

        Some(status)
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
