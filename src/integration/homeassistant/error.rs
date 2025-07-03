//! Error handling for the Home Assistant client.

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),
    #[error("Request rejected by the circuit breaker")]
    RequestRejected,
    #[error("JSON serialization failed: {0}")]
    JsonSerializationFailed(#[from] serde_json::Error),
    // Add more variants as needed
}
pub type Result<T> = std::result::Result<T, Error>;
