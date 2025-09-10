//! # HTTP Management
//!
//! Handles HTTP response management and session state.

// Pane import removed - using semantic operations instead
use super::AppState;
use std::collections::HashMap;

impl AppState {
    /// Get current request execution status
    pub fn is_executing_request(&self) -> bool {
        self.status_line.is_executing()
    }

    /// Get session headers
    pub fn session_headers(&self) -> &HashMap<String, String> {
        // Delegate to HTTP context for better domain separation
        self.http_context().session_headers()
    }

    /// Get request text from buffer
    pub fn get_request_text(&self) -> String {
        self.pane_manager.get_request_text()
    }

    /// Set response from HTTP response
    /// Note: JSON formatting is now handled at the view model layer
    pub fn set_response_from_http(
        &mut self,
        response: &bluenote::HttpResponse,
        formatted_body: String,
    ) {
        let status_code = response.status().as_u16();
        let status_message = response
            .status()
            .canonical_reason()
            .unwrap_or("")
            .to_string();
        let duration_ms = response.duration_ms();

        self.response.set_status_code(status_code);
        self.response.set_status_message(status_message.clone());
        self.response.set_duration_ms(duration_ms);
        self.response.set_body(formatted_body.clone());

        // Update status line with HTTP status
        self.status_line
            .set_http_status(status_code, status_message, duration_ms);

        // Update response buffer content using semantic operation
        self.pane_manager.set_response_content(&formatted_body);

        // Response content setting already resets cursor and scroll positions

        // Recalculate pane dimensions now that we have a response
        let (width, height) = self.pane_manager.terminal_dimensions;
        self.pane_manager.update_terminal_size(width, height, true);

        tracing::debug!("Pane dimensions updated after HTTP response");

        // Full redraw is needed when response first appears to draw the response pane
        // This will also update the status bar with TAT and message
        tracing::debug!(
            "Response set from HTTP response: status={}, duration={}ms",
            status_code,
            duration_ms
        );
    }

    /// Set response with status code and content
    pub fn set_response(&mut self, status_code: u16, content: String) {
        self.response.set_status_code(status_code);
        self.response.set_body(content.clone());

        // Update response buffer using semantic operation
        self.pane_manager.set_response_content(&content);

        // Recalculate pane dimensions now that we have a response
        let (width, height) = self.pane_manager.terminal_dimensions;
        self.pane_manager.update_terminal_size(width, height, true);

        tracing::debug!("Pane dimensions updated after manual response");

        // Full redraw is needed when response first appears
        tracing::debug!(
            "Response set: status={}, content_length={}",
            status_code,
            content.len()
        );
    }

    /// Get response status code
    pub fn get_response_status_code(&self) -> Option<u16> {
        self.response.status_code()
    }

    /// Get response status message
    pub fn get_response_status_message(&self) -> Option<String> {
        self.response.status_message().cloned()
    }

    /// Get response duration in milliseconds
    pub fn get_response_duration_ms(&self) -> Option<u64> {
        self.response.duration_ms()
    }

    /// Get response text content
    pub fn get_response_text(&self) -> String {
        self.pane_manager.get_response_text()
    }

    /// Check if the HTTP response content-type indicates JSON content
    ///
    /// Checks for both "application/json" and "text/json" content types as commonly
    /// used by REST APIs and web services.
    ///
    /// # Arguments
    ///
    /// * `response` - The HTTP response containing headers
    ///
    /// # Returns
    ///
    /// `true` if content-type indicates JSON, `false` otherwise
    pub fn is_json_content_type(&self, response: &bluenote::HttpResponse) -> bool {
        let headers = response.headers();
        
        if let Some(content_type) = headers.get("content-type") {
            if let Ok(content_type_str) = content_type.to_str() {
                let content_type_lower = content_type_str.to_lowercase();
                let is_json = content_type_lower.contains("application/json")
                    || content_type_lower.contains("text/json");
                tracing::debug!("Content-type check: '{}' -> is_json: {}", content_type_str, is_json);
                return is_json;
            } else {
                tracing::debug!("Content-type header exists but cannot be converted to string");
            }
        } else {
            tracing::debug!("No content-type header found");
        }
        false
    }
}
