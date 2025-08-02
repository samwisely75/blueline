//! # HTTP Integration Module
//!
//! This module provides HTTP client functionality using bluenote library.
//! It handles request parsing, execution, and response formatting.

use anyhow::Result;
use bluenote::{get_blank_profile, HttpClient, HttpRequestArgs, IniProfile, Url, UrlPath};
use std::collections::HashMap;

/// HTTP request arguments parsed from the request buffer
#[derive(Debug)]
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

/// Type alias for the result of parsing HTTP requests from text
/// Returns (request_args, url_string) on success, or error message on failure
pub type ParseRequestResult = Result<(BufferRequestArgs, String), String>;

/// Parse HTTP request from text content
/// Returns (BufferRequestArgs, url_str) or error message
pub fn parse_request_from_text(
    text: &str,
    session_headers: &HashMap<String, String>,
) -> ParseRequestResult {
    let lines: Vec<&str> = text.lines().collect();

    if lines.is_empty() || lines[0].trim().is_empty() {
        return Err("No request to execute".to_string());
    }

    // Parse first line as method and URL
    let parts: Vec<&str> = lines[0].split_whitespace().collect();
    if parts.len() < 2 {
        return Err("Invalid request format. Use: METHOD URL".to_string());
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

    // Create request args
    let request_args = BufferRequestArgs {
        method: Some(method),
        url_path: url.to_url_path().cloned(),
        body,
        headers: session_headers.clone(),
    };

    Ok((request_args, url_str))
}

/// Create a default HTTP profile for cases where no profile is configured
pub fn create_default_profile() -> IniProfile {
    // Set a default server if needed
    // For now, we'll leave it blank and handle missing server in the client
    get_blank_profile()
}

/// Execute HTTP request and return formatted response
pub async fn execute_http_request(
    client: &HttpClient,
    request_args: &BufferRequestArgs,
    url_str: &str,
    verbose: bool,
) -> Result<(String, u16, Option<u64>)> {
    let start_time = std::time::Instant::now();

    // Execute the request
    let response = client.request(request_args).await?;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_request_simple() {
        let text = "GET http://example.com/api/users";
        let headers = HashMap::new();

        let result = parse_request_from_text(text, &headers);
        assert!(result.is_ok());

        let (args, url) = result.unwrap();
        assert_eq!(args.method(), Some(&"GET".to_string()));
        assert_eq!(url, "http://example.com/api/users");
        assert!(args.body().is_none());
    }

    #[test]
    fn test_parse_request_with_body() {
        let text = "POST http://example.com/api/users\n\n{\"name\": \"test\"}";
        let headers = HashMap::new();

        let result = parse_request_from_text(text, &headers);
        assert!(result.is_ok());

        let (args, url) = result.unwrap();
        assert_eq!(args.method(), Some(&"POST".to_string()));
        assert_eq!(url, "http://example.com/api/users");
        assert_eq!(args.body(), Some(&"{\"name\": \"test\"}".to_string()));
    }

    #[test]
    fn test_parse_request_with_headers() {
        let text = "GET http://example.com/api/users";
        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), "Bearer token123".to_string());

        let result = parse_request_from_text(text, &headers);
        assert!(result.is_ok());

        let (args, _) = result.unwrap();
        assert_eq!(
            args.headers().get("Authorization"),
            Some(&"Bearer token123".to_string())
        );
    }

    #[test]
    fn test_parse_request_empty() {
        let text = "";
        let headers = HashMap::new();

        let result = parse_request_from_text(text, &headers);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "No request to execute");
    }

    #[test]
    fn test_parse_request_invalid_format() {
        let text = "GET";
        let headers = HashMap::new();

        let result = parse_request_from_text(text, &headers);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Invalid request format. Use: METHOD URL"
        );
    }
}
