//! Integration tests for the SolarLog client.
use grelsolar::solarlog::{Client, InverterStatus};
use httpmock::{Mock, prelude::*};
use reqwest::Url;
use rstest::*;

#[fixture]
async fn server() -> MockServer {
    MockServer::start_async().await
}

#[fixture]
async fn client(#[future] server: MockServer) -> Client {
    let server = server.await;
    let url = Url::parse(&server.url("/")).unwrap();
    Client::new(&url, "password")
}

async fn mock_login<'a>(server: &'a MockServer) -> Mock<'a> {
    server
        .mock_async(|when, then| {
            when.method(POST)
                .path("/login")
                .header("content-type", "application/x-www-form-urlencoded")
                .body("u=user&p=password");
            then.status(200)
                .header(
                    "set-cookie",
                    "SolarLog=Wazi4Y08JTGY1W56wqPMjMVOa7MxLttaB5n/1Z7NKvg=",
                )
                .header("content-type", "text/html")
                .body("SUCCESS - Password was correct, you are now logged in");
        })
        .await
}

#[rstest]
#[tokio::test]
async fn test_login(#[future] client: Client, #[future] server: MockServer) {
    let client = client.await;
    let server = server.await;
    let mock = mock_login(&server).await;
    let result = client.login().await;
    mock.assert_async().await;
    assert!(result.is_ok());
}

#[rstest]
#[tokio::test]
async fn test_logout(#[future] client: Client, #[future] server: MockServer) {
    let client = client.await;
    let server = server.await;
    mock_login(&server).await;
    let mock = server
        .mock_async(|when, then| {
            when.method(POST).path("/logout").header(
                "cookie",
                "SolarLog=Wazi4Y08JTGY1W56wqPMjMVOa7MxLttaB5n/1Z7NKvg=",
            );
            then.status(200)
                .header("set-cookie", "SolarLog=")
                .header("content-type", "text/html")
                .body("SUCESS - You are now logged out."); // Typos in solarLog API responses
        })
        .await;
    client.login().await.unwrap();
    client.logout().await;
    mock.assert_async().await;
}

#[rstest]
#[tokio::test]
async fn test_get_current_power(#[future] client: Client, #[future] server: MockServer) {
    let client = client.await;
    let server = server.await;
    mock_login(&server).await;
    let mock = server
        .mock_async(|when, then| {
            when.method(POST).path("/getjp");
            then.status(200)
                .json_body(serde_json::json!({"782": {"0": 1234}}));
        })
        .await;
    let power = client.get_current_power().await.unwrap();
    mock.assert_async().await;
    assert_eq!(power, Some(1234));
}

#[rstest]
#[tokio::test]
async fn test_get_status(#[future] client: Client, #[future] server: MockServer) {
    let client = client.await;
    let server = server.await;
    mock_login(&server).await;
    let mock = server
        .mock_async(|when, then| {
            when.method(POST).path("/getjp");
            then.status(200)
                .json_body(serde_json::json!({"608": {"0": "On-grid"}}));
        })
        .await;
    let status = client.get_status().await.unwrap();
    mock.assert_async().await;
    assert_eq!(status, Some(InverterStatus::OnGrid));
}

#[rstest]
#[tokio::test]
async fn test_get_energy_today(#[future] client: Client, #[future] server: MockServer) {
    let client = client.await;
    let server = server.await;
    let today = chrono::Local::now().format("%d.%m.%y").to_string();
    mock_login(&server).await;
    let mock = server
        .mock_async(move |when, then| {
            when.method(POST).path("/getjp");
            then.status(200)
                .json_body(serde_json::json!({"777": {"0": [[today.clone(), [42]]]}}));
        })
        .await;
    let energy = client.get_energy_today().await.unwrap();
    mock.assert_async().await;
    assert_eq!(energy, Some(42));
}

#[rstest]
#[tokio::test]
async fn test_get_energy_month(#[future] client: Client, #[future] server: MockServer) {
    let client = client.await;
    let server = server.await;
    let first_day = chrono::Local::now().format("01.%m.%y").to_string();
    mock_login(&server).await;
    let mock = server
        .mock_async(move |when, then| {
            when.method(POST).path("/getjp");
            then.status(200)
                .json_body(serde_json::json!({"779": {"0": [[first_day.clone(), [99]]]}}));
        })
        .await;
    let energy = client.get_energy_month().await.unwrap();
    mock.assert_async().await;
    assert_eq!(energy, Some(99));
}
