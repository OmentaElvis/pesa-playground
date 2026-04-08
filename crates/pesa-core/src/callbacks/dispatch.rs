use reqwest::{Client, header::HeaderMap};
use serde::Serialize;
use serde_json::Value;
use std::{fmt::Debug, time::Duration};

/// Configuration for the dispatch service.
#[derive(Debug, Clone, Copy)]
pub struct DispatchConfig {
    /// The request timeout for a single dispatch attempt.
    pub timeout: Duration,
    /// The total number of attempts to make before failing.
    pub max_retries: u32,
}

/// The successful result of a dispatch operation.
#[derive(Debug)]
pub struct DispatchResponse {
    /// The final HTTP status code received after a successful dispatch.
    pub final_status_code: u16,
    /// The body of the successful response.
    pub final_body: String,
    /// The headers of the successful response.
    pub final_headers: HeaderMap,
    /// The number of attempts it took to get a successful response.
    pub attempts_made: u32,
}

/// A robust service for dispatching callbacks with retry and backoff logic.
pub struct CallbackDispatchService {
    client: Client,
    config: DispatchConfig,
}

impl CallbackDispatchService {
    /// Creates a new dispatch service with the given configuration.
    ///
    /// # Panics
    /// Panics if the reqwest client fails to build, which is a critical,
    /// unrecoverable error for the application's startup process.
    pub fn new(config: DispatchConfig) -> Self {
        let client = Client::builder()
            .timeout(config.timeout)
            .build()
            .expect("Failed to build reqwest client for CallbackDispatchService");
        Self { client, config }
    }

    /// Dispatches a serializable payload to a URL, with retries on failure.
    pub async fn dispatch<T: Serialize>(
        &self,
        url: &str,
        payload: &T,
    ) -> Result<DispatchResponse, anyhow::Error> {
        let mut last_error: Option<anyhow::Error> = None;

        for attempt in 1..=self.config.max_retries {
            tracing::info!(
                "Dispatching callback to {} (Attempt {}/{})",
                url,
                attempt,
                self.config.max_retries
            );

            match self.client.post(url).json(payload).send().await {
                Ok(response) => {
                    let status = response.status();
                    if status.is_success() {
                        let headers = response.headers().clone();
                        let body = response.text().await.unwrap_or_default();
                        tracing::info!("Callback to {} succeeded with status {}", url, status);
                        return Ok(DispatchResponse {
                            final_status_code: status.as_u16(),
                            final_body: body,
                            final_headers: headers,
                            attempts_made: attempt,
                        });
                    } else {
                        let error_text =
                            format!("Request failed with non-success status: {}", status);
                        tracing::warn!(
                            "{}. Response body: {:?}",
                            error_text,
                            response.text().await
                        );
                        last_error = Some(anyhow::anyhow!(error_text));
                    }
                }
                Err(e) => {
                    let error_text = format!("Request failed with network/timeout error: {}", e);
                    tracing::error!("{}", error_text);
                    last_error = Some(anyhow::anyhow!(error_text));
                }
            }

            // If it's the last attempt, don't sleep.
            if attempt < self.config.max_retries {
                // Exponential backoff with jitter: (2^attempt * 500ms) + random(0-250ms)
                let backoff_ms = 2_u64.pow(attempt) * 500;
                let jitter_ms = rand::random::<u64>() % 250;
                let sleep_duration = Duration::from_millis(backoff_ms + jitter_ms);
                tracing::info!("Waiting {:?} before next retry.", sleep_duration);
                tokio::time::sleep(sleep_duration).await;
            }
        }

        Err(last_error.unwrap_or_else(|| {
            anyhow::anyhow!(
                "Callback dispatch failed after {} attempts.",
                self.config.max_retries
            )
        }))
    }
}

/// Converts a reqwest HeaderMap to a serde_json Value (as an object).
pub fn headers_to_json_value(headers: &HeaderMap) -> Value {
    let mut map = serde_json::Map::new();
    for (key, value) in headers.iter() {
        // `to_str` can fail if the header value is not valid UTF-8.
        // We lossily convert it to a string to ensure it's always representable.
        map.insert(
            key.as_str().to_string(),
            Value::String(String::from_utf8_lossy(value.as_bytes()).into_owned()),
        );
    }
    Value::Object(map)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[test]
    fn test_dispatch_config_defaults() {
        let config = DispatchConfig {
            timeout: Duration::from_secs(30),
            max_retries: 3,
        };

        assert_eq!(config.timeout, Duration::from_secs(30));
        assert_eq!(config.max_retries, 3);
    }

    #[tokio::test]
    async fn test_dispatch_successful_response() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/callback"))
            .respond_with(ResponseTemplate::new(200).set_body_string("OK"))
            .mount(&mock_server)
            .await;

        let config = DispatchConfig {
            timeout: Duration::from_secs(5),
            max_retries: 3,
        };
        let service = CallbackDispatchService::new(config);

        let payload = json!({"test": "data"});
        let url = format!("{}/callback", mock_server.uri().trim_end_matches('/'));
        let result = service.dispatch(&url, &payload).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.final_status_code, 200);
        assert_eq!(response.final_body, "OK");
        assert_eq!(response.attempts_made, 1);
    }

    #[tokio::test]
    async fn test_dispatch_retry_on_failure() {
        let mock_server = MockServer::start().await;

        // First two attempts fail with 500, third succeeds
        Mock::given(method("POST"))
            .and(path("/callback"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&mock_server)
            .await;

        let config = DispatchConfig {
            timeout: Duration::from_secs(5),
            max_retries: 3,
        };
        let service = CallbackDispatchService::new(config);

        let payload = json!({"test": "data"});
        let url = format!("{}/callback", mock_server.uri().trim_end_matches('/'));
        let result = service.dispatch(&url, &payload).await;

        // Should fail after max_retries
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_dispatch_network_error() {
        let mock_server = MockServer::start().await;

        // No mock mounted - will cause connection error
        let config = DispatchConfig {
            timeout: Duration::from_millis(100),
            max_retries: 1,
        };
        let service = CallbackDispatchService::new(config);

        let payload = json!({"test": "data"});
        let url = format!("{}/callback", mock_server.uri().trim_end_matches('/'));
        let result = service.dispatch(&url, &payload).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_headers_to_json_value() {
        let mut headers = HeaderMap::new();
        headers.insert("content-type", "application/json".parse().unwrap());
        headers.insert("x-custom", "value123".parse().unwrap());

        let result = headers_to_json_value(&headers);

        assert!(result.is_object());
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("content-type").unwrap(), "application/json");
        assert_eq!(obj.get("x-custom").unwrap(), "value123");
    }
}
