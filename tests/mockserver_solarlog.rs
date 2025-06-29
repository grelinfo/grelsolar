//! Mock server for SolarLog API
use chrono::NaiveDate;
use httpmock::{Method::POST, Mock, MockServer};
use reqwest::Url;
use serde_json::json;

pub struct SolarlogMockServer {
    pub server: MockServer,
}

impl SolarlogMockServer {
    /// Create and start a new mock server
    pub async fn start() -> Self {
        let server = MockServer::start_async().await;
        Self { server }
    }

    /// Get url
    pub fn url(&self) -> Url {
        let url = self.server.base_url();
        Url::parse(&url).expect("cannot parse url")
    }

    /// Get password
    pub fn password(&self) -> &str {
        "password"
    }

    /// Mock login success
    pub async fn mock_login_success<'a>(&'a self) -> Mock<'a> {
        self.server
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

    /// Mock login failure
    pub async fn mock_login_failure<'a>(&'a self) -> Mock<'a> {
        self.server
            .mock_async(|when, then| {
                when.method(POST)
                    .path("/login")
                    .header("content-type", "application/x-www-form-urlencoded")
                    .body("u=user&p=password");
                then.status(200)
                    .header("content-type", "text/html")
                    .body("FAILED - Password was wrong");
            })
            .await
    }

    /// Mock logout success
    pub async fn mock_logout_success<'a>(&'a self) -> Mock<'a> {
        self.server
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
            .await
    }

    /// Mock current power
    /// Returns a tuple with the mock and the expected current power value
    pub async fn mock_current_power<'a>(&'a self) -> (Mock<'a>, i64) {
        let mock =
            self.server
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
        (mock, 1234)
    }

    /// Mock query status
    /// Returns a tuple with the mock and the expected status string
    pub async fn mock_query_status<'a>(&'a self) -> (Mock<'a>, &'static str) {
        let mock =
            self.server
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
        (mock, "On-grid")
    }

    /// Mock energy today
    /// Returns a tuple with the mock, the day date, and the expected energy value
    pub async fn mock_energy_daily<'a>(&'a self) -> (Mock<'a>, NaiveDate, i64) {
        let mock =
            self.server
                .mock_async(|when, then| {
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
        let day = NaiveDate::from_ymd_opt(2025, 6, 25).expect("cannot create day date");
        (mock, day, 510)
    }

    /// Mock energy monthly
    /// Returns a tuple with the mock, the month date, and the expected energy value
    pub async fn mock_energy_monhtly<'a>(&'a self) -> (Mock<'a>, NaiveDate, i64) {
        let mock =
            self.server
                .mock_async(|when, then| {
                    when.method(POST)
                .path("/getjp")
                .header(
                    "cookie",
                    "SolarLog=Wazi4Y08JTGY1W56wqPMjMVOa7MxLttaB5n/1Z7NKvg=",
                )
                .body(r#"token=Wazi4Y08JTGY1W56wqPMjMVOa7MxLttaB5n/1Z7NKvg=;{"779":{"0":null}}"#);
                    then.status(200).json_body(json!(
                        {
                            "779": {
                                "0": [["01.06.25", [550370]]]
                            }
                        }
                    ));
                })
                .await;
        let month = NaiveDate::from_ymd_opt(2025, 6, 1).expect("cannot create month date");
        (mock, month, 550370)
    }
}
