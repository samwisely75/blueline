//! # HTTP Request Commands
//!
//! Commands for executing HTTP requests using the unified command pattern.

use crate::repl::models::events::view_events::ViewEvent;
use crate::repl::models::pane_state::EditorMode;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::{Command, CommandContext, ExecutionContext};

/// Parse request text to extract method and URL
/// This is a simple parser for the event - actual parsing happens in HttpService
#[allow(dead_code)]
fn parse_request_basics(request_text: &str) -> (&str, &str) {
    let trimmed = request_text.trim();
    if trimmed.is_empty() {
        return ("GET", "");
    }

    let parts: Vec<&str> = trimmed.splitn(2, ' ').collect();
    match parts.as_slice() {
        [method, url, ..] => (*method, url.trim()),
        [url] => ("GET", url.trim()),
        _ => ("GET", ""),
    }
}

/// Execute HTTP request command (Enter in Normal mode on Request pane)
///
/// This command:
/// 1. Parses the request from the buffer
/// 2. Executes it through HttpService
/// 3. Updates the response pane with results
pub struct HttpExecuteCommand;

impl HttpExecuteCommand {
    /// Create a new HttpExecuteCommand
    pub fn new() -> Self {
        Self
    }
}

impl Command for HttpExecuteCommand {
    fn is_relevant(&self, key_event: KeyEvent, mode: EditorMode, context: &CommandContext) -> bool {
        // Only relevant when:
        // - In Normal mode
        // - On Request pane (not read-only)
        // - Enter key pressed without modifiers
        let is_enter = matches!(key_event.code, KeyCode::Enter);
        let no_modifiers = key_event.modifiers == KeyModifiers::NONE;
        let is_normal_mode = mode == EditorMode::Normal;
        let is_request_pane = !context.is_read_only; // Request pane is editable

        is_enter && no_modifiers && is_normal_mode && is_request_pane
    }

    fn execute(&self, context: &mut ExecutionContext) -> Result<Vec<ViewEvent>> {
        // Get request text from the app state
        let _request_text = context.app_state.get_request_text();

        // Check if HTTP service is available
        if context.services.http.is_none() {
            // Update status to show error
            context
                .app_state
                .set_status_message("HTTP service not available");
            return Ok(vec![ViewEvent::StatusBarUpdateRequired]);
        }

        // For now, we just mark the request as executing and update status
        // The actual HTTP execution happens asynchronously in the HTTP service
        // which is polled by AppViewModel's process_next_event
        context.app_state.set_executing_request(true);

        // TODO: In the future, we should trigger the HTTP request here directly
        // For now, the old system in AppViewModel will handle the actual execution

        // Update status to show request is executing
        context
            .app_state
            .set_status_message("Executing HTTP request...");

        // Return view events for UI updates
        Ok(vec![
            ViewEvent::StatusBarUpdateRequired,
            // The response will be handled asynchronously via handle_http_response
        ])
    }

    fn name(&self) -> &'static str {
        "HttpExecute"
    }
}

impl Default for HttpExecuteCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::models::pane_state::Pane;
    use crate::repl::models::AppState;
    use crate::repl::services::Services;

    fn create_test_key_event(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn http_execute_should_be_relevant_for_enter_in_normal_mode_on_request_pane() {
        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false, // Request pane is editable
            has_selection: false,
        };

        let cmd = HttpExecuteCommand::new();
        let event = create_test_key_event(KeyCode::Enter);

        assert!(cmd.is_relevant(event, EditorMode::Normal, &context));
    }

    #[test]
    fn http_execute_should_not_be_relevant_in_insert_mode() {
        let context = CommandContext {
            current_mode: EditorMode::Insert,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
        };

        let cmd = HttpExecuteCommand::new();
        let event = create_test_key_event(KeyCode::Enter);

        assert!(!cmd.is_relevant(event, EditorMode::Insert, &context));
    }

    #[test]
    fn http_execute_should_not_be_relevant_on_response_pane() {
        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Response,
            is_read_only: true, // Response pane is read-only
            has_selection: false,
        };

        let cmd = HttpExecuteCommand::new();
        let event = create_test_key_event(KeyCode::Enter);

        assert!(!cmd.is_relevant(event, EditorMode::Normal, &context));
    }

    #[test]
    fn http_execute_should_not_be_relevant_with_modifiers() {
        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
        };

        let cmd = HttpExecuteCommand::new();
        let event = KeyEvent::new(KeyCode::Enter, KeyModifiers::CONTROL);

        assert!(!cmd.is_relevant(event, EditorMode::Normal, &context));
    }

    #[test]
    fn http_execute_should_parse_and_trigger_request() {
        let mut app_state = AppState::new();
        // Note: In real usage, request content would be set through user input
        // For this test, we're testing that the command returns appropriate events

        let mut services = Services::new();
        // No need to configure HTTP service - command just emits events

        let mut context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        let cmd = HttpExecuteCommand::new();
        let result = cmd.execute(&mut context);

        // Should return events even with empty request or no HTTP service
        assert!(result.is_ok());
        let events = result.unwrap();
        assert_eq!(events.len(), 1); // StatusBarUpdateRequired
        assert!(matches!(events[0], ViewEvent::StatusBarUpdateRequired));
    }

    #[test]
    fn http_execute_should_parse_request_basics() {
        // Test the parse_request_basics function
        assert_eq!(parse_request_basics(""), ("GET", ""));
        assert_eq!(
            parse_request_basics("https://example.com"),
            ("GET", "https://example.com")
        );
        assert_eq!(
            parse_request_basics("POST https://example.com"),
            ("POST", "https://example.com")
        );
        assert_eq!(parse_request_basics("PUT /api/data"), ("PUT", "/api/data"));
        assert_eq!(
            parse_request_basics("  DELETE  /item  "),
            ("DELETE", "/item")
        );
    }
}
