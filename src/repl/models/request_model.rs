//! Request model for MVVM architecture
//!
//! This model manages HTTP request state including method, URL, headers, and body.
//! It handles request building and validation without any view concerns.

use crate::repl::events::ModelEvent;
use std::collections::HashMap;

/// HTTP method types supported by the client
#[derive(Debug, Clone, PartialEq)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    HEAD,
    OPTIONS,
}

impl HttpMethod {
    /// Parse HTTP method from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "GET" => Some(Self::GET),
            "POST" => Some(Self::POST),
            "PUT" => Some(Self::PUT),
            "DELETE" => Some(Self::DELETE),
            "PATCH" => Some(Self::PATCH),
            "HEAD" => Some(Self::HEAD),
            "OPTIONS" => Some(Self::OPTIONS),
            _ => None,
        }
    }
    
    /// Convert to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::GET => "GET",
            Self::POST => "POST",
            Self::PUT => "PUT",
            Self::DELETE => "DELETE",
            Self::PATCH => "PATCH",
            Self::HEAD => "HEAD",
            Self::OPTIONS => "OPTIONS",
        }
    }
}

impl Default for HttpMethod {
    fn default() -> Self {
        Self::GET
    }
}

/// HTTP request model containing all request data
#[derive(Debug, Clone)]
pub struct RequestModel {
    /// HTTP method for the request
    pub method: HttpMethod,
    /// Target URL for the request
    pub url: String,
    /// HTTP headers as key-value pairs
    pub headers: HashMap<String, String>,
    /// Request body content
    pub body: String,
    /// Whether the request is currently being executed
    pub is_executing: bool,
}

impl RequestModel {
    /// Create a new request model with default values
    pub fn new() -> Self {
        Self {
            method: HttpMethod::default(),
            url: String::new(),
            headers: HashMap::new(),
            body: String::new(),
            is_executing: false,
        }
    }
    
    /// Set the HTTP method
    pub fn set_method(&mut self, method: HttpMethod) {
        self.method = method;
    }
    
    /// Set the request URL
    pub fn set_url(&mut self, url: String) {
        self.url = url;
    }
    
    /// Add or update a header
    pub fn set_header(&mut self, key: String, value: String) {
        self.headers.insert(key, value);
    }
    
    /// Remove a header
    pub fn remove_header(&mut self, key: &str) -> Option<String> {
        self.headers.remove(key)
    }
    
    /// Clear all headers
    pub fn clear_headers(&mut self) {
        self.headers.clear();
    }
    
    /// Set the request body
    pub fn set_body(&mut self, body: String) {
        self.body = body;
    }
    
    /// Clear the request body
    pub fn clear_body(&mut self) {
        self.body.clear();
    }
    
    /// Mark request as executing and return event
    pub fn start_execution(&mut self) -> Option<ModelEvent> {
        if !self.is_executing {
            self.is_executing = true;
            Some(ModelEvent::RequestExecuted)
        } else {
            None
        }
    }
    
    /// Mark request as finished executing
    pub fn finish_execution(&mut self) {
        self.is_executing = false;
    }
    
    /// Check if the request has required fields for execution
    pub fn is_valid(&self) -> bool {
        !self.url.is_empty()
    }
    
    /// Get content type from headers
    pub fn content_type(&self) -> Option<&String> {
        self.headers.get("Content-Type")
            .or_else(|| self.headers.get("content-type"))
    }
    
    /// Set content type header
    pub fn set_content_type(&mut self, content_type: String) {
        self.headers.insert("Content-Type".to_string(), content_type);
    }
    
