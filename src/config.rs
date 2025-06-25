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
    DurationParseError(#[from] humantime::DurationError),
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
            solar_power_period: Env::var("SOLAR_POWER_PERIOD").or("5s").as_duration()?,
            solar_energy_period: Env::var("SOLAR_ENERGY_PERIOD").or("60s").as_duration()?,
            solar_status_period: Env::var("SOLAR_STATUS_PERIOD").or("60s").as_duration()?,
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
        humantime::parse_duration(&value)
            .map(|d| Duration::from_secs(d.as_secs()))
            .map_err(ConfigError::DurationParseError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::time::Duration;

    fn set_env(key: &str, value: &str) {
        unsafe {
            env::set_var(key, value);
        }
    }

    fn clear_env(key: &str) {
        unsafe {
            env::remove_var(key);
        }
    }

    #[test]
    fn test_as_string_env_present() {
        unsafe {
            env::set_var("TEST_STRING", "hello");
        }
        let val = Env::var("TEST_STRING").as_string().unwrap();
        assert_eq!(val, "hello");
        clear_env("TEST_STRING");
    }

    #[test]
    fn test_as_string_env_empty() {
        unsafe {
            env::set_var("TEST_STRING", "   ");
        }
        let err = Env::var("TEST_STRING").as_string().unwrap_err();
        matches!(err, ConfigError::EnvVarNotFoundError(_));
        clear_env("TEST_STRING");
    }

    #[test]
    fn test_as_string_env_missing_with_default() {
        clear_env("TEST_STRING");
        let val = Env::var("TEST_STRING").or("default").as_string().unwrap();
        assert_eq!(val, "default");
    }

    #[test]
    fn test_as_string_env_missing_no_default() {
        clear_env("TEST_STRING");
        let err = Env::var("TEST_STRING").as_string().unwrap_err();
        matches!(err, ConfigError::EnvVarNotFoundError(_));
    }

    #[test]
    fn test_as_url_valid() {
        set_env("TEST_URL", "http://localhost:1234");
        let url = Env::var("TEST_URL").as_url().unwrap();
        assert_eq!(url.as_str(), "http://localhost:1234/");
        clear_env("TEST_URL");
    }

    #[test]
    fn test_as_url_invalid() {
        set_env("TEST_URL", "not a url");
        let err = Env::var("TEST_URL").as_url().unwrap_err();
        matches!(err, ConfigError::UrlParseError(_));
        clear_env("TEST_URL");
    }

    #[test]
    fn test_as_duration_seconds() {
        set_env("TEST_DUR", "42s");
        let dur = Env::var("TEST_DUR").as_duration().unwrap();
        assert_eq!(dur, Duration::from_secs(42));
        clear_env("TEST_DUR");
    }

    #[test]
    fn test_as_duration_human() {
        set_env("TEST_DUR", "2m");
        let dur = Env::var("TEST_DUR").as_duration().unwrap();
        assert_eq!(dur, Duration::from_secs(120));
        clear_env("TEST_DUR");
    }

    #[test]
    fn test_as_duration_invalid() {
        set_env("TEST_DUR", "not a duration");
        let err = Env::var("TEST_DUR").as_duration().unwrap_err();
        matches!(err, ConfigError::DurationParseError(_));
        clear_env("TEST_DUR");
    }

    #[test]
    fn test_config_from_env() {
        set_env("APP_LOG", "debug");
        set_env("APP_LOG_STYLE", "auto");
        set_env("SOLARLOG_URL", "http://localhost:8080");
        set_env("SOLARLOG_PASSWORD", "test_password");
        set_env("HOME_ASSISTANT_URL", "http://localhost:8001");
        set_env("HOME_ASSISTANT_TOKEN", "test_token");
        set_env("SOLAR_POWER_PERIOD", "10s");
        set_env("SOLAR_ENERGY_PERIOD", "20s");
        set_env("SOLAR_STATUS_PERIOD", "30s");

        let config = Config::from_env().unwrap();

        assert_eq!(config.app_name, env!("CARGO_PKG_NAME"));
        assert_eq!(config.app_version, env!("CARGO_PKG_VERSION"));
        assert_eq!(config.app_log, "debug");
        assert_eq!(config.app_log_style, "auto");
        assert_eq!(config.solarlog_url.as_str(), "http://localhost:8080/");
        assert_eq!(config.solarlog_password, "test_password");
        assert_eq!(config.home_assistant_url.as_str(), "http://localhost:8001/");
        assert_eq!(config.home_assistant_token, "test_token");
        assert_eq!(config.solar_power_period, Duration::from_secs(10));
        assert_eq!(config.solar_energy_period, Duration::from_secs(20));
        assert_eq!(config.solar_status_period, Duration::from_secs(30));

        clear_env("APP_LOG");
        clear_env("APP_LOG_STYLE");
        clear_env("SOLARLOG_URL");
        clear_env("SOLARLOG_PASSWORD");
        clear_env("HOME_ASSISTANT_URL");
        clear_env("HOME_ASSISTANT_TOKEN");
        clear_env("SOLAR_POWER_PERIOD");
        clear_env("SOLAR_ENERGY_PERIOD");
        clear_env("SOLAR_STATUS_PERIOD");
    }

    #[test]
    fn test_configure_logger() {
        set_env("APP_LOG", "debug");
        configure_logger();
        let log_level = log::max_level();
        assert_eq!(log_level, log::LevelFilter::Debug);
        clear_env("APP_LOG");
    }
}
