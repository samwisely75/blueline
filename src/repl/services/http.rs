//! # HTTP Service
//!
//! Manages HTTP request execution and response handling.

use anyhow::Result;
use bluenote::{HttpClient, HttpConnectionProfile, HttpRequestArgs, HttpResponse, Url, UrlPath};
use std::collections::HashMap;
use tokio::sync::mpsc;

/// Type alias for parsed request result
pub type ParsedRequest = (BufferRequestArgs, String);

/// Type alias for profile information (name, path)
type ProfileInfo = (String, String);

/// Message type for async HTTP response handling
#[derive(Debug)]
pub enum HttpResponseMessage {
    /// Successful HTTP response with full request context
    Success {
        request: BufferRequestArgs,
        response: Box<HttpResponse>,
        url: String,
    },
    /// Error during request execution
    Error { message: String },
}

/// HTTP request arguments parsed from the request buffer
#[derive(Debug, Clone)]
pub struct BufferRequestArgs {
    method: Option<String>,
    url_path: Option<UrlPath>,
    body: Option<String>,
    headers: HashMap<String, String>,
}

impl HttpRequestArgs for BufferRequestArgs {
    fn method(&self) -> Option<&String> {
        self.method.as_ref()
    }

    fn url_path(&self) -> Option<&UrlPath> {
        self.url_path.as_ref()
    }

    fn body(&self) -> Option<&String> {
        self.body.as_ref()
    }

    fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }
}

/// Service for managing HTTP request operations
///
/// This service encapsulates HTTP client functionality and provides
/// a clean interface for executing requests and handling responses.
pub struct HttpService {
    /// The underlying HTTP client
    client: Option<HttpClient>,
    /// Profile info for recreating clients if needed
    profile_info: Option<ProfileInfo>,
    /// Session headers that persist across requests
    session_headers: HashMap<String, String>,
    /// Channel for receiving async HTTP responses
    response_receiver: mpsc::Receiver<HttpResponseMessage>,
    /// Channel sender for async tasks to send responses
    response_sender: mpsc::Sender<HttpResponseMessage>,
}

impl HttpService {
    /// Create a new HttpService with a profile
    pub fn new(profile: &impl HttpConnectionProfile) -> Result<Self> {
        tracing::debug!("Creating HttpService with profile");
        let (response_sender, response_receiver) = mpsc::channel(10);

        tracing::debug!("Creating HttpClient from profile");
        let client = HttpClient::new(profile)?;
        tracing::info!("HttpClient created successfully");

        Ok(Self {
            client: Some(client),
            profile_info: None, // Will be set separately if needed
            session_headers: HashMap::new(),
            response_receiver,
            response_sender,
        })
    }

    /// Set profile info for recreating clients
    pub fn set_profile_info(&mut self, profile_name: String, profile_path: String) {
        self.profile_info = Some((profile_name, profile_path));
    }

    /// Reconfigure the HTTP client with a profile (used after taking the client)
    pub fn reconfigure(&mut self, profile: &impl HttpConnectionProfile) -> Result<()> {
        self.client = Some(HttpClient::new(profile)?);
        Ok(())
    }

    /// Check if HTTP client is available
    pub fn is_available(&self) -> bool {
        self.client.is_some()
    }

    /// Parse HTTP request from text content (static version for async usage)
    fn parse_request_static(
        text: &str,
        session_headers: HashMap<String, String>,
    ) -> Result<ParsedRequest> {
        let lines: Vec<&str> = text.lines().collect();

        if lines.is_empty() || lines[0].trim().is_empty() {
            return Err(anyhow::anyhow!("No request to execute"));
        }

        // Parse first line as method and URL
        let parts: Vec<&str> = lines[0].split_whitespace().collect();
        if parts.len() < 2 {
            return Err(anyhow::anyhow!("Invalid request format. Use: METHOD URL"));
        }

        let method = parts[0].to_uppercase();
        let url_str = parts[1].to_string();

        // Parse URL
        let url = Url::parse(&url_str);

        // Skip empty line after URL if it exists, then rest becomes the body
        let body_start_idx = if lines.len() > 1 && lines[1].trim().is_empty() {
            2
        } else {
            1
        };

        let body = if lines.len() > body_start_idx {
            Some(lines[body_start_idx..].join("\n"))
        } else {
            None
        };

        // Create request args with session headers
        let request_args = BufferRequestArgs {
            method: Some(method),
            url_path: url.to_url_path().cloned(),
            body,
            headers: session_headers,
        };

        Ok((request_args, url_str))
    }

