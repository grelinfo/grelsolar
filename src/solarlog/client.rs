//! SolarLog Client.
//! This client is the higher level API client for SolarLog.
use super::http_client::{self, HttpClient};
use reqwest::Url;
use serde_json::Value::Null;
use serde_json::json;
use std::str::FromStr;
use strum_macros::Display;
use strum_macros::EnumString;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("HTTP client error: {0}")]
    HttpClientError(#[from] http_client::Error),
    #[error("JSON serialization error: {0}")]
    JsonSerializationError(#[from] serde_json::Error),
}

type Result<T> = std::result::Result<T, Error>;

pub struct Client {
    http: HttpClient,
}

static CURRENT_POWER: &str = "782";
static DAILY_ENERGY: &str = "777";
static MONTHLY_ENERGY: &str = "779";
static STATUS: &str = "608";

/// Solar-Log inverter status.
#[derive(Debug, PartialEq, EnumString, Display)]
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
    pub fn new(url: &Url, password: &str) -> Self {
        let inner = HttpClient::new(url, password);
        Client { http: inner }
    }

    /// Login to SolarLog device.
    /// No operation is performed if already logged in.
    pub async fn login(&self) -> Result<()> {
        self.http.login(false).await?;
        Ok(())
    }

    /// Logout from SolarLog device.
    pub async fn logout(&self) {
        self.http.logout().await;
    }

    /// Get the power produced or consumed in Watt (W).
    pub async fn get_current_power(&self) -> Result<Option<i64>> {
        let value = self
            .get_inverter_value_as_str(CURRENT_POWER, 0)
            .await?
            .and_then(|s| s.parse::<i64>().ok());
        Ok(value)
    }

    /// Get the inverter status.
    pub async fn get_status(&self) -> Result<Option<InverterStatus>> {
        let value = self
            .get_inverter_value_as_str(STATUS, 0)
            .await?
            .and_then(|s| InverterStatus::from_str(&s).ok());
        Ok(value)
    }

    /// Get the energy produced or consumed during the current day in watt-hours (Wh).
    pub async fn get_energy_today(&self) -> Result<Option<i64>> {
        let today = chrono::Local::now().format("%d.%m.%y").to_string();
        self.get_inverter_date_value_as_i64(DAILY_ENERGY, 0, &today)
            .await
    }

    /// Get the energy produced or consumed during the current month in watt-hours (Wh).
    pub async fn get_energy_month(&self) -> Result<Option<i64>> {
        let first_day_of_this_month = chrono::Local::now().format("01.%m.%y").to_string();
        self.get_inverter_date_value_as_i64(MONTHLY_ENERGY, 0, &first_day_of_this_month)
            .await
    }

    /// Get the SolarLog device for a specific key and inverter ID, returning the value as a string.
    async fn get_inverter_value_as_str(
        &self,
        key: &str,
        inverter_id: u8,
    ) -> Result<Option<String>> {
        let query = json!({ key: { inverter_id.to_string(): Null } });
        let json_str = self.http.query(&query.to_string()).await?;
        let value = serde_json::from_str::<serde_json::Value>(&json_str)?
            .get(key)
            .and_then(|v| v.get(inverter_id.to_string()))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        Ok(value)
    }

    /// Get the SolarLog device for a specific key and inverter ID, returning the value as an i32 for a specific date.
    async fn get_inverter_date_value_as_i64(
        &self,
        key: &str,
        inverter_id: u8,
        date_str: &str,
    ) -> Result<Option<i64>> {
        let query = json!({ key: { inverter_id.to_string(): Null } });
        let json_str = self.http.query(&query.to_string()).await?;
        let value = serde_json::from_str::<serde_json::Value>(&json_str)?
            .get(key)
            .and_then(|v| v.get(inverter_id.to_string()))
            .and_then(|v| v.as_array())
            .and_then(|arr| {
                arr.iter().find_map(|entry| {
                    match (
                        entry.get(0)?.as_str(),
                        entry.get(1)?.as_array()?.first()?.as_i64(),
                    ) {
                        (Some(date), Some(val)) if date == date_str => Some(val),
                        _ => None,
                    }
                })
            });
        Ok(value)
    }
}
