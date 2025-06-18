//! SolarLog HTTP client.
//! This is the lower level client for SolarLog.
use failsafe::{
    backoff::{self, Constant},
    failure_policy::{self, ConsecutiveFailures},
    futures::CircuitBreaker,
};
use reqwest::{Client, StatusCode, Url};
use std::time::Duration;
use tokio::sync::RwLock;
use tokio_retry::RetryIf;
use tokio_retry::strategy::{ExponentialBackoff, jitter};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Wrong password")]
    WrongPassword,
    #[error("Query impossible")]
    QueryImpossible,
    #[error("Access denied")]
    AccessDenied,
    #[error("Request error: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("Request rejected by the circuit breaker")]
    RequestRejected,
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct HttpClient {
    client: Client,
    password: String,
    base_url: Url,
    token: RwLock<Option<String>>,
    circuit_breaker: failsafe::StateMachine<ConsecutiveFailures<Constant>, ()>,
}

impl HttpClient {
    /// Creates a new instance of `HttpClient`.
    pub fn new(url: &Url, password: &str) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_millis(500)) // 0.5 seconds timeout
            .build()
            .expect("Failed to create HTTP client");
        HttpClient {
            client: client,
            password: password.to_string(),
            base_url: url.clone(),
            token: RwLock::new(None),
            circuit_breaker: circuit_breaker(),
        }
    }

    /// Login to SolarLog device
    pub async fn login(&self) -> Result<()> {
        RetryIf::spawn(
            retry_strategy(),
            || async {
                self.circuit_breaker
                    .call_with(is_recorded_error, self.refresh_token(true))
                    .await
                    .map_err(|err| match err {
                        failsafe::Error::Inner(e) => e,
                        failsafe::Error::Rejected => Error::RequestRejected,
                    })
            },
            is_retryable_error,
        )
        .await?;
        Ok(())
    }

    /// Logout from SolarLog device
    pub async fn logout(&self) {
        let mut token_lock = self.token.write().await;
        *token_lock = None;
    }

    /// Query the SolarLog device.
    pub async fn query(&self, query: &str) -> Result<String> {
        let text = RetryIf::spawn(
            retry_strategy(),
            || async {
                self.circuit_breaker
                    .call_with(is_recorded_error, self.request_getjp(query))
                    .await
                    .map_err(|err| match err {
                        failsafe::Error::Inner(e) => e,
                        failsafe::Error::Rejected => Error::RequestRejected,
                    })
            },
            is_retryable_error,
        )
        .await?;
        Ok(text)
    }

    /// Private methods --------------------------------------------------------

    // Refresh token and return it if successful
    /// If `force` is true, it will always refresh the token even if it is already set.
    async fn refresh_token(&self, force: bool) -> Result<String> {
        let mut token_write = self.token.write().await;
        if token_write.is_some() && !force {
            return Ok(token_write.as_ref().unwrap().clone());
        }
        let token = self.request_login().await?;
        *token_write = token.clone();
        token.ok_or(Error::WrongPassword)
    }

    /// Get token if it exists, otherwise refresh it.
    async fn get_token(&self) -> Result<String> {
        if let Some(token) = self.token.read().await.as_ref() {
            return Ok(token.clone());
        }
        self.refresh_token(false).await
    }

    /// Clear the token if matching the provided token.
    async fn clear_token(&self, token: &str) {
        let mut token_write = self.token.write().await;
        if let Some(ref current_token) = *token_write {
            if current_token == token {
                *token_write = None;
            }
        }
    }

    /// Internal method to request a login and retrieve the session token.
    async fn request_login(&self) -> Result<Option<String>> {
        log::debug!("Sending login request");
        let url = self
            .base_url
            .join("/login")
            .expect("cannot build login URL");
        let params = [("u", "user"), ("p", &self.password)];
        let response = self.client.post(url).form(&params).send().await?;
        let token = response
            .cookies()
            .find(|c| c.name() == "SolarLog")
            .map(|c| c.value().to_string());
        if let Some(_) = token {
            log::debug!("Login successful");
        } else {
            log::debug!("Login failed: no token received");
        }
        Ok(token)
    }

    /// Internal method to perform the actual query.
    async fn request_getjp(&self, query: &str) -> Result<String> {
        log::debug!("Sending query request: {}", query);
        let token = self.get_token().await?;
        let url = self
            .base_url
            .join("/getjp")
            .expect("cannot build query URL");
        let body = format!("token={};{}", token, query);
        let response = self
            .client
            .post(url)
            .header("Cookie", format!("SolarLog={token}"))
            .body(body)
            .send()
            .await?
            .error_for_status()
            .map_err(Error::RequestError)?;
        let text = response.text().await?;
        match text.as_str() {
            t if t.contains("QUERY IMPOSSIBLE") => {
                return Err(Error::QueryImpossible);
            }
            t if t.contains("ACCESS DENIED") => {
                log::debug!("Access denied, clearing token");
                self.clear_token(&token).await;
                return Err(Error::AccessDenied);
            }
            _ => {}
        }
        log::debug!("Query result: {}", text);
        Ok(text)
    }
}

/// Creates a circuit breaker with a failure policy that allows up to 3 consecutive failures and will retry after 60 seconds.
fn circuit_breaker() -> failsafe::StateMachine<ConsecutiveFailures<Constant>, ()> {
    let backoff = backoff::constant(Duration::from_secs(60));
    let policy = failure_policy::consecutive_failures(3, backoff);
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
        Error::RequestError(err) => is_client_error(err),
        Error::WrongPassword => false,
        Error::QueryImpossible => false,
        Error::AccessDenied => true, // Retry if token expires or is invalid
        Error::RequestRejected => false, // Don't retry on circuit breaker rejection
    }
}

/// Predicate function for the circuit breaker to record errors that are not client errors.
fn is_recorded_error(error: &Error) -> bool {
    match error {
        Error::RequestError(err) => !is_client_error(err), // Don't record client errors
        Error::WrongPassword => false,                     // Don't record wrong password errors
        Error::QueryImpossible => false,                   // Don't record query impossible errors
        Error::AccessDenied => false,                      // Don't record access denied errors
        Error::RequestRejected => false, // Don't record circuit breaker rejections
    }
}