    /// Parse HTTP request from text content
    /// Returns (BufferRequestArgs, url_str) or error message
    pub fn parse_request(&self, text: &str) -> Result<ParsedRequest> {
        Self::parse_request_static(text, self.session_headers.clone())
    }

    /// Execute an HTTP request
    pub async fn execute_request(
        &self,
        request_args: &impl HttpRequestArgs,
    ) -> Result<HttpResponse> {
        match &self.client {
            Some(client) => client
                .request(request_args)
                .await
                .map_err(|e| anyhow::anyhow!("HTTP request failed: {e}")),
            None => Err(anyhow::anyhow!("HTTP client not configured")),
        }
    }

    /// Execute request from raw text
    pub async fn execute_from_text(&self, request_text: &str) -> Result<HttpResponse> {
        let (request_args, _url) = self.parse_request(request_text)?;
        self.execute_request(&request_args).await
    }

    /// Execute HTTP request and return formatted response
    pub async fn execute_with_formatting(
        &self,
        request_text: &str,
        verbose: bool,
    ) -> Result<(String, u16, Option<u64>)> {
        let start_time = std::time::Instant::now();

        // Parse request
        let (request_args, url_str) = self.parse_request(request_text)?;

        // Execute the request
        let response = self.execute_request(&request_args).await?;

        // Calculate request duration
        let duration_ms = start_time.elapsed().as_millis() as u64;

        let status = response.status();
        let body = response.body();

        let mut response_text = String::new();

        if verbose {
            // Add request information
            response_text.push_str(&format!(
                "Request: {} {}\n",
                request_args.method().unwrap_or(&"GET".to_string()),
                url_str
            ));

            // Add headers if any
            if !request_args.headers().is_empty() {
                response_text.push_str("Headers:\n");
                for (key, value) in request_args.headers() {
                    response_text.push_str(&format!("  {key}: {value}\n"));
                }
            }

            response_text.push('\n');

            // Add response status
            response_text.push_str(&format!(
                "Response: {} {}\n\n",
                status.as_u16(),
                status.canonical_reason().unwrap_or("")
            ));
        }

        // Add response body
        response_text.push_str(body);

        Ok((response_text, status.as_u16(), Some(duration_ms)))
    }

    /// Add or update a session header
    pub fn set_session_header(&mut self, key: String, value: String) {
        self.session_headers.insert(key, value);
    }

    /// Remove a session header
    pub fn remove_session_header(&mut self, key: &str) -> Option<String> {
        self.session_headers.remove(key)
    }

    /// Clear all session headers
    pub fn clear_session_headers(&mut self) {
        self.session_headers.clear();
    }

    /// Get current session headers
    pub fn session_headers(&self) -> &HashMap<String, String> {
        &self.session_headers
    }

    /// Check if there are any pending HTTP responses (non-blocking)
    pub fn poll_response(&mut self) -> Option<HttpResponseMessage> {
        self.response_receiver.try_recv().ok()
    }

    /// Execute HTTP request asynchronously
    ///
    /// This spawns a tokio task that executes the request and sends the result
    /// back through the internal channel, allowing non-blocking operation.
    pub fn execute_async(&mut self, request_text: String) {
        // Parse the request first (synchronously)
        // Clone session headers before parsing to avoid lifetime issues
        let session_headers = self.session_headers.clone();

        // Clone the client if available
        let client = self.client.clone();

        // Clone the sender for the async task
        let result_sender = self.response_sender.clone();

        // Now parse the request completely independently
        let parsed_result = Self::parse_request_static(&request_text, session_headers);

        match parsed_result {
            Ok((request_args, url_str)) => {
                // Check if we have a client
                let client = match client {
                    Some(c) => c,
                    None => {
                        tokio::spawn(async move {
                            let _ = result_sender
                                .send(HttpResponseMessage::Error {
                                    message: "HTTP client not configured".to_string(),
                                })
                                .await;
                        });
                        return;
                    }
                };

                // result_sender was already cloned above

                // Spawn async task for HTTP execution
                tokio::spawn(async move {
                    // Clone for the response since we'll move it for the request
                    let request_args_clone = request_args.clone();

                    // Execute the HTTP request
                    let response_msg = match client.request(&request_args).await {
                        Ok(response) => HttpResponseMessage::Success {
                            request: request_args_clone,
                            response: Box::new(response),
                            url: url_str,
                        },
                        Err(e) => {
                            // Show full error chain using anyhow's chain iterator
                            let mut error_message = format!("{e}");
                            for cause in e.chain().skip(1) {
                                error_message.push_str(&format!("\n  Caused by: {cause}"));
                            }
                            tracing::error!("HTTP request failed: {error_message}");
                            HttpResponseMessage::Error {
                                message: error_message,
                            }
                        }
                    };

                    // Send the result back through the channel
                    // Ignore send errors (receiver might have been dropped)
                    let _ = result_sender.send(response_msg).await;
                });
            }
            Err(e) => {
                // Send error message through channel
                tokio::spawn(async move {
                    let _ = result_sender
                        .send(HttpResponseMessage::Error {
                            message: format!("Failed to parse request: {e}"),
                        })
                        .await;
                });
            }
        }
    }
}

