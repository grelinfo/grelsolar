//! Home Assistant Client.
//! This client is the higher level API client for Home Assistant.

use super::http_client::HttpClient;
use crate::home_assistant::{http_client, schemas::StateCreateOrUpdate};
use chrono::TimeZone;
use reqwest::Url;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("HTTP client error: {0}")]
    HttpClientError(#[from] http_client::Error),
    #[error("JSON serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

type Result<T> = std::result::Result<T, Error>;

pub struct Client {
    http: HttpClient,
}

impl Client {
    /// Creates a new instance of `Client`.
    pub fn new(url: &Url, token: &str) -> Self {
        let http = HttpClient::new(url, token);
        Client { http }
    }

    /// Set the solar energy produced today in Home Assistant.
    pub async fn set_solar_energy_today(&self, energy_today: i64) -> Result<()> {
        let kwh = energy_today as f64 / 1000.0; // Convert to kWh
        let last_reset = today_midnight_rfc3339();
        let state = StateCreateOrUpdate {
            state: kwh.to_string(),
            attributes: Some(
                [
                    ("unit_of_measurement".to_string(), "kWh".to_string()),
                    ("friendly_name".to_string(), "Solar Energy".to_string()),
                    ("device_class".to_string(), "energy".to_string()),
                    ("state_class".to_string(), "total_increasing".to_string()),
                    ("last_reset".to_string(), last_reset),
                ]
                .into_iter()
                .collect(),
            ),
        };
        self.http.set_state("sensor.solar_energy", &state).await?;
        Ok(())
    }

    /// Set the solar current power in Home Assistant.
    pub async fn set_solar_current_power(&self, power: i64) -> Result<()> {
        let state = StateCreateOrUpdate {
            state: power.to_string(),
            attributes: Some(
                [
                    ("unit_of_measurement".to_string(), "W".to_string()),
                    ("friendly_name".to_string(), "Solar Power".to_string()),
                    ("state_class".to_string(), "measurement".to_string()),
                ]
                .into_iter()
                .collect(),
            ),
        };
        self.http.set_state("sensor.solar_power", &state).await?;
        Ok(())
    }

    // Set the solar current status in Home Assistant.
    pub async fn set_solar_status(&self, status: &str) -> Result<()> {
        let state = StateCreateOrUpdate {
            state: status.to_string(),
            attributes: Some(
                [("friendly_name".to_string(), "Solar Status".to_string())]
                    .into_iter()
                    .collect(),
            ),
        };
        self.http.set_state("sensor.solar_status", &state).await?;
        Ok(())
    }
}

fn today_midnight_rfc3339() -> String {
    chrono::Local::now()
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .and_then(|midnight| chrono::Local.from_local_datetime(&midnight).single())
        .map(|dt| dt.to_rfc3339())
        .expect("cannot create midnight time")
}
