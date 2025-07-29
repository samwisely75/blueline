//! # HTTP Request Model
//!
//! Model for storing HTTP request data including method, URL, headers, and body.

/// Type alias for HTTP headers to reduce complexity
pub type HttpHeaders = Vec<(String, String)>;

/// HTTP request model
#[derive(Debug, Clone)]
pub struct RequestModel {
    method: String,
    url: String,
    headers: HttpHeaders,
    body: String,
}

impl RequestModel {
    pub fn new() -> Self {
        Self {
            method: "GET".to_string(),
            url: String::new(),
            headers: Vec::new(),
            body: String::new(),
        }
    }

    pub fn method(&self) -> &str {
        &self.method
    }

    pub fn set_method(&mut self, method: String) {
        self.method = method;
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn set_url(&mut self, url: String) {
        self.url = url;
    }

    pub fn headers(&self) -> &HttpHeaders {
        &self.headers
    }

    pub fn add_header(&mut self, key: String, value: String) {
        self.headers.push((key, value));
    }

    pub fn body(&self) -> &str {
        &self.body
    }

    pub fn set_body(&mut self, body: String) {
        self.body = body;
    }
}

impl Default for RequestModel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_model_should_create_with_defaults() {
        let request = RequestModel::new();

        assert_eq!(request.method(), "GET");
        assert!(request.url().is_empty());
        assert!(request.headers().is_empty());
        assert!(request.body().is_empty());
    }

    #[test]
    fn request_model_should_set_method() {
        let mut request = RequestModel::new();

        request.set_method("POST".to_string());

        assert_eq!(request.method(), "POST");
    }

    #[test]
    fn request_model_should_add_header() {
        let mut request = RequestModel::new();

        request.add_header("Content-Type".to_string(), "application/json".to_string());

        assert_eq!(request.headers().len(), 1);
        assert_eq!(
            request.headers()[0],
            ("Content-Type".to_string(), "application/json".to_string())
        );
    }
}
