//! # HTTP Response Model
//!
//! Model for storing HTTP response data including status code, headers, and body.

use super::request_model::HttpHeaders;

/// HTTP response model
#[derive(Debug, Clone)]
pub struct ResponseModel {
    status_code: Option<u16>,
    status_message: Option<String>,
    duration_ms: Option<u64>,
    headers: HttpHeaders,
    body: String,
}

impl ResponseModel {
    pub fn new() -> Self {
        Self {
            status_code: None,
            status_message: None,
            duration_ms: None,
            headers: Vec::new(),
            body: String::new(),
        }
    }

    pub fn status_code(&self) -> Option<u16> {
        self.status_code
    }

    pub fn set_status_code(&mut self, status_code: u16) {
        self.status_code = Some(status_code);
    }

    pub fn status_message(&self) -> Option<&String> {
        self.status_message.as_ref()
    }

    pub fn set_status_message(&mut self, status_message: String) {
        self.status_message = Some(status_message);
    }

    pub fn duration_ms(&self) -> Option<u64> {
        self.duration_ms
    }

    pub fn set_duration_ms(&mut self, duration_ms: u64) {
        self.duration_ms = Some(duration_ms);
    }

    pub fn headers(&self) -> &HttpHeaders {
        &self.headers
    }

    pub fn set_headers(&mut self, headers: HttpHeaders) {
        self.headers = headers;
    }

    pub fn body(&self) -> &str {
        &self.body
    }

    pub fn set_body(&mut self, body: String) {
        self.body = body;
    }

    pub fn clear(&mut self) {
        self.status_code = None;
        self.status_message = None;
        self.duration_ms = None;
        self.headers.clear();
        self.body.clear();
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
        let response = ResponseModel::new();

        assert_eq!(response.status_code(), None);
        assert!(response.headers().is_empty());
        assert!(response.body().is_empty());
    }

    #[test]
    fn response_model_should_set_status_code() {
        let mut response = ResponseModel::new();

        response.set_status_code(200);

        assert_eq!(response.status_code(), Some(200));
    }

    #[test]
    fn response_model_should_clear() {
        let mut response = ResponseModel::new();
        response.set_status_code(200);
        response.set_body("test".to_string());

        response.clear();

        assert_eq!(response.status_code(), None);
        assert!(response.body().is_empty());
    }
}
