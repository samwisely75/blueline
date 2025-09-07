//! # HTTP Context
//!
//! Manages HTTP-specific state and operations including:
//! - Session headers management
//! - Request/response state coordination
//! - HTTP execution status
//!
//! This provides a clean separation of HTTP concerns from editor and UI domains.

use std::collections::HashMap;

/// HTTP context managing HTTP-specific state and operations
///
/// This context encapsulates all HTTP-related functionality that was previously
/// scattered throughout AppState, providing cleaner domain separation.
#[derive(Debug)]
pub struct HttpContext {
    /// HTTP session headers that persist across requests
    session_headers: HashMap<String, String>,
}

impl HttpContext {
    /// Create a new HttpContext with empty session state
    pub fn new() -> Self {
        Self {
            session_headers: HashMap::new(),
        }
    }

    /// Get session headers
    pub fn session_headers(&self) -> &HashMap<String, String> {
        &self.session_headers
    }

    /// Get mutable reference to session headers
    #[allow(dead_code)]
    pub fn session_headers_mut(&mut self) -> &mut HashMap<String, String> {
        &mut self.session_headers
    }

    /// Add or update a session header
    #[allow(dead_code)]
    pub fn set_session_header(&mut self, key: String, value: String) {
        self.session_headers.insert(key, value);
    }

    /// Remove a session header
    #[allow(dead_code)]
    pub fn remove_session_header(&mut self, key: &str) -> Option<String> {
        self.session_headers.remove(key)
    }

    /// Clear all session headers
    #[allow(dead_code)]
    pub fn clear_session_headers(&mut self) {
        self.session_headers.clear();
    }
}

impl Default for HttpContext {
    fn default() -> Self {
        Self::new()
    }
}
