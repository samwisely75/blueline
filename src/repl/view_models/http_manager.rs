//! # HTTP Management
//!
//! Handles HTTP client configuration, request execution, and response management.

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
        self.verbose = verbose;
        tracing::debug!("Verbose mode set to: {}", verbose);
    }

    /// Get session headers
    pub fn session_headers(&self) -> &HashMap<String, String> {
        &self.session_headers
    }

    /// Get request text from buffer
    pub fn get_request_text(&self) -> String {
        self.request_buffer.content().lines().join("\n")
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
        self.response.set_status_message(status_message);
        self.response.set_duration_ms(duration_ms);
        self.response.set_body(body.clone());

        // Update response buffer content
        self.response_buffer.content_mut().set_text(&body);
        self.response_buffer
            .set_cursor(crate::repl::events::LogicalPosition::zero());

        // Rebuild response display cache
        let content_width = self.get_content_width();
        let response_lines = self.response_buffer.content().lines().to_vec();
        if let Ok(cache) = crate::repl::models::build_display_cache(
            &response_lines,
            content_width,
            self.wrap_enabled,
        ) {
            self.response_display_cache = cache;
        }

        // Reset response display cursor and scroll
        self.response_display_cursor = (0, 0);
        self.response_scroll_offset = (0, 0);

        // Recalculate request pane height now that we have a response
        self.request_pane_height = self.terminal_height / 2;

        // Full redraw is needed when response first appears to draw the response pane
        // This will also update the status bar with TAT and message
        self.emit_view_event(crate::repl::events::ViewEvent::FullRedrawRequired);

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

        // Update response buffer
        self.response_buffer.content_mut().set_text(&content);
        self.response_buffer
            .set_cursor(crate::repl::events::LogicalPosition::zero());

        // Rebuild response display cache
        let content_width = self.get_content_width();
        let response_lines = self.response_buffer.content().lines().to_vec();
        if let Ok(cache) = crate::repl::models::build_display_cache(
            &response_lines,
            content_width,
            self.wrap_enabled,
        ) {
            self.response_display_cache = cache;
        }

        // Reset response display cursor and scroll
        self.response_display_cursor = (0, 0);
        self.response_scroll_offset = (0, 0);

        // Recalculate request pane height now that we have a response
        self.request_pane_height = self.terminal_height / 2;

        // Full redraw is needed when response first appears
        self.emit_view_event(crate::repl::events::ViewEvent::FullRedrawRequired);

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
        self.response_buffer.content().lines().join("\n")
    }

    /// Check if verbose mode is enabled
    pub fn is_verbose(&self) -> bool {
        self.verbose
    }
}
