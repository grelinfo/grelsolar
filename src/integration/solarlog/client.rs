//! SolarLog Client.
//! This client is the higher level API client for SolarLog.
use super::http_client::HttpClient;
use super::{Error, Result};
use chrono::NaiveDate;
use reqwest::Url;
use serde_json::Value;
use serde_json::Value::Null;
use serde_json::json;
use std::str::FromStr;
use strum_macros::Display;
use strum_macros::EnumString;

pub struct Client {
    http: HttpClient,
}

static CURRENT_POWER: &str = "782";
static DAILY_ENERGY: &str = "777";
static MONTHLY_ENERGY: &str = "779";
static STATUS: &str = "608";

/// Solar-Log inverter status.
#[derive(Debug, PartialEq, EnumString, Display, Clone)]
pub enum InverterStatus {
    #[strum(serialize = "Idle Initializing")]
    IdleInitializing,
    #[strum(serialize = "Idle  Detecting ISO")]
    IdleDetectingIso,
    #[strum(serialize = "Idle Detecting irradiation")]
    IdleDetectingIrradiation,
    #[strum(serialize = "Idle Grid detecting")]
    IdleGridDetecting,
    #[strum(serialize = "Idle No irradiation")]
    IdleNoIrradiation,
    #[strum(serialize = "Starting")]
    Starting,
    #[strum(serialize = "On-grid")]
    OnGrid,
    #[strum(serialize = "On-grid Power limit")]
    OnGridPowerLimit,
    #[strum(serialize = "On-grid self derating")]
    OnGridSelfDerating,
    #[strum(serialize = "Grid dispatch cos(Phi)-P curve")]
    GridDispatchCosPhiPCurve,
    #[strum(serialize = "Grid dispatch QU curve")]
    GridDispatchQuCurve,
    #[strum(serialize = "Shutdown Fault")]
    ShutdownFault,
    #[strum(serialize = "Shutdown Command")]
    ShutdownCommand,
    #[strum(serialize = "Shutdown OVGR")]
    ShutdownOvgr,
    #[strum(serialize = "Shutdown Communication disconnected")]
    ShutdownCommDisconnected,
    #[strum(serialize = "Shutdown Power limit")]
    ShutdownPowerLimit,
    #[strum(serialize = "Shutdown Start manually")]
    ShutdownStartManually,
    #[strum(serialize = "Shutdown DC switch OFF")]
    ShutdownDcSwitchOff,
    #[strum(serialize = "Spot-check")]
    SpotCheck,
    #[strum(serialize = "Spot-checking")]
    SpotChecking,
    #[strum(serialize = "Inspecting")]
    Inspecting,
    #[strum(serialize = "AFCI self-check")]
    AfciSelfCheck,
    #[strum(serialize = "IV scanning")]
    IvScanning,
    #[strum(serialize = "DC input detection")]
    DcInputDetection,
}

impl InverterStatus {
    /// Returns `true` if the inverter status is shutting down.
    pub fn is_shutdown(&self) -> bool {
        matches!(
            self,
            InverterStatus::ShutdownFault
                | InverterStatus::ShutdownCommand
                | InverterStatus::ShutdownOvgr
                | InverterStatus::ShutdownCommDisconnected
                | InverterStatus::ShutdownPowerLimit
                | InverterStatus::ShutdownStartManually
                | InverterStatus::ShutdownDcSwitchOff
        )
    }
    /// Returns `true` if the inverter status is idle.
    pub fn is_idle(&self) -> bool {
        matches!(
            self,
            InverterStatus::IdleInitializing
                | InverterStatus::IdleDetectingIso
                | InverterStatus::IdleDetectingIrradiation
                | InverterStatus::IdleGridDetecting
                | InverterStatus::IdleNoIrradiation
        )
    }
    /// Returns `true` if the inverter status is producing power.
    pub fn is_on_grid(&self) -> bool {
        matches!(
            self,
            InverterStatus::OnGrid
                | InverterStatus::OnGridPowerLimit
                | InverterStatus::OnGridSelfDerating
        )
    }
}

impl Client {
    /// Creates a new instance of `Client`.
    pub fn new(url: Url, password: String) -> Self {
        let inner = HttpClient::new(url, password);
        Client { http: inner }
    }

    /// Login to SolarLog device.
    /// No operation is performed if already logged in.
    pub async fn login(&self) -> Result<()> {
        self.http.login(false).await?;
        Ok(())
    }

    pub async fn is_logged_in(&self) -> bool {
        self.http.is_logged_in().await
    }

    /// Logout from SolarLog device.
    /// Return `true` if logout was successful, `false` otherwise.
    pub async fn logout(&self) -> bool {
        self.http.logout().await
    }

    /// Get the power produced or consumed in Watt (W).
    pub async fn get_current_power(&self) -> Result<i64> {
        let query = Self::create_inverter_query(CURRENT_POWER, 0);
        let json_value = self.http.query(&query).await?;
        Self::extract_inverter_value_as_i64(&json_value, CURRENT_POWER, 0)
    }

