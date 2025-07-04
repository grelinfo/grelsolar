//! Integration tests for the SolarBridgeBackgroundService.
use crate::mockserver_homeassistant::HomeAssistantMockServer;
use crate::mockserver_solarlog::SolarlogMockServer;
use grelsolar::integration::homeassistant::Client as HomeAssistantClient;
use grelsolar::integration::solarlog::{self, Client as SolarLogClient};
use grelsolar::services::solarbridge::SolarBridgeBackgroundService;
use std::sync::Arc;
use tokio::time::Duration;

mod mockserver_homeassistant;
mod mockserver_solarlog;

async fn mock_setup() -> (
    SolarlogMockServer,
    HomeAssistantMockServer,
    SolarBridgeBackgroundService,
) {
    let solarlog_mockserver = SolarlogMockServer::start().await;
    let homeassistant_mockserver = HomeAssistantMockServer::start().await;

    let solarlog_client = Arc::new(SolarLogClient::new(
        solarlog_mockserver.url(),
        solarlog_mockserver.password(),
    ));

    let homeassistant_client = Arc::new(HomeAssistantClient::new(
        homeassistant_mockserver.url(),
        homeassistant_mockserver.token(),
    ));

    solarlog_mockserver.mock_login_success().await;
    solarlog_client
        .login()
        .await
        .expect("login failed in fixture");

    let service = SolarBridgeBackgroundService::new(
        solarlog_client,
        homeassistant_client,
        Duration::from_micros(1),
        Duration::from_micros(1),
        Duration::from_micros(1),
    );

    (solarlog_mockserver, homeassistant_mockserver, service)
}

#[tokio::test]
async fn test_sync_solar_power() {
    let (solarlog_mockserver, homeassistant_mockserver, service) = mock_setup().await;
    let (solarlog_mock, expected) = solarlog_mockserver.mock_current_power().await;
    let homeassistant_mock = homeassistant_mockserver
        .mock_set_solar_power(expected)
        .await;

    let result = service.sync_solar_power(None).await;

    solarlog_mock.assert_async().await;
    homeassistant_mock.assert_async().await;
    assert_eq!(result.unwrap(), Some(expected));
}

#[tokio::test]
async fn test_sync_solar_power_no_change() {
    let (solarlog_mockserver, homeassistant_mockserver, service) = mock_setup().await;
    let (solarlog_mock, expected) = solarlog_mockserver.mock_current_power().await;

    let homeassistant_mock = homeassistant_mockserver
        .mock_set_solar_power(expected)
        .await;

    // Second sync should not change anything
    let result = service.sync_solar_power(Some(expected)).await;

    solarlog_mock.assert_async().await;
    assert_eq!(homeassistant_mock.hits_async().await, 0);
    assert_eq!(result.unwrap(), Some(expected));
}

#[tokio::test]
async fn test_sync_solar_status() {
    let (solarlog_mockserver, homeassistant_mockserver, service) = mock_setup().await;
    let (solarlog_mock, expected) = solarlog_mockserver.mock_status().await;
    let homeassistant_mock = homeassistant_mockserver
        .mock_set_solar_status(expected)
        .await;

    let result = service.sync_solar_status(None).await;

    solarlog_mock.assert_async().await;
    homeassistant_mock.assert_async().await;
    assert_eq!(
        result.unwrap().map(|s| s.to_string()),
        Some(expected.to_string())
    );
}

#[tokio::test]
async fn test_sync_solar_status_no_change() {
    let (solarlog_mockserver, homeassistant_mockserver, service) = mock_setup().await;
    let (solarlog_mock, expected) = solarlog_mockserver.mock_status().await;

    let homeassistant_mock = homeassistant_mockserver
        .mock_set_solar_status(expected)
        .await;

    let inverter_status =
        solarlog::InverterStatus::try_from(expected).expect("cannot parse inverter status");
    let result = service.sync_solar_status(Some(&inverter_status)).await;

    solarlog_mock.assert_async().await;
    assert_eq!(homeassistant_mock.hits_async().await, 0);
    assert_eq!(
        result.unwrap().map(|s| s.to_string()),
        Some(expected.to_string())
    );
}

#[tokio::test]
async fn test_sync_solar_energy() {
    let (solarlog_mockserver, homeassistant_mockserver, service) = mock_setup().await;
    let (solarlog_mock, day, expected) = solarlog_mockserver.mock_energy_daily().await;
    let last_reset = SolarBridgeBackgroundService::day_midnight(&day);
    let energy_kwh = (expected as f64) / 1000.0; // Convert to kWh
    let homeassistant_mock = homeassistant_mockserver
        .mock_set_solar_energy(energy_kwh, &last_reset)
        .await;

    let result = service.sync_solar_energy(day, None).await;

    solarlog_mock.assert_async().await;
    homeassistant_mock.assert_async().await;
    assert_eq!(result.unwrap(), Some(expected));
}

#[tokio::test]
async fn test_sync_solar_energy_no_change() {
    let (solarlog_mockserver, homeassistant_mockserver, service) = mock_setup().await;
    let (solarlog_mock, day, expected) = solarlog_mockserver.mock_energy_daily().await;
    let last_reset = SolarBridgeBackgroundService::day_midnight(&day);
    let energy_kwh = (expected as f64) / 1000.0; // Convert to kWh
    let homeassistant_mock = homeassistant_mockserver
        .mock_set_solar_energy(energy_kwh, &last_reset)
        .await;

    // Second sync should not change anything
    let result = service.sync_solar_energy(day, Some(expected)).await;

    solarlog_mock.assert_async().await;
    assert_eq!(homeassistant_mock.hits_async().await, 0);
    assert_eq!(result.unwrap(), Some(expected));
}
