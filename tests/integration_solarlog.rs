//! Integration tests for the SolarLog client.
use grelsolar::integration::solarlog::{Client, Error, InverterStatus};
use rstest::{fixture, rstest};

use crate::mockserver_solarlog::SolarlogMockServer;

mod mockserver_solarlog;

#[fixture]
/// Combined fixture yielding both a new client and its mock server
async fn client_server() -> (Client, SolarlogMockServer) {
    let _ = env_logger::builder().is_test(true).try_init();
    let server = SolarlogMockServer::start().await;
    let client = Client::new(server.url(), server.password().to_string());
    (client, server)
}

#[fixture]
/// Combined fixture yielding a logged-in client and its mock server
async fn client_server_logged() -> (Client, SolarlogMockServer) {
    let _ = env_logger::builder().is_test(true).try_init();
    let server = SolarlogMockServer::start().await;
    let client = Client::new(server.url(), server.password().to_string());
    // perform login in fixture
    let _login_mock = server.mock_login_success().await;
    client.login().await.expect("login failed in fixture");
    (client, server)
}

#[rstest]
#[tokio::test]
async fn test_login_success(#[future] client_server: (Client, SolarlogMockServer)) {
    let (client, server) = client_server.await;

    let mock = server.mock_login_success().await;

    let result = client.login().await;

    mock.assert_async().await;
    assert!(result.is_ok());
}

#[rstest]
#[tokio::test]
async fn test_login_failure(#[future] client_server: (Client, SolarlogMockServer)) {
    let (client, server) = client_server.await;

    let mock = server.mock_login_failure().await;

    let result = client.login().await;

    mock.assert_async().await;
    assert!(matches!(result, Err(Error::WrongPassword)));
}

#[rstest]
#[tokio::test]
async fn test_logout(#[future] client_server_logged: (Client, SolarlogMockServer)) {
    let (client, server) = client_server_logged.await;
    let mock = server.mock_logout_success().await;

    let result = client.logout().await;

    assert!(result, "Logout should be successful");
    mock.assert_async().await;
}

#[rstest]
#[tokio::test]
async fn test_get_current_power(#[future] client_server_logged: (Client, SolarlogMockServer)) {
    let (client, server) = client_server_logged.await;
    let (mock, expected) = server.mock_current_power().await;

    let power = client.get_current_power().await;

    mock.assert_async().await;
    assert_eq!(power.expect("failed to get current power"), expected);
}

#[rstest]
#[tokio::test]
async fn test_get_status(#[future] client_server_logged: (Client, SolarlogMockServer)) {
    let (client, server) = client_server_logged.await;
    let (mock, _expected) = server.mock_query_status().await;

    let status = client.get_status().await;

    mock.assert_async().await;
    assert!(matches!(status, Ok(InverterStatus::OnGrid)));
}

#[rstest]
#[tokio::test]
async fn test_get_energy_of_day(#[future] client_server_logged: (Client, SolarlogMockServer)) {
    let (client, server) = client_server_logged.await;
    let (mock, day, expected) = server.mock_energy_daily().await;

    let energy = client.get_energy_of_day(day).await;

    mock.assert_async().await;
    assert_eq!(energy.expect("failed to get energy of month"), expected);
}

#[rstest]
#[tokio::test]
async fn test_get_energy_of_month(#[future] client_server_logged: (Client, SolarlogMockServer)) {
    let (client, server) = client_server_logged.await;
    let (mock, month, expected) = server.mock_energy_monhtly().await;

    let energy = client.get_energy_of_month(month).await;

    mock.assert_async().await;
    assert_eq!(energy.expect("failed to get energy of month"), expected);
}