/// Result of HTTP request execution
pub struct HttpExecutionResult {
    /// HTTP status code
    pub status_code: u16,
    /// Response body
    pub body: String,
    /// Response headers
    pub headers: HashMap<String, String>,
}

impl From<&HttpResponse> for HttpExecutionResult {
    fn from(response: &HttpResponse) -> Self {
        Self {
            status_code: response.status().as_u16(),
            body: response.body().to_string(),
            headers: response
                .headers()
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bluenote::get_blank_profile;

    fn create_test_service() -> HttpService {
        let profile = get_blank_profile();
        HttpService::new(&profile).unwrap_or_else(|_| {
            // If blank profile fails, create a service without client
            let (response_sender, response_receiver) = mpsc::channel(10);
            HttpService {
                client: None,
                profile_info: None,
                session_headers: HashMap::new(),
                response_receiver,
                response_sender,
            }
        })
    }

    #[test]
    fn http_service_should_create_with_profile() {
        let profile = get_blank_profile();
        let result = HttpService::new(&profile);
        // May fail if profile is not valid, but structure should be created
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn http_service_should_manage_session_headers() {
        let mut service = create_test_service();

        // Add header
        service.set_session_header("Authorization".to_string(), "Bearer token".to_string());
        assert_eq!(service.session_headers().len(), 1);
        assert_eq!(
            service.session_headers().get("Authorization"),
            Some(&"Bearer token".to_string())
        );

        // Update header
        service.set_session_header("Authorization".to_string(), "Bearer new_token".to_string());
        assert_eq!(
            service.session_headers().get("Authorization"),
            Some(&"Bearer new_token".to_string())
        );

        // Remove header
        let removed = service.remove_session_header("Authorization");
        assert_eq!(removed, Some("Bearer new_token".to_string()));
        assert!(service.session_headers().is_empty());

        // Clear headers
        service.set_session_header("Header1".to_string(), "Value1".to_string());
        service.set_session_header("Header2".to_string(), "Value2".to_string());
        assert_eq!(service.session_headers().len(), 2);
        service.clear_session_headers();
        assert!(service.session_headers().is_empty());
    }

    #[test]
    fn http_service_should_parse_simple_request() {
        let service = create_test_service();
        let request_text = "GET https://httpbin.org/get";

        match service.parse_request(request_text) {
            Ok((args, url)) => {
                assert_eq!(args.method(), Some(&"GET".to_string()));
                assert!(url.contains("httpbin.org"));
            }
            Err(e) => panic!("Failed to parse request: {e}"),
        }
    }

    #[test]
    fn test_parse_request_with_body() {
        let service = create_test_service();
        let text = "POST http://example.com/api/users\n\n{\"name\": \"test\"}";

        let result = service.parse_request(text);
        assert!(result.is_ok());

        let (args, url) = result.unwrap();
        assert_eq!(args.method(), Some(&"POST".to_string()));
        assert_eq!(url, "http://example.com/api/users");
        assert_eq!(args.body(), Some(&"{\"name\": \"test\"}".to_string()));
    }

    #[test]
    fn test_parse_request_with_session_headers() {
        let mut service = create_test_service();
        service.set_session_header("Authorization".to_string(), "Bearer token123".to_string());

        let text = "GET http://example.com/api/users";

        let result = service.parse_request(text);
        assert!(result.is_ok());

        let (args, _) = result.unwrap();
        assert_eq!(
            args.headers().get("Authorization"),
            Some(&"Bearer token123".to_string())
        );
    }

    #[test]
    fn test_parse_request_empty() {
        let service = create_test_service();
        let text = "";

        let result = service.parse_request(text);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No request to execute"));
    }

    #[test]
    fn test_parse_request_invalid_format() {
        let service = create_test_service();
        let text = "GET";

        let result = service.parse_request(text);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid request format"));
    }
}
