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

    server.mock_login_ok().await;
    client.login().await.expect("login failed in fixture");

    (client, server)
}

#[rstest]
#[tokio::test]
async fn test_login_ok(#[future] client_server: (Client, SolarlogMockServer)) {
    let (client, server) = client_server.await;

    let mock = server.mock_login_ok().await;

    let result = client.login().await;

    mock.assert_async().await;
    assert!(result.is_ok());
}

#[rstest]
#[tokio::test]
async fn test_login_with_wrong_password(#[future] client_server: (Client, SolarlogMockServer)) {
    let (client, server) = client_server.await;

    let mock = server.mock_login_with_wrong_password().await;

    let result = client.login().await;

    mock.assert_async().await;
    assert!(matches!(result, Err(Error::WrongPassword)));
}

#[rstest]
#[tokio::test]
async fn test_login_with_server_error(#[future] client_server: (Client, SolarlogMockServer)) {
    let (client, server) = client_server.await;
    let mock = server.mock_login_with_server_error().await;

    let result_1 = client.login().await;
    let result_2 = client.login().await;

    assert!(mock.hits_async().await > 2, "should retry on server error");
    assert!(matches!(result_1, Err(Error::RequestFailed(_))));
    assert!(
        matches!(result_2, Err(Error::RequestRejected)),
        "second login should be rejected by circuit breaker"
    );
}

#[rstest]
#[tokio::test]
async fn test_logout(#[future] client_server_logged: (Client, SolarlogMockServer)) {
    let (client, server) = client_server_logged.await;
    let mock = server.mock_logout_ok().await;

    let result = client.logout().await;

    assert!(result, "Logout should be successful");
    mock.assert_async().await;
}

#[rstest]
#[tokio::test]
async fn test_logout_with_server_error(
    #[future] client_server_logged: (Client, SolarlogMockServer),
) {
    let (client, server) = client_server_logged.await;
    let mock = server.mock_logout_with_server_error().await;

    let result = client.logout().await;

    assert_eq!(mock.hits_async().await, 1, "should try to logout once");
    assert!(!result, "logout should fail due to server error");
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
    let (mock, _expected) = server.mock_status().await;

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

#[rstest]
#[tokio::test]
async fn test_is_logged_in_true(#[future] client_server_logged: (Client, SolarlogMockServer)) {
    let (client, _server) = client_server_logged.await;

    let logged_in = client.is_logged_in().await;

    assert!(logged_in);
}

#[rstest]
#[tokio::test]
async fn test_is_logged_in_false(#[future] client_server: (Client, SolarlogMockServer)) {
    let (client, _server) = client_server.await;

    let logged_in = client.is_logged_in().await;

    assert!(!logged_in);
}

#[rstest]
#[tokio::test]
async fn test_logout_without_login(#[future] client_server: (Client, SolarlogMockServer)) {
    let (client, server) = client_server.await;
    let mock = server.mock_logout_ok().await;

    let result = client.logout().await;

    assert!(!result);
    assert_eq!(mock.hits_async().await, 0);
}

#[rstest]
#[tokio::test]
async fn test_client_with_server_error(
    #[future] client_server_logged: (Client, SolarlogMockServer),
) {
    let (client, server) = client_server_logged.await;
    let mock = server.mock_query_server_error().await;

    let result_call_1 = client.get_current_power().await;
    let result_call_2 = client.get_current_power().await;

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

#[rstest]
#[tokio::test]
async fn test_query_with_query_impossible(
    #[future] client_server_logged: (Client, SolarlogMockServer),
) {
    let (client, server) = client_server_logged.await;
    let mock = server.mock_query_impossible().await;

    let result_call_1 = client.get_current_power().await;
    let result_call_2 = client.get_current_power().await;

    assert_eq!(
        mock.hits_async().await,
        2,
        "should not retry on impossible query"
    );
    assert!(
        matches!(result_call_1, Err(Error::QueryImpossible)),
        "request should fail due to impossible query"
    );
    assert!(
        matches!(result_call_2, Err(Error::QueryImpossible)),
        "circuit breaker should not reject the request for impossible query"
    );
}

#[rstest]
#[tokio::test]
async fn test_query_with_access_denied(
    #[future] client_server_logged: (Client, SolarlogMockServer),
) {
    let (client, server) = client_server_logged.await;
    let mock = server.mock_query_access_denied().await;

    let result_call_1 = client.get_current_power().await;
    let result_call_2 = client.get_current_power().await;

    assert!(mock.hits_async().await > 2, "should retry on access denied");
    assert!(
        matches!(result_call_1, Err(Error::AccessDenied)),
        "request should fail due to access denied"
    );
    assert!(
        matches!(result_call_2, Err(Error::AccessDenied)),
        "circuit breaker should not reject the request for access denied"
    );
}
