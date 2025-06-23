//! Error handling for the SolarLog API client.

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Login expired during operation")]
    LoginExpired,
    #[error("Wrong password")]
    WrongPassword,
    #[error("Query impossible")]
    QueryImpossible,
    #[error("Access denied")]
    AccessDenied,
    #[error("Request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),
    #[error("Request rejected by the circuit breaker")]
    RequestRejected,
    #[error("JSON serialization failed: {0}")]
    JsonSerializationFailed(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
