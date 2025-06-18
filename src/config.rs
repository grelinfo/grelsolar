//! Application configuration loaded from environment variables.

use reqwest::Url;
use std::env;
use thiserror::Error;

/// Holds all configuration for the application.
#[derive(Debug, Clone)]
pub struct Config {
    pub solarlog_url: Url,
    pub solarlog_password: String,
    pub home_assistant_url: Url,
    pub home_assistant_token: String,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Environment variable not found: {0}")]
    EnvVarNotFoundError(String),
    #[error("Failed to parse URL for environment variable: {0}")]
    UrlParseError(String),
    #[error("Empty value for environment variable: {0}")]
    EmptyEnvVarError(String),
}

impl Config {
    /// Creates a new `Config` instance by reading environment variables.
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            solarlog_url: url_from_env("SOLARLOG_URL")?,
            solarlog_password: string_from_env("SOLARLOG_PASSWORD", false)?,
            home_assistant_url: url_from_env("HOME_ASSISTANT_URL")?,
            home_assistant_token: string_from_env("HOME_ASSISTANT_TOKEN", false)?,
        })
    }
}

fn string_from_env(name: &str, allow_empty: bool) -> Result<String, ConfigError> {
    let value = env::var(name)
        .map_err(|_| ConfigError::EnvVarNotFoundError(name.to_string()))?;
    if !allow_empty && value.is_empty() {
        Err(ConfigError::EmptyEnvVarError(name.to_string()))
    } else {
        Ok(value)
    }
}

fn url_from_env(name: &str) -> Result<Url, ConfigError> {
    let value = string_from_env(name, false)?;
    Url::parse(&value)
        .map_err(|_| ConfigError::UrlParseError(name.to_string()))
}