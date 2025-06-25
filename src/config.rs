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
            app_name: env!("CARGO_PKG_NAME").to_string(),
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            app_log: Env::var("APP_LOG").or("error").as_string()?,
            app_log_style: Env::var("APP_LOG_STYLE").or("always").as_string()?,
            solarlog_url: Env::var("SOLARLOG_URL").as_url()?,
            solarlog_password: Env::var("SOLARLOG_PASSWORD").as_string()?,
            home_assistant_url: Env::var("HOME_ASSISTANT_URL").as_url()?,
            home_assistant_token: Env::var("HOME_ASSISTANT_TOKEN").as_string()?,
            solar_power_period: Env::var("SOLAR_POWER_PERIOD_SEC").or("5").as_duration()?,
            solar_energy_period: Env::var("SOLAR_ENERGY_PERIOD_SEC").or("60").as_duration()?,
            solar_status_period: Env::var("SOLAR_STATUS_PERIOD_SEC").or("60").as_duration()?,
        })
    }
}

pub fn configure_logger() {
    let env = env_logger::Env::default()
        .filter_or("APP_LOG", "info")
        .write_style_or("APP_LOG_STYLE", "always");
    env_logger::init_from_env(env);
}

struct Env {
    name: String,
    default: Option<String>,
}

impl Env {
    fn var(name: &str) -> Self {
        Env {
            name: name.to_string(),
            default: None,
        }
    }

    fn or(self, default: &str) -> Self {
        Env {
            name: self.name,
            default: Some(default.to_string()),
        }
    }

    fn as_string(&self) -> Result<String, ConfigError> {
        match env::var(&self.name) {
            Ok(value) if !value.trim().is_empty() => Ok(value),
            Ok(_) => Err(ConfigError::EnvVarNotFoundError(self.name.clone())),
            Err(_) => match &self.default {
                Some(default_value) => Ok(default_value.clone()),
                None => Err(ConfigError::EnvVarNotFoundError(self.name.clone())),
            },
        }
    }

    fn as_url(&self) -> Result<Url, ConfigError> {
        let value = self.as_string()?;
        Url::parse(&value).map_err(|_| ConfigError::UrlParseError(self.name.clone()))
    }

    fn as_duration(&self) -> Result<Duration, ConfigError> {
        let value = self.as_string()?;
        value
            .parse::<u64>()
            .map(Duration::from_secs)
            .map_err(|_| ConfigError::DurationParseError(self.name.clone()))
    }
}
