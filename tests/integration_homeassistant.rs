//! Integration tests for the Home Assistant client.
use crate::mockserver_homeassistant::HomeAssistantMockServer;
use chrono::TimeZone;
use grelsolar::integration::homeassistant::{Client, Error};
use rstest::fixture;
use rstest::*;

mod mockserver_homeassistant;

#[fixture]
/// Combined fixture yielding a client and its HomeAssistantMockServer
async fn client_server() -> (Client, HomeAssistantMockServer) {
    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(log::LevelFilter::Debug)
        .try_init();
    let server = HomeAssistantMockServer::start().await;
    let url = server.url();
    let token = server.token().to_string();
    let client = Client::new(url, token);
    (client, server)
}

#[rstest]
#[tokio::test]
async fn test_client_set_solar_energy(#[future] client_server: (Client, HomeAssistantMockServer)) {
    let (client, server) = client_server.await;
    let energy_today = 1280;
    let last_reset = chrono::Utc.with_ymd_and_hms(2025, 6, 23, 0, 0, 0).unwrap();
    let mock = server
        .mock_set_solar_energy((energy_today as f64) / 1000.0, &last_reset)
        .await;

    let result = client.set_solar_energy(energy_today, &last_reset).await;

    mock.assert_async().await;
    assert!(result.is_ok());
}

#[rstest]
#[tokio::test]
async fn test_client_set_solar_current_power(
    #[future] client_server: (Client, HomeAssistantMockServer),
) {
    let (client, server) = client_server.await;
    let power = 701;
    let mock = server.mock_set_solar_power(power).await;

    let result = client.set_solar_current_power(power).await;

    mock.assert_async().await;
    assert!(result.is_ok());
}

#[rstest]
#[tokio::test]
async fn test_client_set_solar_status(#[future] client_server: (Client, HomeAssistantMockServer)) {
    let (client, server) = client_server.await;
    let status = "On-grid";
    let mock = server.mock_set_solar_status(status).await;

    let result = client.set_solar_status(status).await;

    mock.assert_async().await;
    assert!(result.is_ok());
}

#[rstest]
#[tokio::test]
async fn test_client_reliability_server_error(
    #[future] client_server: (Client, HomeAssistantMockServer),
) {
    let (client, server) = client_server.await;
    let mock = server.mock_error_solar_power().await;

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
