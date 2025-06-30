//! SolarLog HTTP client.
//! This is the lower level client for SolarLog.
use crate::solarlog::error::{Error, Result};
use failsafe::{
    backoff::{self, Constant},
    failure_policy::{self, ConsecutiveFailures},
    futures::CircuitBreaker,
};
use reqwest::{Client, StatusCode, Url};
use serde_json::Value;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio_retry::RetryIf;
use tokio_retry::strategy::{ExponentialBackoff, jitter};

pub struct HttpClient {
    client: Client,
    password: String,
    base_url: Url,
    token: RwLock<Option<String>>,
    circuit_breaker: failsafe::StateMachine<ConsecutiveFailures<Constant>, ()>,
}

impl HttpClient {
    /// Creates a new instance of `HttpClient`.
    pub fn new(url: Url, password: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_millis(500)) // 0.5 seconds timeout
            .build()
            .expect("Failed to create HTTP client");
        HttpClient {
            client,
            password,
            base_url: url,
            token: RwLock::new(None),
            circuit_breaker: Self::circuit_breaker(),
        }
    }

    /// Check if the client is logged in.
    /// Returns `true` if logged in, `false` otherwise.
    pub async fn is_logged_in(&self) -> bool {
        let token_read = self.token.read().await;
        token_read.is_some()
    }

    /// Login to SolarLog device.
    /// If `force` is true, it will always login even if already logged in.
    pub async fn login(&self, force: bool) -> Result<()> {
        RetryIf::spawn(
            Self::retry_strategy(),
            || async {
                self.circuit_breaker
                    .call_with(Self::is_recorded_error, self.do_login(force))
                    .await
                    .map_err(|err| match err {
                        failsafe::Error::Inner(e) => e,
                        failsafe::Error::Rejected => Error::RequestRejected,
                    })
            },
            Self::is_retryable_error,
        )
        .await?;
        Ok(())
    }

    /// Logs out from the SolarLog device.
    ///
    /// This method clears the authentication token and sends a logout request to the device.
    /// The logout request is performed only once, without retries or circuit breaker protection.
    /// If the logout request fails, the error is logged as a warning, but no error is returned to the caller.
    /// Returns `true` if logout was successful, `false` otherwise.
    pub async fn logout(&self) -> bool {
        let mut token_lock = self.token.write().await;
        if let Some(token) = token_lock.take() {
            if let Err(err) = self.request_logout(&token).await {
                log::warn!("Failed to logout: {err}");
                false
            } else {
                log::debug!("Logout successful");
                true
            }
        } else {
            log::debug!("No token to logout");
            false
        }
    }

    /// Query the SolarLog device.
    pub async fn query(&self, query: &str) -> Result<Value> {
        RetryIf::spawn(
            Self::retry_strategy(),
            || async {
                self.circuit_breaker
                    .call_with(Self::is_recorded_error, self.do_query(query))
                    .await
                    .map_err(|err| match err {
                        failsafe::Error::Inner(e) => e,
                        failsafe::Error::Rejected => Error::RequestRejected,
                    })
            },
            Self::is_retryable_error,
        )
        .await
    }

    // Execute a login operation.
    /// If `force` is true, it will always refresh the token even if it is already set.
    async fn do_login(&self, force: bool) -> Result<()> {
        if !force && self.is_logged_in().await {
            return Ok(());
        }
        self.refresh_token(force).await?;
        Ok(())
    }

    /// Execute a query operation.
    async fn do_query(&self, query: &str) -> Result<Value> {
        self.do_login(false).await?;
        let token_read = self.token.read().await;
        let token = token_read.as_ref().ok_or(Error::TokenExpired)?;
        let response_text = self.request_getjp(token, query).await?;
        let result = Self::validate_query_response(&response_text);
        match result {
            Ok(text) => serde_json::from_str(text).map_err(Error::ResponseJsonError),
            Err(Error::AccessDenied) => {
                // Clone and release the read lock before acquiring the write lock
                let token = token.clone();
                drop(token_read); // Release read lock before acquiring write lock
                self.clear_token(&token).await;
                Err(Error::AccessDenied)
            }
            Err(e) => Err(e),
        }
    }

    /// Refresh the login token if necessary.
    /// If `force` is true, it will always refresh the token even if it is already set.
    async fn refresh_token(&self, force: bool) -> Result<()> {
        let mut token_write = self.token.write().await;
        if !force && token_write.is_some() {
            log::debug!("Token already set, no need to refresh");
            return Ok(());
        }
        let token = self.request_login().await?;
        match token {
            Some(_) => {
                *token_write = token;
                log::debug!("Token refreshed");
                Ok(())
            }
            None => {
                *token_write = None;
                log::debug!("Token cleared");
                Err(Error::WrongPassword)
            }
        }
    }

    /// Clear the token if matching the provided token.
    /// Avoid clearing the wrong token when switching from read to write lock.
    async fn clear_token(&self, token: &str) {
        let mut token_write = self.token.write().await;
        if let Some(ref current_token) = *token_write {
            if current_token == token {
                *token_write = None;
                log::debug!("Token cleared");
            }
        }
    }

    /// Perform a login request to the SolarLog device.
    /// Returns the token if successful, or `None` if login failed.
    async fn request_login(&self) -> Result<Option<String>> {
        log::debug!("Send login request");
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
        if token.is_some() {
            log::debug!("Login successful, token received");
        } else {
            log::debug!("Login failed, no token received");
        }
        Ok(token)
    }

    /// Perform a logout request to the SolarLog device.
    pub async fn request_logout(&self, token: &str) -> Result<()> {
        log::debug!("Send logout request");
        let url = self
            .base_url
            .join("/logout")
            .expect("cannot build logout URL");
        self.client
            .post(url)
            .header("cookie", format!("SolarLog={token}"))
            .send()
            .await?
            .error_for_status()
            .map_err(Error::RequestFailed)?;
        log::debug!("Logout successful");
        Ok(())
    }

    /// Perform a GET request to the SolarLog device with the provided query.
    async fn request_getjp(&self, token: &str, query: &str) -> Result<String> {
        log::debug!("Send query request: {query}");
        let url = self
            .base_url
            .join("/getjp")
            .expect("cannot build query URL");
        let body = format!("token={token};{query}");
        let response = self
            .client
            .post(url)
            .header("cookie", format!("SolarLog={token}"))
            .body(body)
            .send()
            .await?
            .error_for_status()
            .map_err(Error::RequestFailed)?;
        let text = response.text().await?;
        log::debug!("Query response: {text}");
        Ok(text)
    }

    /// Pure function to parse the SolarLog getjp response and map to Result.
    fn validate_query_response(text: &str) -> Result<&str> {
        if text.contains("QUERY IMPOSSIBLE") {
            return Err(Error::QueryImpossible);
        }
        if text.contains("ACCESS DENIED") {
            return Err(Error::AccessDenied);
        }
        Ok(text)
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
            Error::RequestFailed(err) => !Self::is_client_error(err), // Retry if not a client error
            Error::AccessDenied => true, // Retry if token expires or is invalid
            Error::TokenExpired => true, // Retry if login expired
            _ => false,                  // Don't retry on other errors
        }
    }

    /// Predicate function for the circuit breaker to record errors that are not client errors.
    fn is_recorded_error(error: &Error) -> bool {
        match error {
            Error::RequestFailed(err) => !Self::is_client_error(err), // Record if not a client error
            _ => false,                                               // Don't record other errors
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::StatusCode;

    fn make_reqwest_error_with_status(status: StatusCode) -> reqwest::Error {
        let response = reqwest::Response::from(
            http::response::Builder::new()
                .status(status)
                .body(Vec::new())
                .unwrap(),
        );
        response.error_for_status().unwrap_err()
    }

    fn create_json_serialization_error() -> Error {
        Error::ResponseJsonError(serde_json::from_str::<serde_json::Value>("not_json").unwrap_err())
    }

    #[test]
    fn test_new_http_client() {
        let url = Url::parse("http://localhost:8080").expect("cannot parse URL");
        let password = String::from("test_password");
        let client = HttpClient::new(url.clone(), password.clone());
        assert_eq!(client.base_url, url);
        assert_eq!(client.password, password);
    }

    #[test]
    fn test_error_for_response() {
        assert!(HttpClient::validate_query_response(r#"{"780": 3628}"#).is_ok());
        assert!(matches!(
            HttpClient::validate_query_response(r#"{{"QUERY IMPOSSIBLE 000"}}"#),
            Err(Error::QueryImpossible)
        ));
        assert!(matches!(
            HttpClient::validate_query_response(r#"{"780": "ACCESS DENIED"}"#),
            Err(Error::AccessDenied)
        ));
    }

    #[test]
    fn test_is_client_error() {
        let err_400 = make_reqwest_error_with_status(StatusCode::BAD_REQUEST);
        let err_500 = make_reqwest_error_with_status(StatusCode::INTERNAL_SERVER_ERROR);

        assert!(HttpClient::is_client_error(&err_400));
        assert!(!HttpClient::is_client_error(&err_500));
    }

    #[test]
    fn test_is_retryable_error() {
        let err_400 = Error::RequestFailed(make_reqwest_error_with_status(StatusCode::BAD_REQUEST));
        let err_500 = Error::RequestFailed(make_reqwest_error_with_status(
            StatusCode::INTERNAL_SERVER_ERROR,
        ));

        assert!(!HttpClient::is_retryable_error(&err_400));
        assert!(HttpClient::is_retryable_error(&err_500));
        assert!(!HttpClient::is_retryable_error(&Error::WrongPassword));
        assert!(!HttpClient::is_retryable_error(&Error::QueryImpossible));
        assert!(HttpClient::is_retryable_error(&Error::AccessDenied));
        assert!(!HttpClient::is_retryable_error(&Error::RequestRejected));
        assert!(HttpClient::is_retryable_error(&Error::TokenExpired));
        assert!(!HttpClient::is_retryable_error(
            &create_json_serialization_error()
        ));
    }

    #[test]
    fn test_is_recorded_error() {
        let err_400 = Error::RequestFailed(make_reqwest_error_with_status(StatusCode::BAD_REQUEST));
        let err_500 = Error::RequestFailed(make_reqwest_error_with_status(
            StatusCode::INTERNAL_SERVER_ERROR,
        ));

        assert!(!HttpClient::is_recorded_error(&err_400));
        assert!(HttpClient::is_recorded_error(&err_500));
        assert!(!HttpClient::is_recorded_error(&Error::WrongPassword));
        assert!(!HttpClient::is_recorded_error(&Error::QueryImpossible));
        assert!(!HttpClient::is_recorded_error(&Error::AccessDenied));
        assert!(!HttpClient::is_recorded_error(&Error::RequestRejected));
        assert!(!HttpClient::is_recorded_error(&Error::TokenExpired));
        assert!(!HttpClient::is_recorded_error(
            &create_json_serialization_error()
        ));
    }
}
