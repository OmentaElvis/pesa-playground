use anyhow::Context;
use reqwest::{Client, header::HeaderMap};
use serde::{Serialize, de::DeserializeOwned};

/// A helper client for making HTTP requests within self-tests.
///
/// This abstracts away the boilerplate of `reqwest` setup,
/// focusing on JSON-based API interactions.
#[derive(Clone, Default)]
pub struct TestApiClient {
    client: Client,
}

impl TestApiClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    /// Makes a POST request with a JSON body and deserializes the JSON response.
    ///
    /// # Arguments
    /// * `url` - The URL to send the request to.
    /// * `body` - The request body, which must be serializable to JSON.
    /// * `headers` - Optional custom headers to send with the request.
    pub async fn post_json<S, R>(
        &self,
        url: &str,
        body: &S,
        headers: Option<HeaderMap>,
    ) -> anyhow::Result<R>
    where
        S: Serialize,
        R: DeserializeOwned,
    {
        let mut request_builder = self.client.post(url).json(body);

        if let Some(h) = headers {
            request_builder = request_builder.headers(h);
        }

        let response = request_builder
            .send()
            .await
            .context(format!("Failed to send POST request to {}", url))?;

        // Check if the response was successful before attempting to deserialize
        let response = response
            .error_for_status()
            .context(format!("API POST request to {} failed with status", url))?;

        let response_text = response
            .text()
            .await
            .context("Failed to get response text")?;

        serde_json::from_str(&response_text).context(format!(
            "Failed to deserialize response from {}: {}",
            url, response_text
        ))
    }

    /// Makes a GET request and deserializes the JSON response.
    ///
    /// # Arguments
    /// * `url` - The URL to send the request to.
    /// * `headers` - Optional custom headers to send with the request.
    pub async fn get_json<R>(&self, url: &str, headers: Option<HeaderMap>) -> anyhow::Result<R>
    where
        R: DeserializeOwned,
    {
        let mut request_builder = self.client.get(url);

        if let Some(h) = headers {
            request_builder = request_builder.headers(h);
        }

        let response = request_builder
            .send()
            .await
            .context(format!("Failed to send GET request to {}", url))?;

        let response = response
            .error_for_status()
            .context(format!("API GET request to {} failed with status", url))?;

        let response_text = response
            .text()
            .await
            .context("Failed to get response text")?;

        serde_json::from_str(&response_text).context(format!(
            "Failed to deserialize response from {}: {}",
            url, response_text
        ))
    }

    /// Makes a POST request with a JSON body and returns the raw response.
    /// This allows the caller to inspect status codes before deserializing.
    pub async fn post_json_raw<S>(
        &self,
        url: &str,
        body: &S,
        headers: Option<HeaderMap>,
    ) -> anyhow::Result<reqwest::Response>
    where
        S: Serialize,
    {
        let mut request_builder = self.client.post(url).json(body);

        if let Some(h) = headers {
            request_builder = request_builder.headers(h);
        }

        let response = request_builder
            .send()
            .await
            .context(format!("Failed to send POST request to {}", url))?;

        Ok(response)
    }
}
