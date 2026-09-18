//! Home Assistant Client.
//! This client is the higher level API client for Home Assistant.

use super::Result;
use super::http_client::HttpClient;
use super::schemas::StateCreateOrUpdate;
use reqwest::Url;
use std::time::Duration;

/// State of a sensor whose source cannot be reached.
const UNAVAILABLE: &str = "unavailable";

pub struct Client {
    http: HttpClient,
}

impl Client {
    /// Creates a new instance of `Client`.
    /// `timeout` applies to each HTTP request.
    pub fn new(url: Url, token: String, timeout: Duration) -> Self {
        let http = HttpClient::new(url, token, timeout);
        Client { http }
    }

    /// Set the solar energy produced today in Home Assistant.
    pub async fn set_solar_energy(&self, energy_today: i64) -> Result<()> {
        let state = Self::create_solar_energy_state(energy_today);
        self.http.set_state("sensor.solar_energy", &state).await?;
        Ok(())
    }

    /// Set the solar current power in Home Assistant.
    pub async fn set_solar_current_power(&self, power: i64) -> Result<()> {
        let state = Self::create_solar_current_power_state(power);
        self.http.set_state("sensor.solar_power", &state).await?;
        Ok(())
    }

    /// Set the solar current status in Home Assistant.
    pub async fn set_solar_status(&self, status: &str) -> Result<()> {
        let state = Self::create_solar_status_state(status);
        self.http.set_state("sensor.solar_status", &state).await?;
        Ok(())
    }

    /// Mark the solar energy sensor as unavailable in Home Assistant.
    pub async fn set_solar_energy_unavailable(&self) -> Result<()> {
        let state = Self::unavailable(Self::create_solar_energy_state(0));
        self.http.set_state("sensor.solar_energy", &state).await
    }

    /// Mark the solar power sensor as unavailable in Home Assistant.
    pub async fn set_solar_current_power_unavailable(&self) -> Result<()> {
        let state = Self::unavailable(Self::create_solar_current_power_state(0));
        self.http.set_state("sensor.solar_power", &state).await
    }

    /// Mark the solar status sensor as unavailable in Home Assistant.
    pub async fn set_solar_status_unavailable(&self) -> Result<()> {
        let state = Self::unavailable(Self::create_solar_status_state(""));
        self.http.set_state("sensor.solar_status", &state).await
    }

    /// Replace the value of a state with `unavailable`, keeping its attributes.
    fn unavailable(state: StateCreateOrUpdate) -> StateCreateOrUpdate {
        StateCreateOrUpdate {
            state: UNAVAILABLE.to_string(),
            ..state
        }
    }

    /// Create current power state for solar status.
    fn create_solar_current_power_state(power: i64) -> StateCreateOrUpdate {
        StateCreateOrUpdate {
            state: power.to_string(),
            attributes: Some(
                [
                    ("unit_of_measurement".to_string(), "W".to_string()),
                    ("friendly_name".to_string(), "Solar Power".to_string()),
                    ("device_class".to_string(), "power".to_string()),
                    ("state_class".to_string(), "measurement".to_string()),
                ]
                .into_iter()
                .collect(),
            ),
        }
    }

    /// Create the state for solar energy produced today.
    fn create_solar_energy_state(energy_today: i64) -> StateCreateOrUpdate {
        let kwh = energy_today as f64 / 1000.0; // Convert to kWh
        StateCreateOrUpdate {
            state: kwh.to_string(),
            attributes: Some(
                [
                    ("unit_of_measurement".to_string(), "kWh".to_string()),
                    ("friendly_name".to_string(), "Solar Energy".to_string()),
                    ("device_class".to_string(), "energy".to_string()),
                    ("state_class".to_string(), "total_increasing".to_string()),
                ]
                .into_iter()
                .collect(),
            ),
        }
    }

    /// Create the state for solar status.
    fn create_solar_status_state(status: &str) -> StateCreateOrUpdate {
        StateCreateOrUpdate {
            state: status.to_string(),
            attributes: Some(
                [("friendly_name".to_string(), "Solar Status".to_string())]
                    .into_iter()
                    .collect(),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    /// Test client creation with a valid URL and token don't panic.
    #[tokio::test]
    async fn test_new() {
        let url = Url::parse("http://localhost:8123").unwrap();
        let token = String::from("test_token");
        Client::new(url, token, Duration::from_millis(500));
    }

    #[rstest]
    #[case(1234, "1234")]
    #[case(0, "0")]
    #[case(-567, "-567")]
    fn test_create_solar_current_power_state(#[case] power: i64, #[case] expected_state: &str) {
        let expected = StateCreateOrUpdate {
            state: expected_state.to_string(),
            attributes: Some(
                [
                    ("unit_of_measurement".to_string(), "W".to_string()),
                    ("friendly_name".to_string(), "Solar Power".to_string()),
                    ("device_class".to_string(), "power".to_string()),
                    ("state_class".to_string(), "measurement".to_string()),
                ]
                .into_iter()
                .collect(),
            ),
        };

        let state = Client::create_solar_current_power_state(power);

        assert_eq!(state, expected);
    }

    #[rstest]
    #[case(5000, "5")]
    #[case(0, "0")]
    #[case(-5000, "-5")]
    #[case(1234, "1.234")]
    #[case(-1234, "-1.234")]
    #[case(123, "0.123")]
    #[case(-123, "-0.123")]
    fn test_create_solar_energy_state(#[case] energy_today: i64, #[case] expected_state: &str) {
        let expected = StateCreateOrUpdate {
            state: expected_state.to_string(),
            attributes: Some(
                [
                    ("unit_of_measurement".to_string(), "kWh".to_string()),
                    ("friendly_name".to_string(), "Solar Energy".to_string()),
                    ("device_class".to_string(), "energy".to_string()),
                    ("state_class".to_string(), "total_increasing".to_string()),
                ]
                .into_iter()
                .collect(),
            ),
        };

        let state = Client::create_solar_energy_state(energy_today);

        assert_eq!(state, expected);
    }

    #[test]
    fn test_unavailable_keeps_attributes() {
        let state = Client::create_solar_current_power_state(1234);

        let unavailable = Client::unavailable(state);

        assert_eq!(unavailable.state, "unavailable");
        assert_eq!(
            unavailable.attributes,
            Client::create_solar_current_power_state(0).attributes
        );
    }

    #[rstest]
    #[case("On-grid")]
    #[case("Idle No irradiation")]
    fn test_create_solar_status_state(#[case] status: &str) {
        let expected = StateCreateOrUpdate {
            state: status.to_string(),
            attributes: Some(
                [("friendly_name".to_string(), "Solar Status".to_string())]
                    .into_iter()
                    .collect(),
            ),
        };

        let state = Client::create_solar_status_state(status);

        assert_eq!(state, expected);
    }
}
