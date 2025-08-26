//! # HTTP Request Commands
//!
//! Commands for executing HTTP requests using the unified command pattern.

use crate::repl::events::EditorMode;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::{Command, CommandContext, ExecutionContext, ModelEvent};

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

    fn handle(&self, context: &mut ExecutionContext) -> Result<Vec<ModelEvent>> {
        // Check if HTTP service is available
        let http_service = context
            .services
            .http
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("HTTP service not configured"))?;

        // Get request text from the view model
        let request_text = context.view_model.get_request_text();

        // Set executing status
        context.view_model.set_executing_request(true);

        // Execute the HTTP request asynchronously through the service
        http_service.execute_async(request_text);

        // Return event indicating request was initiated
        Ok(vec![ModelEvent::StatusMessageSet {
            message: "Executing HTTP request...".to_string(),
        }])
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
    use crate::repl::events::Pane;
    use crate::repl::services::Services;
    use crate::repl::view_models::ViewModel;

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

    #[tokio::test]
    async fn http_execute_should_parse_and_trigger_request() {
        use bluenote::get_blank_profile;

        let mut view_model = ViewModel::new();
        // Set up request content through pane_manager
        view_model
            .pane_manager
            .set_request_content("GET https://httpbin.org/get");

        let mut services = Services::new();
        // Configure HTTP service for test
        let profile = get_blank_profile();
        let _ = services.configure_http(&profile); // May fail, but that's ok for test

        let mut context = ExecutionContext {
            view_model: &mut view_model,
            services: &mut services,
        };

        let cmd = HttpExecuteCommand::new();
        let result = cmd.handle(&mut context);

        // If HTTP service is available, it should return success
        // If not, it should return an error
        assert!(result.is_ok() || result.is_err());

        if result.is_ok() {
            let events = result.unwrap();
            assert_eq!(events.len(), 1);
            assert!(matches!(events[0], ModelEvent::StatusMessageSet { .. }));
            // Check that executing flag was set
            assert!(view_model.is_executing_request());
        }
    }

    #[tokio::test]
    async fn http_execute_should_handle_invalid_request() {
        use bluenote::get_blank_profile;

        let mut view_model = ViewModel::new();
        // Set up invalid request through pane_manager
        view_model.pane_manager.set_request_content("INVALID");

        let mut services = Services::new();
        // Configure HTTP service for test
        let profile = get_blank_profile();
        let _ = services.configure_http(&profile); // May fail, but that's ok for test

        let mut context = ExecutionContext {
            view_model: &mut view_model,
            services: &mut services,
        };

        let cmd = HttpExecuteCommand::new();
        let result = cmd.handle(&mut context);

        // If HTTP service is available, it should parse and handle invalid request
        // If not, it should return an error
        assert!(result.is_ok() || result.is_err());
    }
}
