//! Response model for MVVM architecture
//!
//! This model manages HTTP response state including status, headers, body, and timing.
//! It handles response processing and formatting without any view concerns.

use crate::repl::events::ModelEvent;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// HTTP response status information
#[derive(Debug, Clone, PartialEq)]
pub struct ResponseStatus {
    pub code: u16,
    pub reason: String,
}

impl ResponseStatus {
    /// Create a new response status
    pub fn new(code: u16, reason: String) -> Self {
        Self { code, reason }
    }

    /// Check if the status indicates success (2xx)
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.code)
    }

    /// Check if the status indicates client error (4xx)
    pub fn is_client_error(&self) -> bool {
        (400..500).contains(&self.code)
    }

    /// Check if the status indicates server error (5xx)
    pub fn is_server_error(&self) -> bool {
        (500..600).contains(&self.code)
    }

    /// Get status as string (e.g., "200 OK")
    pub fn as_string(&self) -> String {
        format!("{} {}", self.code, self.reason)
    }
}

/// HTTP response timing information
#[derive(Debug, Clone)]
pub struct ResponseTiming {
    pub start_time: Option<Instant>,
    pub end_time: Option<Instant>,
    pub duration: Option<Duration>,
}

impl ResponseTiming {
    /// Create new timing information
    pub fn new() -> Self {
        Self {
            start_time: None,
            end_time: None,
            duration: None,
        }
    }

    /// Mark the start of the request
    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
        self.end_time = None;
        self.duration = None;
    }

    /// Mark the end of the request and calculate duration
    pub fn finish(&mut self) {
        self.end_time = Some(Instant::now());
        if let Some(start) = self.start_time {
            self.duration = Some(start.elapsed());
        }
    }

    /// Get duration in milliseconds
    pub fn duration_ms(&self) -> Option<u64> {
        self.duration.map(|d| d.as_millis() as u64)
    }
}

impl Default for ResponseTiming {
    fn default() -> Self {
        Self::new()
    }
}

/// HTTP response model containing all response data
#[derive(Debug, Clone)]
pub struct ResponseModel {
    /// HTTP response status
    pub status: Option<ResponseStatus>,
    /// Response headers as key-value pairs
    pub headers: HashMap<String, String>,
    /// Response body content
    pub body: String,
    /// Response timing information
    pub timing: ResponseTiming,
    /// Whether a response is currently being received
    pub is_receiving: bool,
    /// Any error that occurred during the request
    pub error: Option<String>,
}

impl ResponseModel {
    /// Create a new response model with default values
    pub fn new() -> Self {
        Self {
            status: None,
            headers: HashMap::new(),
            body: String::new(),
            timing: ResponseTiming::new(),
            is_receiving: false,
            error: None,
        }
    }

    /// Clear the response data for a new request
    pub fn clear(&mut self) {
        self.status = None;
        self.headers.clear();
        self.body.clear();
        self.timing = ResponseTiming::new();
        self.is_receiving = false;
        self.error = None;
    }

    /// Start receiving a response
    pub fn start_receiving(&mut self) {
        self.clear();
        self.is_receiving = true;
        self.timing.start();
    }

    /// Set the response status
    pub fn set_status(&mut self, status: ResponseStatus) {
        self.status = Some(status);
    }

    /// Add or update a response header
    pub fn set_header(&mut self, key: String, value: String) {
        self.headers.insert(key, value);
    }

    /// Set the response body
    pub fn set_body(&mut self, body: String) {
        self.body = body;
    }

