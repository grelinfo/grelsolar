//! Error handling for the SolarLog API client.

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Authentication failed: token expired")]
    TokenExpired,
    #[error("Authentication failed: wrong password")]
    WrongPassword,
    #[error("Authorization failed: access denied (token not allowed or expired)")]
    AccessDenied,

    #[error("Request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),
    #[error("Request rejected: circuit breaker open")]
    RequestRejected,

    #[error("Query failed: impossible query")]
    QueryImpossible,
    #[error("Response JSON error: {0}")]
    ResponseJsonError(#[from] serde_json::Error),
    #[error("Value parse error: {0}")]
    ValueParseError(String),
}

pub type Result<T> = std::result::Result<T, Error>;