    /// Get the inverter status.
    pub async fn get_status(&self) -> Result<InverterStatus> {
        let query = Self::create_inverter_query(STATUS, 0);
        let json_value = self.http.query(&query).await?;
        Self::extract_inverter_status(&json_value)
    }

    /// Get the energy produced or consumed during the current day in watt-hours (Wh).
    pub async fn get_energy_of_day(&self, day: NaiveDate) -> Result<i64> {
        let query = Self::create_inverter_query(DAILY_ENERGY, 0);
        let json_value = self.http.query(&query).await?;
        Self::extract_energy_of_day(&json_value, day)
    }

    /// Get the energy produced or consumed during the current month in watt-hours (Wh).
    pub async fn get_energy_of_month(&self, month: NaiveDate) -> Result<i64> {
        let query = Self::create_inverter_query(MONTHLY_ENERGY, 0);
        let json_value = self.http.query(&query).await?;
        Self::extract_energy_of_month(&json_value, month)
    }

    /// Get the value for a specific inverter ID and key as a string.
    fn create_inverter_query(index: &str, inverter_id: u8) -> String {
        json!({ index: { inverter_id.to_string(): Null } }).to_string()
    }

    /// Extract the energy for the current day.
    fn extract_energy_of_day(json_value: &Value, day: NaiveDate) -> Result<i64> {
        let day_string = day.format("%d.%m.%y").to_string();
        Self::extract_inverter_value_by_id_as_i64(json_value, DAILY_ENERGY, 0, &day_string)
    }

    /// Extract the energy for the current month.
    fn extract_energy_of_month(json_value: &Value, month: NaiveDate) -> Result<i64> {
        let month_string = month.format("01.%m.%y").to_string();
        Self::extract_inverter_value_by_id_as_i64(json_value, MONTHLY_ENERGY, 0, &month_string)
    }

    /// Extract the status of the first inverter as a enum.
    pub fn extract_inverter_status(json_value: &Value) -> Result<InverterStatus> {
        let status_str = Self::extract_inverter_value_as_string(json_value, STATUS, 0)?;
        InverterStatus::from_str(status_str)
            .map_err(|_| Error::ValueParseError(format!("Invalid inverter status: {status_str}")))
    }

    /// Extract the value for a specific inverter ID and index as i64.
    fn extract_inverter_value_as_i64(
        json_value: &Value,
        index: &str,
        inverter_id: u8,
    ) -> Result<i64> {
        let value = json_value[index][inverter_id.to_string()]
            .as_str()
            .ok_or_else(|| {
                Error::ValueParseError(format!(
                    "Missing value for index {index} and inverter {inverter_id}"
                ))
            })?
            .parse::<i64>()
            .map_err(|_| {
                Error::ValueParseError(format!(
                    "Invalid i64 value for index {index} and inverter {inverter_id}"
                ))
            })?;
        Ok(value)
    }

