//! # HTTP Management
//!
//! Handles HTTP client configuration, request execution, and response management.

// Pane import removed - using semantic operations instead
use crate::repl::view_models::core::ViewModel;
use anyhow::Result;
use bluenote::{HttpClient, HttpConnectionProfile};
use std::collections::HashMap;

impl ViewModel {
    /// Set HTTP client from profile
    pub fn set_http_client(&mut self, profile: &impl HttpConnectionProfile) -> Result<()> {
        let client = HttpClient::new(profile)?;
        self.http_client = Some(client);
        tracing::debug!("HTTP client configured with profile");
        Ok(())
    }

    /// Get reference to HTTP client
    pub fn http_client(&self) -> Option<&HttpClient> {
        self.http_client.as_ref()
    }

    /// Set verbose mode
    pub fn set_verbose(&mut self, verbose: bool) {
        self.http_verbose = verbose;
        tracing::debug!("Verbose mode set to: {}", verbose);
    }

    /// Get current request execution status
    pub fn is_executing_request(&self) -> bool {
        self.status_line.is_executing()
    }

    /// Set request execution status and update status bar
    pub fn set_executing_request(&mut self, executing: bool) {
        self.status_line.set_executing(executing);
        if executing {
            tracing::debug!("Request execution started");
        } else {
            tracing::debug!("Request execution finished");
        }
        // Emit status bar update to reflect execution state
        self.emit_view_event([crate::repl::events::ViewEvent::StatusBarUpdateRequired]);
    }

    /// Get session headers
    pub fn session_headers(&self) -> &HashMap<String, String> {
        &self.http_session_headers
    }

    /// Get request text from buffer
    pub fn get_request_text(&self) -> String {
        self.pane_manager.get_request_text()
    }

    /// Set response from HTTP response
    pub fn set_response_from_http(&mut self, response: &bluenote::HttpResponse) {
        let status_code = response.status().as_u16();
        let status_message = response
            .status()
            .canonical_reason()
            .unwrap_or("")
            .to_string();
        let duration_ms = response.duration_ms();
        let body = response.body().to_string();

        self.response.set_status_code(status_code);
        self.response.set_status_message(status_message.clone());
        self.response.set_duration_ms(duration_ms);
        self.response.set_body(body.clone());

        // Update status line with HTTP status
        self.status_line
            .set_http_status(status_code, status_message, duration_ms);

        // Update response buffer content using semantic operation
        let _events = self.pane_manager.set_response_content(&body);

        // Response content setting already resets cursor and scroll positions

        // Recalculate pane dimensions now that we have a response
        let (width, height) = self.pane_manager.terminal_dimensions;
        self.pane_manager.update_terminal_size(width, height, true);

        tracing::debug!("Pane dimensions updated after HTTP response");

        // Full redraw is needed when response first appears to draw the response pane
        // This will also update the status bar with TAT and message
        self.emit_view_event([crate::repl::events::ViewEvent::FullRedrawRequired]);

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
        let _events = self.pane_manager.set_response_content(&content);

        // Recalculate pane dimensions now that we have a response
        let (width, height) = self.pane_manager.terminal_dimensions;
        self.pane_manager.update_terminal_size(width, height, true);

        tracing::debug!("Pane dimensions updated after manual response");

        // Full redraw is needed when response first appears
        self.emit_view_event([crate::repl::events::ViewEvent::FullRedrawRequired]);

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

    /// Check if verbose mode is enabled
    pub fn is_verbose(&self) -> bool {
        self.http_verbose
    }
}