    /// Complete the response and return event
    pub fn finish_receiving(&mut self) -> Option<ModelEvent> {
        if self.is_receiving {
            self.is_receiving = false;
            self.timing.finish();

            if let Some(status) = &self.status {
                Some(ModelEvent::ResponseReceived {
                    status: status.as_string(),
                    body: self.body.clone(),
                })
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Set an error that occurred during the request
    pub fn set_error(&mut self, error: String) {
        self.error = Some(error);
        self.is_receiving = false;
        self.timing.finish();
    }

    /// Get content type from response headers
    pub fn content_type(&self) -> Option<&String> {
        self.headers
            .get("Content-Type")
            .or_else(|| self.headers.get("content-type"))
    }

    /// Check if the response has any data
    pub fn has_data(&self) -> bool {
        self.status.is_some() || !self.body.is_empty() || self.error.is_some()
    }

    /// Check if the response was successful
    pub fn is_success(&self) -> bool {
        self.error.is_none() && self.status.as_ref().is_some_and(|s| s.is_success())
    }

    /// Get response size in bytes
    pub fn content_length(&self) -> usize {
        self.body.len()
    }

    /// Parse response from HTTP text format
    pub fn parse_from_text(text: &str) -> Result<Self, String> {
        let lines: Vec<&str> = text.lines().collect();
        if lines.is_empty() {
            return Err("Empty response text".to_string());
        }

        // Parse status line (HTTP/1.1 200 OK)
        let status_line = lines[0];
        let parts: Vec<&str> = status_line.split_whitespace().collect();
        if parts.len() < 3 {
            return Err("Invalid status line format".to_string());
        }

        let code: u16 = parts[1]
            .parse()
            .map_err(|_| format!("Invalid status code: {}", parts[1]))?;
        let reason = parts[2..].join(" ");

        let status = ResponseStatus::new(code, reason);
        let mut headers = HashMap::new();
        let mut body_start = lines.len();

        // Parse headers
        for (i, line) in lines.iter().enumerate().skip(1) {
            if line.trim().is_empty() {
                body_start = i + 1;
                break;
            }

            if let Some(colon_pos) = line.find(':') {
                let key = line[..colon_pos].trim().to_string();
                let value = line[colon_pos + 1..].trim().to_string();
                headers.insert(key, value);
            } else {
                return Err(format!("Invalid header format: {}", line));
            }
        }

        // Parse body (everything after empty line)
        let body = if body_start < lines.len() {
            lines[body_start..].join("\n")
        } else {
            String::new()
        };

        Ok(Self {
            status: Some(status),
            headers,
            body,
            timing: ResponseTiming::new(),
            is_receiving: false,
            error: None,
        })
    }

    /// Convert response to HTTP text format
    pub fn to_http_text(&self) -> String {
        let mut result = String::new();

        // Add status line
        if let Some(status) = &self.status {
            result.push_str(&format!("HTTP/1.1 {}\n", status.as_string()));
        }

        // Add headers
        for (key, value) in &self.headers {
            result.push_str(&format!("{}: {}\n", key, value));
        }

        // Add empty line before body
        if !self.body.is_empty() {
            result.push('\n');
            result.push_str(&self.body);
        }

        result
    }

    /// Get a formatted summary of the response
    pub fn summary(&self) -> String {
        match (&self.status, &self.error) {
            (Some(status), None) => {
                let duration = self
                    .timing
                    .duration_ms()
                    .map(|ms| format!(" ({}ms)", ms))
                    .unwrap_or_default();
                format!(
                    "{} - {} bytes{}",
                    status.as_string(),
                    self.content_length(),
                    duration
                )
            }
            (None, Some(error)) => format!("Error: {}", error),
            (Some(status), Some(error)) => format!("{} - Error: {}", status.as_string(), error),
            (None, None) => "No response".to_string(),
        }
    }
}

impl Default for ResponseModel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn response_model_should_create_with_defaults() {
        let model = ResponseModel::new();

        assert!(model.status.is_none());
        assert!(model.headers.is_empty());
        assert_eq!(model.body, "");
        assert!(!model.is_receiving);
        assert!(model.error.is_none());
        assert!(!model.has_data());
    }

    #[test]
    fn response_status_should_classify_codes() {
        let success = ResponseStatus::new(200, "OK".to_string());
        assert!(success.is_success());
        assert!(!success.is_client_error());
        assert!(!success.is_server_error());

        let client_error = ResponseStatus::new(404, "Not Found".to_string());
        assert!(!client_error.is_success());
        assert!(client_error.is_client_error());
        assert!(!client_error.is_server_error());

        let server_error = ResponseStatus::new(500, "Internal Server Error".to_string());
        assert!(!server_error.is_success());
        assert!(!server_error.is_client_error());
        assert!(server_error.is_server_error());
    }

    #[test]
    fn response_timing_should_track_duration() {
        let mut timing = ResponseTiming::new();

        timing.start();
        assert!(timing.start_time.is_some());
        assert!(timing.end_time.is_none());

        // Small delay to ensure measurable duration
        std::thread::sleep(Duration::from_millis(1));

        timing.finish();
        assert!(timing.end_time.is_some());
        assert!(timing.duration.is_some());
        assert!(timing.duration_ms().unwrap() > 0);
    }

    #[test]
    fn response_model_should_handle_lifecycle() {
        let mut model = ResponseModel::new();

        model.start_receiving();
        assert!(model.is_receiving);
        assert!(model.timing.start_time.is_some());

        model.set_status(ResponseStatus::new(200, "OK".to_string()));
        model.set_body("response body".to_string());

        let event = model.finish_receiving().unwrap();
        assert!(!model.is_receiving);
        assert!(model.timing.duration.is_some());

        match event {
            ModelEvent::ResponseReceived { status, body } => {
                assert_eq!(status, "200 OK");
                assert_eq!(body, "response body");
            }
            _ => panic!("Expected ResponseReceived event"),
        }
    }

    #[test]
    fn response_model_should_handle_errors() {
        let mut model = ResponseModel::new();

        model.start_receiving();
        model.set_error("Connection timeout".to_string());

        assert!(!model.is_receiving);
        assert_eq!(model.error, Some("Connection timeout".to_string()));
        assert!(!model.is_success());
    }

    #[test]
    fn response_model_should_parse_http_text() {
        let http_text =
            "HTTP/1.1 200 OK\nContent-Type: application/json\nContent-Length: 13\n\n{\"ok\": true}";

        let model = ResponseModel::parse_from_text(http_text).unwrap();

        assert_eq!(model.status.unwrap().code, 200);
        assert_eq!(
            model.headers.get("Content-Type"),
            Some(&"application/json".to_string())
        );
        assert_eq!(model.body, "{\"ok\": true}");
    }

    #[test]
    fn response_model_should_convert_to_http_text() {
        let mut model = ResponseModel::new();
        model.set_status(ResponseStatus::new(200, "OK".to_string()));
        model.set_header("Content-Type".to_string(), "application/json".to_string());
        model.set_body("{\"ok\": true}".to_string());

        let http_text = model.to_http_text();

        assert!(http_text.contains("HTTP/1.1 200 OK"));
        assert!(http_text.contains("Content-Type: application/json"));
        assert!(http_text.contains("{\"ok\": true}"));
    }

    #[test]
    fn response_model_should_provide_summary() {
        let mut model = ResponseModel::new();

        // Test successful response
        model.set_status(ResponseStatus::new(200, "OK".to_string()));
        model.set_body("test".to_string());
        let summary = model.summary();
        assert!(summary.contains("200 OK"));
        assert!(summary.contains("4 bytes"));

        // Test error response
        model.set_error("Connection failed".to_string());
        let summary = model.summary();
        assert!(summary.contains("Error: Connection failed"));
    }

    #[test]
    fn response_model_should_handle_invalid_http_text() {
        assert!(ResponseModel::parse_from_text("").is_err());
        assert!(ResponseModel::parse_from_text("INVALID").is_err());
        assert!(ResponseModel::parse_from_text("HTTP/1.1 INVALID OK").is_err());
    }
}
