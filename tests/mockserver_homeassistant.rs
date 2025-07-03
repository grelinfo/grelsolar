//! Mock server for Home Assistant API
use httpmock::{Method::POST, Mock, MockServer};
use reqwest::Url;
use serde_json::json;

/// Wrapper around `MockServer` for Home Assistant endpoint mocks.
pub struct HomeAssistantMockServer {
    pub server: MockServer,
}

impl HomeAssistantMockServer {
    /// Start and return a running MockServer for Home Assistant.
    pub async fn start() -> Self {
        let server = MockServer::start_async().await;
        HomeAssistantMockServer { server }
    }

    /// Get the base URL to use when constructing the client.
    pub fn url(&self) -> Url {
        Url::parse(&self.server.base_url()).expect("invalid mock server URL")
    }

    /// Token to use in Authorization headers in mocks.
    pub fn token(&self) -> &str {
        "test_token"
    }

    /// Mock the set state for solar energy with sample request/response.
    pub async fn mock_set_solar_energy<'a>(
        &'a self,
        energy_kwh: f64,
        last_reset: &str,
    ) -> Mock<'a> {
        self.server
            .mock_async(move |when, then| {
                when.method(POST)
                    .path("/api/states/sensor.solar_energy")
                    .header("Authorization", format!("Bearer {}", self.token()))
                    .header("Content-Type", "application/json")
                    .json_body(json!({
                        "state": format!("{:.2}", energy_kwh),
                        "attributes": {
                            "device_class": "energy",
                            "state_class": "total_increasing",
                            "last_reset": last_reset,
                            "friendly_name": "Solar Energy",
                            "unit_of_measurement": "kWh"
                        }
                    }));
                then.status(200)
                    .header("content-type", "application/json")
                    .json_body(json!({
                        "entity_id": "sensor.solar_energy",
                        "state": format!("{:.2}", energy_kwh),
                        "attributes": {
                            "friendly_name": "Solar Energy",
                            "unit_of_measurement": "kWh",
                            "last_reset": last_reset,
                            "device_class": "energy",
                            "state_class": "total_increasing"
                        },
                        "last_changed": "2025-06-23T06:15:37.912667+00:00",
                        "last_reported": "2025-06-23T06:15:37.912667+00:00",
                        "last_updated": "2025-06-23T06:15:37.912667+00:00",
                        "context": {
                            "id": "X7TQ47E2AGDK5CWNR3VPYDJP01",
                            "parent_id": null,
                            "user_id": "b7c2e6d3f124c9e5f763a9821576c30"
                        }
                    }));
            })
            .await
    }

    /// Mock the set state for solar power with sample request/response.
    pub async fn mock_set_solar_power<'a>(&'a self, power: i64) -> Mock<'a> {
        self.server
            .mock_async(move |when, then| {
                when.method(POST)
                    .path("/api/states/sensor.solar_power")
                    .header("Authorization", format!("Bearer {}", self.token()))
                    .header("Content-Type", "application/json")
                    .json_body(json!({
                        "state": power.to_string(),
                        "attributes": {
                            "unit_of_measurement": "W",
                            "friendly_name": "Solar Power",
                            "state_class": "measurement"
                        }
                    }));
                then.status(200)
                    .header("content-type", "application/json")
                    .json_body(json!({
                        "entity_id": "sensor.solar_power",
                        "state": power.to_string(),
                        "attributes": {
                            "unit_of_measurement": "W",
                            "state_class": "measurement",
                            "friendly_name": "Solar Power"
                        },
                        "last_changed": "2025-06-23T06:22:32.877327+00:00",
                        "last_reported": "2025-06-23T06:22:32.877327+00:00",
                        "last_updated": "2025-06-23T06:22:32.877327+00:00",
                        "context": {
                            "id": "X7TQ47E2AGDK5CWNR3VPYDJP01",
                            "parent_id": null,
                            "user_id": "b7c2e6d3f124c9e5f763a9821576c30"
                        }
                    }));
            })
            .await
    }

    /// Mock the set state for solar status with sample request/response.
    pub async fn mock_set_solar_status<'a>(&'a self, status: &str) -> Mock<'a> {
        self.server
            .mock_async(move |when, then| {
                when.method(POST)
                    .path("/api/states/sensor.solar_status")
                    .header("Authorization", format!("Bearer {}", self.token()))
                    .header("Content-Type", "application/json")
                    .json_body(json!({
                        "state": status,
                        "attributes": { "friendly_name": "Solar Status" }
                    }));
                then.status(200)
                    .header("content-type", "application/json")
                    .json_body(json!({
                        "entity_id": "sensor.solar_status",
                        "state": status,
                        "attributes": { "friendly_name": "Solar Status" },
                        "last_changed": "2025-06-23T04:07:37.906287+00:00",
                        "last_reported": "2025-06-23T04:07:37.906287+00:00",
                        "last_updated": "2025-06-23T04:07:37.906287+00:00",
                        "context": {
                            "id": "X7TQ47E2AGDK5CWNR3VPYDJP01",
                            "parent_id": null,
                            "user_id": "b7c2e6d3f124c9e5f763a9821576c30"
                        }
                    }));
            })
            .await
    }

    /// Mock a server error on setting solar power to test retry/circuit breaker.
    pub async fn mock_error_solar_power<'a>(&'a self) -> Mock<'a> {
        self.server
            .mock_async(move |when, then| {
                when.method(POST).path("/api/states/sensor.solar_power");
                then.status(500).header("content-type", "application/json");
            })
            .await
    }
}