    /// Parse request from HTTP text format
    /// Returns the parsed request and any parsing errors
    pub fn parse_from_text(text: &str) -> Result<Self, String> {
        let lines: Vec<&str> = text.lines().collect();
        if lines.is_empty() {
            return Err("Empty request text".to_string());
        }
        
        // Parse request line (METHOD URL)
        let request_line = lines[0];
        let parts: Vec<&str> = request_line.split_whitespace().collect();
        if parts.len() < 2 {
            return Err("Invalid request line format".to_string());
        }
        
        let method = HttpMethod::from_str(parts[0])
            .ok_or_else(|| format!("Unknown HTTP method: {}", parts[0]))?;
        let url = parts[1].to_string();
        
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
            method,
            url,
            headers,
            body,
            is_executing: false,
        })
    }
    
    /// Convert request to HTTP text format
    pub fn to_http_text(&self) -> String {
        let mut result = format!("{} {}\n", self.method.as_str(), self.url);
        
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
        let model = RequestModel::new();
        
        assert_eq!(model.method, HttpMethod::GET);
        assert_eq!(model.url, "");
        assert!(model.headers.is_empty());
        assert_eq!(model.body, "");
        assert!(!model.is_executing);
    }

    #[test]
    fn http_method_should_parse_from_string() {
        assert_eq!(HttpMethod::from_str("GET"), Some(HttpMethod::GET));
        assert_eq!(HttpMethod::from_str("post"), Some(HttpMethod::POST));
        assert_eq!(HttpMethod::from_str("PUT"), Some(HttpMethod::PUT));
        assert_eq!(HttpMethod::from_str("invalid"), None);
    }

    #[test]
    fn http_method_should_convert_to_string() {
        assert_eq!(HttpMethod::GET.as_str(), "GET");
        assert_eq!(HttpMethod::POST.as_str(), "POST");
        assert_eq!(HttpMethod::DELETE.as_str(), "DELETE");
    }

    #[test]
    fn request_model_should_manage_headers() {
        let mut model = RequestModel::new();
        
        model.set_header("Content-Type".to_string(), "application/json".to_string());
        assert_eq!(model.headers.get("Content-Type"), Some(&"application/json".to_string()));
        
        model.set_content_type("text/plain".to_string());
        assert_eq!(model.content_type(), Some(&"text/plain".to_string()));
        
        model.remove_header("Content-Type");
        assert!(model.headers.is_empty());
    }

    #[test]
    fn request_model_should_validate() {
        let mut model = RequestModel::new();
        assert!(!model.is_valid()); // No URL
        
        model.set_url("https://api.example.com".to_string());
        assert!(model.is_valid());
    }

    #[test]
    fn request_model_should_handle_execution_state() {
        let mut model = RequestModel::new();
        
        let event = model.start_execution().unwrap();
        assert!(model.is_executing);
        assert_eq!(event, ModelEvent::RequestExecuted);
        
        // Starting execution again should return None
        assert!(model.start_execution().is_none());
        
        model.finish_execution();
        assert!(!model.is_executing);
    }

    #[test]
    fn request_model_should_parse_http_text() {
        let http_text = "POST https://api.example.com/users\nContent-Type: application/json\nAuthorization: Bearer token123\n\n{\"name\": \"John\"}";
        
        let model = RequestModel::parse_from_text(http_text).unwrap();
        
        assert_eq!(model.method, HttpMethod::POST);
        assert_eq!(model.url, "https://api.example.com/users");
        assert_eq!(model.headers.get("Content-Type"), Some(&"application/json".to_string()));
        assert_eq!(model.headers.get("Authorization"), Some(&"Bearer token123".to_string()));
        assert_eq!(model.body, "{\"name\": \"John\"}");
    }

    #[test]
    fn request_model_should_convert_to_http_text() {
        let mut model = RequestModel::new();
        model.set_method(HttpMethod::POST);
        model.set_url("https://api.example.com/users".to_string());
        model.set_header("Content-Type".to_string(), "application/json".to_string());
        model.set_body("{\"name\": \"John\"}".to_string());
        
        let http_text = model.to_http_text();
        
        assert!(http_text.contains("POST https://api.example.com/users"));
        assert!(http_text.contains("Content-Type: application/json"));
        assert!(http_text.contains("{\"name\": \"John\"}"));
    }

    #[test]
    fn request_model_should_handle_invalid_http_text() {
        assert!(RequestModel::parse_from_text("").is_err());
        assert!(RequestModel::parse_from_text("INVALID").is_err());
        assert!(RequestModel::parse_from_text("INVALID_METHOD https://example.com").is_err());
    }
}