    /// Extract the value for a specific inverter ID and index as a string.
    fn extract_inverter_value_as_string<'a>(
        json_value: &'a Value,
        index: &str,
        inverter_id: u8,
    ) -> Result<&'a str> {
        json_value
            .get(index)
            .and_then(|v| v.get(inverter_id.to_string()))
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                Error::ValueParseError(format!(
                    "Missing string value for index {index} and inverter {inverter_id}"
                ))
            })
    }

    /// Extract the value for a specific inverter ID and id as an i64.
    fn extract_inverter_value_by_id_as_i64(
        json_value: &Value,
        index: &str,
        inverter_id: u8,
        id: &str,
    ) -> Result<i64> {
        let value = json_value
            .get(index)
            .and_then(|v| v.get(inverter_id.to_string()))
            .and_then(|v| v.as_array())
            .and_then(|arr| {
                arr.iter().find_map(|element| {
                    let element_id = element.get(0)?.as_str()?;
                    let element_value = element.get(1)?.as_array()?.first()?.as_i64()?;
                    (element_id == id).then_some(element_value)
                })
            })
            .ok_or(Error::ValueParseError(format!(
                "Missing value for index {index}, inverter {inverter_id}, id {id}"
            )))?;
        Ok(value)
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inverter_status_is_shutdown() {
        let shutdown_statuses = [
            InverterStatus::ShutdownFault,
            InverterStatus::ShutdownCommand,
            InverterStatus::ShutdownOvgr,
            InverterStatus::ShutdownCommDisconnected,
            InverterStatus::ShutdownPowerLimit,
            InverterStatus::ShutdownStartManually,
            InverterStatus::ShutdownDcSwitchOff,
        ];
        for status in shutdown_statuses.iter() {
            assert!(status.is_shutdown());
        }
        let not_shutdown_statuses = [
            InverterStatus::IdleInitializing,
            InverterStatus::OnGrid,
            InverterStatus::SpotCheck,
        ];
        for status in not_shutdown_statuses.iter() {
            assert!(!status.is_shutdown());
        }
    }

    #[test]
    fn test_inverter_status_is_idle() {
        let idle_statuses = [
            InverterStatus::IdleInitializing,
            InverterStatus::IdleDetectingIso,
            InverterStatus::IdleDetectingIrradiation,
            InverterStatus::IdleGridDetecting,
            InverterStatus::IdleNoIrradiation,
        ];
        for status in idle_statuses.iter() {
            assert!(status.is_idle());
        }
        let not_idle_statuses = [
            InverterStatus::OnGrid,
            InverterStatus::ShutdownFault,
            InverterStatus::SpotCheck,
        ];
        for status in not_idle_statuses.iter() {
            assert!(!status.is_idle());
        }
    }

    #[test]
    fn test_inverter_status_is_on_grid() {
        let on_grid_statuses = [
            InverterStatus::OnGrid,
            InverterStatus::OnGridPowerLimit,
            InverterStatus::OnGridSelfDerating,
        ];
        for status in on_grid_statuses.iter() {
            assert!(status.is_on_grid());
        }
        let not_on_grid_statuses = [
            InverterStatus::IdleInitializing,
            InverterStatus::ShutdownFault,
            InverterStatus::SpotCheck,
        ];
        for status in not_on_grid_statuses.iter() {
            assert!(!status.is_on_grid());
        }
    }

    #[test]
    fn test_inverter_status_from_str_and_display() {
        let pairs = [
            ("Idle Initializing", InverterStatus::IdleInitializing),
            ("On-grid", InverterStatus::OnGrid),
            ("Shutdown Fault", InverterStatus::ShutdownFault),
            ("Spot-check", InverterStatus::SpotCheck),
            ("AFCI self-check", InverterStatus::AfciSelfCheck),
        ];
        for (s, expected) in pairs.iter() {
            let parsed = InverterStatus::from_str(s).unwrap();
            assert_eq!(&parsed, expected);
            assert_eq!(parsed.to_string(), *s);
        }
    }

    #[test]
    fn test_inverter_status_from_str_invalid() {
        assert!(InverterStatus::from_str("Not a status").is_err());
    }

    #[test]
    fn test_client_new() {
        let url = Url::parse("http://localhost:8080").unwrap();
        let password = String::from("test_password");
        Client::new(url, password);
    }

    #[test]
    fn test_create_inverter_query() {
        let index = "777";
        let inverter_id = 1u8;
        let query = Client::create_inverter_query(index, inverter_id);
        // Should produce a JSON string like: {"777":{"1":null}}
        let expected =
            serde_json::json!({ index: { inverter_id.to_string(): serde_json::Value::Null } })
                .to_string();
        assert_eq!(query, expected);
    }

    #[test]
    fn test_extract_inverter_value_as_i64() {
        let json = serde_json::json!({"777": {"0": "12345"}});
        let val = Client::extract_inverter_value_as_i64(&json, "777", 0).unwrap();
        assert_eq!(val, 12345);
        let missing = Client::extract_inverter_value_as_i64(&json, "999", 0);
        assert!(missing.is_err());
    }

    #[test]
    fn test_extract_inverter_value_as_string_with_value() {
        let json = serde_json::json!({"608": {"0": "On-grid"}});
        let val = Client::extract_inverter_value_as_string(&json, "608", 0).unwrap();
        assert_eq!(val, "On-grid");
    }

    #[test]
    fn test_extract_inverter_value_as_string_missing_value() {
        let json = serde_json::json!({"608": {"0": "On-grid"}});
        let missing = Client::extract_inverter_value_as_string(&json, "999", 0);
        assert!(matches!(
            missing,
            Err(Error::ValueParseError(msg)) if msg.contains("Missing string value for index 999 and inverter 0")
        ));
    }

    #[test]
    fn test_extract_energy_for_current_day() {
        let day = NaiveDate::from_ymd_opt(2025, 6, 25).expect("Invalid date");
        let json = serde_json::json!(
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
        );
        let result = Client::extract_energy_of_day(&json, day).expect("cannot extract energy");
        assert_eq!(result, 510);
    }

    #[test]
    fn test_extract_energy_for_current_month() {
        let month = NaiveDate::from_ymd_opt(2025, 6, 1).expect("cannot create month date");
        let json = serde_json::json!(
            {
                "779": {
                    "0": [["01.06.25", [550370]]]
                }
            }
        );
        let month =
            Client::extract_energy_of_month(&json, month).expect("cannot extract month energy");
        assert_eq!(month, 550370);
    }

    #[test]
    fn test_extract_inverter_status() {
        // Valid status
        let json = serde_json::json!({"608": {"0": "On-grid"}});
        let status = Client::extract_inverter_status(&json).unwrap();
        assert_eq!(status, InverterStatus::OnGrid);

        // Invalid status string
        let json = serde_json::json!({"608": {"0": "Not a status"}});
        let status = Client::extract_inverter_status(&json);
        assert!(matches!(
            status,
            Err(Error::ValueParseError(msg)) if msg.contains("Invalid inverter status")
        ));

        // Missing status
        let json = serde_json::json!({"999": {"0": "On-grid"}});
        let status = Client::extract_inverter_status(&json);
        assert!(matches!(
            status,
            Err(Error::ValueParseError(msg)) if msg.contains("Missing string value for index 608 and inverter 0")
        ));
    }
}
