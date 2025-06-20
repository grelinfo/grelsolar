//! Application configuration loaded from environment variables.

use reqwest::Url;
use std::env;
use thiserror::Error;
use tokio::time::Duration;

/// Holds all configuration for the application.
#[derive(Debug, Clone)]
pub struct Config {
    pub app_name: String,
    pub app_version: String,
    pub app_log: String,
    pub app_log_style: String,
    pub solarlog_url: Url,
    pub solarlog_password: String,
    pub home_assistant_url: Url,
    pub home_assistant_token: String,
    pub solar_power_period: Duration,
    pub solar_energy_period: Duration,
    pub solar_status_period: Duration,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Environment variable not found: {0}")]
    EnvVarNotFoundError(String),
    #[error("Failed to parse URL for environment variable: {0}")]
    UrlParseError(String),
    #[error("Failed to parse duration for environment variable: {0}")]
    DurationParseError(String),
}

impl Config {
    /// Creates a new `Config` instance by reading environment variables.
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            // Env var at compile time
            app_name: env!("CARGO_PKG_NAME").to_string(),
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            // Env var at runtime
            app_log: string_from_env_with_default("APP_LOG", "error"),
            app_log_style: string_from_env_with_default("APP_LOG_STYLE", "always"),
            solarlog_url: url_from_env("SOLARLOG_URL")?,
            solarlog_password: string_from_env("SOLARLOG_PASSWORD")?,
            home_assistant_url: url_from_env("HOME_ASSISTANT_URL")?,
            home_assistant_token: string_from_env("HOME_ASSISTANT_TOKEN")?,
            solar_power_period: duration_from_env_with_default("SOLAR_POWER_PERIOD_SEC", 5)?,
            solar_energy_period: duration_from_env_with_default("SOLAR_ENERGY_PERIOD_SEC", 60)?,
            solar_status_period: duration_from_env_with_default("SOLAR_STATUS_PERIOD_SEC", 60)?,
        })
    }
}

pub fn configure_logger() {
    let env = env_logger::Env::default()
        .filter_or("APP_LOG", "info")
        .write_style_or("APP_LOG_STYLE", "always");
    env_logger::init_from_env(env);
}

fn string_from_env_with_default(name: &str, default: &str) -> String {
    env::var(name).unwrap_or_else(|_| default.to_string())
}

fn string_from_env(name: &str) -> Result<String, ConfigError> {
    let value = env::var(name).map_err(|_| ConfigError::EnvVarNotFoundError(name.to_string()))?;
    if value.trim().is_empty() {
        Err(ConfigError::EnvVarNotFoundError(name.to_string()))
    } else {
        Ok(value)
    }
}

fn url_from_env(name: &str) -> Result<Url, ConfigError> {
    let value = string_from_env(name)?;
    Url::parse(&value).map_err(|_| ConfigError::UrlParseError(name.to_string()))
}

fn duration_from_env_with_default(name: &str, default: i64) -> Result<Duration, ConfigError> {
    env::var(name)
        .unwrap_or_else(|_| default.to_string())
        .parse::<u64>()
        .map(Duration::from_secs)
        .map_err(|_| ConfigError::DurationParseError(name.to_string()))
}
