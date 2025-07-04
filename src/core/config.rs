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
    #[allow(dead_code)]
    pub app_log: String,
    #[allow(dead_code)]
    pub app_log_style: String,
    pub solarlog_url: Url,
    pub solarlog_password: String,
    pub homeassistant_url: Url,
    pub homeassistant_token: String,
    pub solar_power_sync_interval: Duration,
    pub solar_energy_sync_interval: Duration,
    pub solar_status_sync_interval: Duration,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Environment variable not found: {0}")]
    EnvVarNotFound(String),
    #[error("Failed to parse URL for environment variable: {0}")]
    InvalidUrl(String),
    #[error("Failed to parse duration for environment variable: {0}")]
    InvalidDuration(#[from] humantime::DurationError),
}

impl Config {
    /// Creates a new `Config` instance by reading environment variables.
    pub fn from_env() -> Result<Self, Error> {
        Ok(Self {
            app_name: env!("CARGO_PKG_NAME").to_string(),
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            app_log: Env::var("APP_LOG").or("error").as_string()?,
            app_log_style: Env::var("APP_LOG_STYLE").or("always").as_string()?,
            solarlog_url: Env::var("SOLARLOG_URL").as_url()?,
            solarlog_password: Env::var("SOLARLOG_PASSWORD").as_string()?,
            homeassistant_url: Env::var("HOMEASSISTANT_URL").as_url()?,
            homeassistant_token: Env::var("HOMEASSISTANT_TOKEN").as_string()?,
            solar_power_sync_interval: Env::var("SOLAR_POWER_SYNC_INTERVAL")
                .or("5s")
                .as_duration()?,
            solar_energy_sync_interval: Env::var("SOLAR_ENERGY_SYNC_INTERVAL")
                .or("60s")
                .as_duration()?,
            solar_status_sync_interval: Env::var("SOLAR_STATUS_SYNC_INTERVAL")
                .or("60s")
                .as_duration()?,
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

    fn as_string(&self) -> Result<String, Error> {
        match env::var(&self.name) {
            Ok(value) if !value.trim().is_empty() => Ok(value),
            Ok(_) => Err(Error::EnvVarNotFound(self.name.clone())),
            Err(_) => match &self.default {
                Some(default_value) => Ok(default_value.clone()),
                None => Err(Error::EnvVarNotFound(self.name.clone())),
            },
        }
    }

    fn as_url(&self) -> Result<Url, Error> {
        let value = self.as_string()?;
        Url::parse(&value).map_err(|_| Error::InvalidUrl(self.name.clone()))
    }

    fn as_duration(&self) -> Result<Duration, Error> {
        let value = self.as_string()?;
        humantime::parse_duration(&value)
            .map(|d| Duration::from_secs(d.as_secs()))
            .map_err(Error::InvalidDuration)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use temp_env::{with_var, with_vars};

    #[test]
    fn test_as_string_env_present() {
        with_var("TEST_STRING", Some("hello"), || {
            let val = Env::var("TEST_STRING").as_string().unwrap();
            assert_eq!(val, "hello");
        });
    }

    #[test]
    fn test_as_string_env_empty() {
        with_var("TEST_STRING", Some("   "), || {
            let err = Env::var("TEST_STRING").as_string().unwrap_err();
            matches!(err, Error::EnvVarNotFound(_));
        });
    }

    #[test]
    fn test_as_string_env_missing_with_default() {
        temp_env::with_var("TEST_STRING", None::<&str>, || {
            let val = Env::var("TEST_STRING").or("default").as_string().unwrap();
            assert_eq!(val, "default");
        });
    }

    #[test]
    fn test_as_string_env_missing_no_default() {
        temp_env::with_var("TEST_STRING", None::<&str>, || {
            let err = Env::var("TEST_STRING").as_string().unwrap_err();
            matches!(err, Error::EnvVarNotFound(_));
        });
    }

    #[test]
    fn test_as_url_valid() {
        with_var("TEST_URL", Some("http://localhost:1234"), || {
            let url = Env::var("TEST_URL").as_url().unwrap();
            assert_eq!(url.as_str(), "http://localhost:1234/");
        });
    }

    #[test]
    fn test_as_url_invalid() {
        with_var("TEST_URL", Some("not a url"), || {
            let err = Env::var("TEST_URL").as_url().unwrap_err();
            matches!(err, Error::InvalidUrl(_));
        });
    }

    #[test]
    fn test_as_duration_seconds() {
        with_var("TEST_DUR", Some("42s"), || {
            let dur = Env::var("TEST_DUR").as_duration().unwrap();
            assert_eq!(dur, Duration::from_secs(42));
        });
    }

    #[test]
    fn test_as_duration_human() {
        with_var("TEST_DUR", Some("2m"), || {
            let dur = Env::var("TEST_DUR").as_duration().unwrap();
            assert_eq!(dur, Duration::from_secs(120));
        });
    }

    #[test]
    fn test_as_duration_invalid() {
        with_var("TEST_DUR", Some("not a duration"), || {
            let err = Env::var("TEST_DUR").as_duration().unwrap_err();
            matches!(err, Error::InvalidDuration(_));
        });
    }

    #[test]
    fn test_config_from_env() {
        with_vars(
            [
                ("APP_LOG", Some("debug")),
                ("APP_LOG_STYLE", Some("auto")),
                ("SOLARLOG_URL", Some("http://localhost:8080")),
                ("SOLARLOG_PASSWORD", Some("test_password")),
                ("HOMEASSISTANT_URL", Some("http://localhost:8001")),
                ("HOMEASSISTANT_TOKEN", Some("test_token")),
                ("SOLAR_POWER_SYNC_INTERVAL", Some("10s")),
                ("SOLAR_ENERGY_SYNC_INTERVAL", Some("20s")),
                ("SOLAR_STATUS_SYNC_INTERVAL", Some("30s")),
            ],
            || {
                let config = Config::from_env().unwrap();
                assert_eq!(config.app_name, env!("CARGO_PKG_NAME"));
                assert_eq!(config.app_version, env!("CARGO_PKG_VERSION"));
                assert_eq!(config.app_log, "debug");
                assert_eq!(config.app_log_style, "auto");
                assert_eq!(config.solarlog_url.as_str(), "http://localhost:8080/");
                assert_eq!(config.solarlog_password, "test_password");
                assert_eq!(config.homeassistant_url.as_str(), "http://localhost:8001/");
                assert_eq!(config.homeassistant_token, "test_token");
                assert_eq!(config.solar_power_sync_interval, Duration::from_secs(10));
                assert_eq!(config.solar_energy_sync_interval, Duration::from_secs(20));
                assert_eq!(config.solar_status_sync_interval, Duration::from_secs(30));
            },
        );
    }

    #[test]
    fn test_configure_logger() {
        with_var("APP_LOG", Some("debug"), || {
            configure_logger();
            let log_level = log::max_level();
            assert_eq!(log_level, log::LevelFilter::Debug);
        });
    }
}
