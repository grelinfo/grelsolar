//! Integration tests for the SolarLog client.
use chrono::NaiveDate;
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
    Client::new(url, String::from("password"))
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
            when.method(POST)
                .path("/getjp")
                .header(
                    "cookie",
                    "SolarLog=Wazi4Y08JTGY1W56wqPMjMVOa7MxLttaB5n/1Z7NKvg=",
                )
                .body(r#"token=Wazi4Y08JTGY1W56wqPMjMVOa7MxLttaB5n/1Z7NKvg=;{"782":{"0":null}}"#);
            then.status(200).body(r#"{"782":{"0":"1234"}}"#);
        })
        .await;
    let power = client.get_current_power().await;
    mock.assert_async().await;
    assert_eq!(power.unwrap(), Some(1234));
}

#[rstest]
#[tokio::test]
async fn test_get_status(#[future] client: Client, #[future] server: MockServer) {
    let client = client.await;
    let server = server.await;
    mock_login(&server).await;
    let mock = server
        .mock_async(|when, then| {
            when.method(POST)
                .path("/getjp")
                .header(
                    "cookie",
                    "SolarLog=Wazi4Y08JTGY1W56wqPMjMVOa7MxLttaB5n/1Z7NKvg=",
                )
                .body(r#"token=Wazi4Y08JTGY1W56wqPMjMVOa7MxLttaB5n/1Z7NKvg=;{"608":{"0":null}}"#);
            then.status(200).body(r#"{"608":{"0":"On-grid"}}"#);
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
    let day = NaiveDate::from_ymd_opt(2025, 6, 25).expect("cannot create day date");
    mock_login(&server).await;
    let mock = server
        .mock_async(move |when, then| {
            when.method(POST)
                .path("/getjp")
                .header(
                    "cookie",
                    "SolarLog=Wazi4Y08JTGY1W56wqPMjMVOa7MxLttaB5n/1Z7NKvg=",
                )
                .body(r#"token=Wazi4Y08JTGY1W56wqPMjMVOa7MxLttaB5n/1Z7NKvg=;{"777":{"0":null}}"#);
            then.status(200).json_body(serde_json::json!(
                {
                    "777": {
                        "0": [
                        ["01.06.25", [21700]],
                        ["02.06.25", [9550]],
                        ["03.06.25", [23300]],
                        ["04.06.25", [10790]],
                        ["05.06.25", [18550]],
                        ["06.06.25", [16720]],
                        ["07.06.25", [11040]],
                        ["08.06.25", [22760]],
                        ["09.06.25", [27600]],
                        ["10.06.25", [25550]],
                        ["11.06.25", [27330]],
                        ["12.06.25", [27250]],
                        ["13.06.25", [26890]],
                        ["14.06.25", [26300]],
                        ["15.06.25", [20500]],
                        ["16.06.25", [26360]],
                        ["17.06.25", [28800]],
                        ["18.06.25", [27390]],
                        ["19.06.25", [27540]],
                        ["20.06.25", [27560]],
                        ["21.06.25", [18850]],
                        ["22.06.25", [27870]],
                        ["23.06.25", [21030]],
                        ["24.06.25", [28430]],
                        ["25.06.25", [510]]
                        ]
                    }
                }
            ));
        })
        .await;
    let energy = client
        .get_energy_of_day(day)
        .await
        .expect("cannot get energy");
    mock.assert_async().await;
    assert_eq!(energy, Some(510));
}

#[rstest]
#[tokio::test]
async fn test_get_energy_month(#[future] client: Client, #[future] server: MockServer) {
    let client = client.await;
    let server = server.await;
    let month = NaiveDate::from_ymd_opt(2025, 6, 1).expect("cannot create month date");
    mock_login(&server).await;
    let mock = server
        .mock_async(move |when, then| {
            when.method(POST)
                .path("/getjp")
                .header(
                    "cookie",
                    "SolarLog=Wazi4Y08JTGY1W56wqPMjMVOa7MxLttaB5n/1Z7NKvg=",
                )
                .body(r#"token=Wazi4Y08JTGY1W56wqPMjMVOa7MxLttaB5n/1Z7NKvg=;{"779":{"0":null}}"#);
            then.status(200).json_body(serde_json::json!(
                {
                    "779": {
                        "0": [["01.06.25", [550370]]]
                    }
                }
            ));
        })
        .await;
    let energy = client
        .get_energy_of_month(month)
        .await
        .expect("cannot get energy");
    mock.assert_async().await;
    assert_eq!(energy, Some(550370));
}
