//! Application configuration loaded from environment variables.
use std::env;

use envconfig::Envconfig;
use humantime::Duration;
use reqwest::Url;

pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const APP_NAME: &str = env!("CARGO_PKG_NAME");

#[derive(Envconfig)]
pub struct Config {
    #[allow(dead_code)]
    #[envconfig(from = "APP_LOG", default = "error")]
    pub app_log: String,
    #[allow(dead_code)]
    #[envconfig(from = "APP_LOG_STYLE", default = "always")]
    pub app_log_style: String,
    #[envconfig(from = "SOLARLOG_URL")]
    pub solarlog_url: Url,
    #[envconfig(from = "SOLARLOG_PASSWORD")]
    pub solarlog_password: String,
    #[envconfig(from = "HOMEASSISTANT_URL")]
    pub homeassistant_url: Url,
    #[envconfig(from = "HOMEASSISTANT_TOKEN")]
    pub homeassistant_token: String,
    #[envconfig(from = "SYNC_POWER_INTERVAL", default = "5s")]
    pub sync_power_interval: Duration,
    #[envconfig(from = "SYNC_ENERGY_INTERVAL", default = "60s")]
    pub sync_energy_interval: Duration,
    #[envconfig(from = "SYNC_STATUS_INTERVAL", default = "60s")]
    pub sync_status_interval: Duration,
}

pub fn configure_logger() {
    let env = env_logger::Env::default()
        .filter_or("APP_LOG", "info")
        .write_style_or("APP_LOG_STYLE", "always");
    env_logger::init_from_env(env);
}

#[cfg(test)]
mod tests {
    use super::*;
    use temp_env::{with_var, with_vars};

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
                ("SYNC_POWER_INTERVAL", Some("10s")),
                ("SYNC_ENERGY_INTERVAL", Some("20s")),
                ("SYNC_STATUS_INTERVAL", Some("30s")),
            ],
            || {
                let config = Config::init_from_env().unwrap();
                assert_eq!(config.app_log, "debug");
                assert_eq!(config.app_log_style, "auto");
                assert_eq!(
                    config.solarlog_url,
                    Url::parse("http://localhost:8080").unwrap()
                );
                assert_eq!(config.solarlog_password, "test_password");
                assert_eq!(
                    config.homeassistant_url,
                    Url::parse("http://localhost:8001").unwrap()
                );
                assert_eq!(config.homeassistant_token, "test_token");
                assert_eq!(
                    config.sync_power_interval,
                    std::time::Duration::from_secs(10).into()
                );
                assert_eq!(
                    config.sync_energy_interval,
                    std::time::Duration::from_secs(20).into()
                );
                assert_eq!(
                    config.sync_status_interval,
                    std::time::Duration::from_secs(30).into()
                );
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
