// //! Integration tests for the SolarBridgeBackgroundService.
// use std::sync::Arc;
// use chrono::Local;
// use grelsolar::services::solar_bridge::SolarBridgeBackgroundService;
// use grelsolar::solarlog::{Client as SolarLogClient, InverterStatus};
// use grelsolar::homeassistant::Client as HomeAssistantClient;
// use httpmock::{MockServer, Method::POST};
// use reqwest::Url;
// use tokio::time::Duration;
// use rstest::*;

// #[fixture]
// async fn solarlog_server() -> MockServer {
//     MockServer::start_async().await
// }

// #[fixture]
// async fn ha_server() -> MockServer {
//     MockServer::start_async().await
// }

// #[fixture]
// async fn solarlog_client(#[future] solarlog_server: MockServer) -> Arc<SolarLogClient> {
//     let server = solarlog_server.await;
//     let url = Url::parse(&server.url("/")).unwrap();
//     Arc::new(SolarLogClient::new(url, "pw".to_string()))
// }

// #[fixture]
// async fn ha_client(#[future] ha_server: MockServer) -> Arc<HomeAssistantClient> {
//     let server = ha_server.await;
//     let url = Url::parse(&server.url("/")).unwrap();
//     Arc::new(HomeAssistantClient::new(url.clone(), "token".to_string()))
// }

// #[fixture]
// async fn service(
//     #[future] solarlog_client: Arc<SolarLogClient>,
//     #[future] ha_client: Arc<HomeAssistantClient>,
// ) -> SolarBridgeBackgroundService {
//     let solarlog_client = solarlog_client.await;
//     let ha_client = ha_client.await;
//     SolarBridgeBackgroundService::new(
//         solarlog_client,
//         ha_client,
//         Duration::from_secs(1),
//         Duration::from_secs(1),
//         Duration::from_secs(1),
//     )
// }

// #[rstest]
// #[tokio::test]
// async fn test_sync_solar_power(
//     #[future] solarlog_server: MockServer,
//     #[future] ha_server: MockServer,
//     #[future] service: SolarBridgeBackgroundService,
// ) {
//     let solarlog_server = solarlog_server.await;
//     let ha_server = ha_server.await;
//     let service = service.await;
//     let power_mock = solarlog_server.mock_async(|when, then| {
//         when.method(POST).path("/getjp");
//         then.status(200).json_body(serde_json::json!({"782": {"0": 1234}}));
//     }).await;
//     let ha_power_mock = ha_server.mock_async(|when, then| {
//         when.method(POST).path("/api/states/sensor.solar_power");
//         then.status(200).json_body(serde_json::json!({"state": "1234"}));
//     }).await;
//     let result = service.sync_solar_power(None).await;
//     assert_eq!(result, Some(1234));
//     ha_power_mock.assert_async().await;
//     power_mock.assert_async().await;
// }

// #[rstest]
// #[tokio::test]
// async fn test_sync_solar_status(
//     #[future] solarlog_server: MockServer,
//     #[future] ha_server: MockServer,
//     #[future] service: SolarBridgeBackgroundService,
// ) {
//     let solarlog_server = solarlog_server.await;
//     let ha_server = ha_server.await;
//     let service = service.await;
//     let status_mock = solarlog_server.mock_async(|when, then| {
//         when.method(POST).path("/getjp");
//         then.status(200).json_body(serde_json::json!({"608": {"0": "On-grid"}}));
//     }).await;
//     let ha_status_mock = ha_server.mock_async(|when, then| {
//         when.method(POST).path("/api/states/sensor.solar_status");
//         then.status(200).json_body(serde_json::json!({"state": "On-grid"}));
//     }).await;
//     let result = service.sync_solar_status(None).await;
//     assert_eq!(result, Some(InverterStatus::OnGrid));
//     ha_status_mock.assert_async().await;
//     status_mock.assert_async().await;
// }

// #[rstest]
// #[tokio::test]
// async fn test_sync_solar_energy(
//     #[future] solarlog_server: MockServer,
//     #[future] ha_server: MockServer,
//     #[future] service: SolarBridgeBackgroundService,
// ) {
//     let solarlog_server = solarlog_server.await;
//     let ha_server = ha_server.await;
//     let service = service.await;
//     let today = Local::now().date_naive();
//     let today_str = today.format("%d.%m.%y").to_string();
//     let energy_mock = solarlog_server.mock_async(move |when, then| {
//         when.method(POST).path("/getjp");
//         then.status(200).json_body(serde_json::json!({"777": {"0": [[today_str.clone(), [42]]]}}));
//     }).await;
//     let ha_energy_mock = ha_server.mock_async(|when, then| {
//         when.method(POST).path("/api/states/sensor.solar_energy");
//         then.status(200).json_body(serde_json::json!({"state": "0.042"}));
//     }).await;
//     let result = service.sync_solar_energy(today, None).await;
//     assert_eq!(result, Some(42));
//     ha_energy_mock.assert_async().await;
//     energy_mock.assert_async().await;
// }
