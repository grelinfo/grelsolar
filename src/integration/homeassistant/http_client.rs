//! Home Assistant HTTP client.
//! This is the lower level client for Home Assistant devices.
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use failsafe::{
    backoff::{self, Constant},
    failure_policy::{self, ConsecutiveFailures},
    futures::CircuitBreaker,
};
use reqwest::{Client, StatusCode, Url};
use serde_json::{self};
use std::time::Duration;
use tokio_retry::RetryIf;
use tokio_retry::strategy::{ExponentialBackoff, jitter};

use super::schemas::StateCreateOrUpdate;
use super::{Error, Result};

pub struct HttpClient {
    client: Client,
    token: String,
    base_url: Url,
    circuit_breaker: failsafe::StateMachine<ConsecutiveFailures<Constant>, ()>,
}

impl HttpClient {
    /// Creates a new instance of `HttpClient`.
    pub fn new(url: Url, token: String) -> Self {
        let client = Client::builder()
            .pool_idle_timeout(Duration::from_secs(30)) // 30 seconds idle timeout
            .pool_max_idle_per_host(2) // Maximum 2 idle connections per host
            .timeout(Duration::from_millis(500)) // 0.5 seconds timeout
            .build()
            .expect("Failed to create HTTP client");
        HttpClient {
            client,
            token,
            base_url: url,
            circuit_breaker: Self::circuit_breaker(),
        }
    }

    /// Creates or updates a state in Home Assistant.
    pub async fn set_state(&self, entity_id: &str, state: &StateCreateOrUpdate) -> Result<()> {
        let body = serde_json::to_string(state)?;
        RetryIf::spawn(
            Self::retry_strategy(),
            || async {
                self.circuit_breaker
                    .call_with(
                        Self::is_recorded_error,
                        self.request_post_state(entity_id, &body),
                    )
                    .await
                    .map_err(|err| match err {
                        failsafe::Error::Rejected => Error::RequestRejected,
                        failsafe::Error::Inner(e) => e,
                    })
            },
            Self::is_retryable_error,
        )
        .await?;
        Ok(())
    }

    /// Internal method to post state to Home Assistant.
    async fn request_post_state(&self, entity_id: &str, body: &str) -> Result<()> {
        log::debug!("Sending post state request for entity '{entity_id}': {body}",);
        let url = self
            .base_url
            .join(&format!("api/states/{entity_id}"))
            .expect("cannot post state URL");
        self.client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Content-Type", "application/json")
            .body(body.to_string())
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    /// Creates a circuit breaker with a failure policy that allows up to 3 consecutive failures and will retry after 60 seconds.
    fn circuit_breaker() -> failsafe::StateMachine<ConsecutiveFailures<Constant>, ()> {
        let backoff = backoff::constant(Duration::from_secs(60));
        let policy = failure_policy::consecutive_failures(5, backoff);
        // Creates a circuit breaker with given policy.
        failsafe::Config::new().failure_policy(policy).build()
    }

    /// Create a retry strategy with exponential backoff starting at 10 milliseconds, with jitter, and a maximum of 3 retries.
    fn retry_strategy() -> impl Iterator<Item = Duration> {
        ExponentialBackoff::from_millis(10).map(jitter).take(3)
    }

    /// Check if the error is a HTTP 4xx client error.
    fn is_client_error(error: &reqwest::Error) -> bool {
        error
            .status()
            .map(|status_code| StatusCode::is_client_error(&status_code))
            .unwrap_or(false)
    }

    // Predicate function for the retry strategy to determine if an error is retryable.
    fn is_retryable_error(error: &Error) -> bool {
        match error {
            Error::RequestFailed(err) => !HttpClient::is_client_error(err), // Don't retry on client errors
            Error::RequestRejected => false, // Don't retry on circuit breaker rejection
            Error::JsonSerializationFailed(_) => false, // Don't retry on serialization errors
        }
    }

    /// Predicate function for the circuit breaker to record errors that are not client errors.
    fn is_recorded_error(error: &Error) -> bool {
        match error {
            Error::RequestFailed(err) => !HttpClient::is_client_error(err), // Don't record client errors
            Error::RequestRejected => false, // Don't record circuit breaker rejections
            Error::JsonSerializationFailed(_) => false, // Don't record serialization errors
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {

    use super::*;
    use reqwest::StatusCode;

    fn create_reqwest_error_with_status(status: StatusCode) -> reqwest::Error {
        let response = http::Response::builder()
            .status(status)
            .body(Vec::new())
            .unwrap();
        reqwest::Response::from(response)
            .error_for_status()
            .unwrap_err()
    }

    fn create_json_serialization_error() -> Error {
        Error::JsonSerializationFailed(serde_json::Error::io(std::io::Error::other("fail")))
    }

    #[test]
    fn test_is_client_error() {
        let err_400 = create_reqwest_error_with_status(StatusCode::BAD_REQUEST);
        let err_500 = create_reqwest_error_with_status(StatusCode::INTERNAL_SERVER_ERROR);

        assert!(
            HttpClient::is_client_error(&err_400),
            "400 error should be a client error"
        );
        assert!(
            !HttpClient::is_client_error(&err_500),
            "500 error should not be a client error"
        );
    }

    #[test]
    fn test_is_retryable_error() {
        let err_400 =
            Error::RequestFailed(create_reqwest_error_with_status(StatusCode::BAD_REQUEST));
        let err_500 = Error::RequestFailed(create_reqwest_error_with_status(
            StatusCode::INTERNAL_SERVER_ERROR,
        ));
        let err_rejected = Error::RequestRejected;
        let err_json = create_json_serialization_error();

        assert!(
            !HttpClient::is_retryable_error(&err_400),
            "4xx errors should not be retryable"
        );
        assert!(
            HttpClient::is_retryable_error(&err_500),
            "5xx errors should be retryable"
        );
        assert!(
            !HttpClient::is_retryable_error(&err_rejected),
            "RequestRejected should not be retryable"
        );
        assert!(
            !HttpClient::is_retryable_error(&err_json),
            "JsonSerializationFailed should not be retryable"
        );
    }

    #[test]
    fn test_is_recorded_error() {
        // Reuse error samples from previous test
        let err_400 =
            Error::RequestFailed(create_reqwest_error_with_status(StatusCode::BAD_REQUEST));
        let err_500 = Error::RequestFailed(create_reqwest_error_with_status(
            StatusCode::INTERNAL_SERVER_ERROR,
        ));
        let err_rejected = Error::RequestRejected;
        let err_json = create_json_serialization_error();

        assert!(
            !HttpClient::is_recorded_error(&err_400),
            "4xx errors should not be recorded"
        );
        assert!(
            HttpClient::is_recorded_error(&err_500),
            "5xx errors should be recorded"
        );
        assert!(
            !HttpClient::is_recorded_error(&err_json),
            "JsonSerializationFailed should not be recorded"
        );
        assert!(
            !HttpClient::is_recorded_error(&err_rejected),
            "RequestRejected should not be recorded"
        );
    }
}
