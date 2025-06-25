//! Integration tests for the Home Assistant client.
use chrono::TimeZone;
use grelsolar::home_assistant::Client;
use grelsolar::home_assistant::Error;
use httpmock::{Method::POST, MockServer};
use reqwest::Url;
use rstest::*;
use serde_json::json;

#[fixture]
fn token() -> String {
    "test_token".to_string()
}

#[fixture]
async fn server() -> MockServer {
    MockServer::start_async().await
}

#[fixture]
async fn client(token: String, #[future] server: MockServer) -> Client {
    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(log::LevelFilter::Debug)
        .try_init();
    let server = server.await;
    let url = Url::parse(&server.url("")).unwrap();
    Client::new(url, token)
}

#[rstest]
#[tokio::test]
async fn test_client_set_solar_energy(
    token: String,
    #[future] client: Client,
    #[future] server: MockServer,
) {
    let energy_today = 1280;
    let last_reset = chrono::Utc.with_ymd_and_hms(2025, 6, 23, 0, 0, 0).unwrap();
    let client = client.await;
    let server = server.await;
    let mock = server
        .mock_async(|when, then| {
            when.method(POST)
                .path("/api/states/sensor.solar_energy")
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .json_body(json!({
                    "state": "1.28",
                    "attributes": {
                        "device_class": "energy",
                        "state_class": "total_increasing",
                        "last_reset": "2025-06-23T00:00:00+00:00",
                        "friendly_name": "Solar Energy",
                        "unit_of_measurement": "kWh"
                    }
                }));
            then.status(200)
                .header("content-type", "application/json")
                .json_body(json!(
                    {
                        "entity_id": "sensor.solar_energy",
                        "state": "1.28",
                        "attributes": {
                            "friendly_name": "Solar Energy",
                            "unit_of_measurement": "kWh",
                            "last_reset": "2025-06-23T00:00:00+00:00",
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
                    }
                ));
        })
        .await;

    let result = client.set_solar_energy(energy_today, &last_reset).await;

    mock.assert_async().await;
    assert!(result.is_ok());
}

#[rstest]
#[tokio::test]
async fn test_client_set_solar_current_power(
    token: String,
    #[future] client: Client,
    #[future] server: MockServer,
) {
    let power = 701;
    let client = client.await;
    let server = server.await;
    let mock = server
        .mock_async(|when, then| {
            when.method(POST)
                .path("/api/states/sensor.solar_power")
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .json_body(json!({
                    "state": "701",
                    "attributes": {
                        "unit_of_measurement": "W",
                        "friendly_name": "Solar Power",
                        "state_class": "measurement"
                    }
                }));
            then.status(200)
                .header("content-type", "application/json")
                .json_body(json!(
                    {
                        "entity_id": "sensor.solar_power",
                        "state": "701",
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
                    }
                ));
        })
        .await;

    let result = client.set_solar_current_power(power).await;

    mock.assert_async().await;
    assert!(result.is_ok());
}

#[rstest]
#[tokio::test]
async fn test_client_set_solar_status(
    token: String,
    #[future] client: Client,
    #[future] server: MockServer,
) {
    let status = "On-grid";
    let client = client.await;
    let server = server.await;
    let mock = server
        .mock_async(|when, then| {
            when.method(POST)
                .path("/api/states/sensor.solar_status")
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .json_body(json!({
                    "state": "On-grid",
                    "attributes": {
                        "friendly_name": "Solar Status",
                    }
                }));
            then.status(200)
                .header("content-type", "application/json")
                .json_body(json!(
                    {
                        "entity_id": "sensor.solar_status",
                        "state": "On-grid",
                        "attributes": {
                            "friendly_name": "Solar Status",
                        },
                        "last_changed": "2025-06-23T04:07:37.906287+00:00",
                        "last_reported": "2025-06-23T04:07:37.906287+00:00",
                        "last_updated": "2025-06-23T04:07:37.906287+00:00",
                        "context": {
                            "id": "X7TQ47E2AGDK5CWNR3VPYDJP01",
                            "parent_id": null,
                            "user_id": "b7c2e6d3f124c9e5f763a9821576c30"
                        }
                    }
                ));
        })
        .await;

    let result = client.set_solar_status(status).await;

    mock.assert_async().await;
    assert!(result.is_ok());
}

#[rstest]
#[tokio::test]
async fn test_client_reliability_server_error(
    token: String,
    #[future] client: Client,
    #[future] server: MockServer,
) {
    let client = client.await;
    let server = server.await;
    let mock = server
        .mock_async(|when, then| {
            when.method(POST)
                .path("/api/states/sensor.solar_power")
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json");
            then.status(500).header("content-type", "application/json");
        })
        .await;

    let result_call_1 = client.set_solar_current_power(1234).await;
    let result_call_2 = client.set_solar_current_power(1234).await;

    assert!(mock.hits_async().await > 2, "should retry on server error");
    assert!(
        matches!(result_call_1, Err(Error::RequestFailed(_))),
        "request should fail due to server error"
    );
    assert!(
        matches!(result_call_2, Err(Error::RequestRejected)),
        "circuit breaker should reject the request due to repeated failures"
    );
}
