//! # HTTP Response Model
//!
//! Model for storing HTTP response data including status code, headers, and body.

use super::request_model::HttpHeaders;

/// HTTP response model
#[derive(Debug, Clone)]
pub struct ResponseModel {
    status_code: Option<u16>,
    headers: HttpHeaders,
    body: String,
}

impl ResponseModel {
    pub fn new() -> Self {
        Self {
            status_code: None,
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
