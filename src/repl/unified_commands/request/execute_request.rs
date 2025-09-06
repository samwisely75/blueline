//! # ExecuteRequest Command
//!
//! Handles Enter key for executing HTTP requests in Normal mode in Request pane.
//! This is an ASYNC command that executes HTTP requests and updates the response pane.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::register_command;
use crate::repl::models::pane_state::EditorMode;
use crate::repl::unified_commands::{Command, CommandContext, ExecutionContext};
use crate::repl::view_models::post_command_actions::PostCommandAction;

/// Execute HTTP request command (Enter in Normal mode on Request pane)
///
/// This command:
/// 1. Parses the request from the request buffer
/// 2. Executes it through HttpService asynchronously
/// 3. Updates the response pane with results
/// 4. Sets appropriate status messages
#[derive(Debug, Default)]
pub struct ExecuteRequestCommand;

impl ExecuteRequestCommand {
    /// Create a new ExecuteRequestCommand
    pub fn new() -> Self {
        Self
    }
}

impl Command for ExecuteRequestCommand {
    fn is_relevant(&self, key_event: KeyEvent, mode: EditorMode, context: &CommandContext) -> bool {
        // Only relevant when:
        // - Enter key pressed without modifiers
        // - In Normal mode
        // - On Request pane (not read-only)
        let is_enter = matches!(key_event.code, KeyCode::Enter);
        let no_modifiers = key_event.modifiers == KeyModifiers::NONE;
        let is_normal_mode = mode == EditorMode::Normal;
        let is_request_pane = !context.is_read_only; // Request pane is editable

        let is_relevant = is_enter && no_modifiers && is_normal_mode && is_request_pane;

        tracing::debug!(
            "ExecuteRequestCommand.is_relevant(): enter={}, normal_mode={}, no_modifiers={}, request_pane={}, result={}",
            is_enter, is_normal_mode, no_modifiers, is_request_pane, is_relevant
        );

        if is_enter && !is_relevant {
            tracing::info!(
                "ExecuteRequestCommand rejected Enter: mode={:?}, modifiers={:?}, read_only={}",
                mode,
                key_event.modifiers,
                context.is_read_only
            );
        }

        is_relevant
    }

    fn execute(
        &self,
        _key_event: KeyEvent,
        context: &mut ExecutionContext,
    ) -> Result<Vec<PostCommandAction>> {
        // Get request text from the app state
        let request_text = context.app_state.get_request_text();

        // Check if we have any request content
        if request_text.trim().is_empty() {
            context
                .app_state
                .set_status_message("No request to execute");
            return Ok(vec![PostCommandAction::StatusBarUpdateRequired]);
        }

        // Check if HTTP service is available
        if context.services.http.is_none() {
            // Update status to show error
            context
                .app_state
                .set_status_message("HTTP service not available - configure a profile first");
            return Ok(vec![PostCommandAction::StatusBarUpdateRequired]);
        }

        // Basic validation: check if request looks valid (has method and URL)
        let lines: Vec<&str> = request_text.lines().collect();
        if lines.is_empty() || lines[0].trim().is_empty() {
            context
                .app_state
                .set_status_message("Invalid request: empty content");
            return Ok(vec![PostCommandAction::StatusBarUpdateRequired]);
        }

        let parts: Vec<&str> = lines[0].split_whitespace().collect();
        if parts.len() < 2 {
            context
                .app_state
                .set_status_message("Invalid request format. Use: METHOD URL");
            return Ok(vec![PostCommandAction::StatusBarUpdateRequired]);
        }

        // Mark request as executing (for UI feedback)
        context.app_state.set_executing_request(true);

        // Execute the HTTP request asynchronously
        let http_service = context.services.http.as_mut().unwrap();
        http_service.execute_async(request_text.to_string());

        // Update status to show request is executing
        context
            .app_state
            .set_status_message("Executing HTTP request...");

        // Return post-command actions for UI updates
        Ok(vec![
            PostCommandAction::StatusBarUpdateRequired,
            // The response will be handled asynchronously via the HTTP service response channel
            // and processed by the main event loop, which will trigger ResponseContentChanged
        ])
    }

    fn name(&self) -> &'static str {
        "ExecuteRequest"
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

    fn create_test_context() -> CommandContext {
        CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false, // Request pane is editable
            has_selection: false,
            ex_command_buffer: String::new(),
        }
    }

    #[test]
    fn execute_request_should_be_relevant_for_enter_in_normal_mode_on_request_pane() {
        let context = create_test_context();
        let cmd = ExecuteRequestCommand::new();
        let event = create_test_key_event(KeyCode::Enter);

        assert!(cmd.is_relevant(event, EditorMode::Normal, &context));
    }

    #[test]
    fn execute_request_should_not_be_relevant_in_insert_mode() {
        let context = create_test_context();
        let cmd = ExecuteRequestCommand::new();
        let event = create_test_key_event(KeyCode::Enter);

        assert!(!cmd.is_relevant(event, EditorMode::Insert, &context));
    }

    #[test]
    fn execute_request_should_not_be_relevant_in_visual_mode() {
        let context = create_test_context();
        let cmd = ExecuteRequestCommand::new();
        let event = create_test_key_event(KeyCode::Enter);

        assert!(!cmd.is_relevant(event, EditorMode::Visual, &context));
    }

    #[test]
    fn execute_request_should_not_be_relevant_in_command_mode() {
        let context = create_test_context();
        let cmd = ExecuteRequestCommand::new();
        let event = create_test_key_event(KeyCode::Enter);

        assert!(!cmd.is_relevant(event, EditorMode::Command, &context));
    }

    #[test]
    fn execute_request_should_not_be_relevant_on_response_pane() {
        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Response,
            is_read_only: true, // Response pane is read-only
            has_selection: false,
            ex_command_buffer: String::new(),
        };
        let cmd = ExecuteRequestCommand::new();
        let event = create_test_key_event(KeyCode::Enter);

        assert!(!cmd.is_relevant(event, EditorMode::Normal, &context));
    }

    #[test]
    fn execute_request_should_not_be_relevant_for_other_keys() {
        let context = create_test_context();
        let cmd = ExecuteRequestCommand::new();
        let event = create_test_key_event(KeyCode::Char('a'));

        assert!(!cmd.is_relevant(event, EditorMode::Normal, &context));
    }

    #[test]
    fn execute_request_should_not_be_relevant_with_modifiers() {
        let context = create_test_context();
        let cmd = ExecuteRequestCommand::new();
        let event = KeyEvent::new(KeyCode::Enter, KeyModifiers::SHIFT);

        assert!(!cmd.is_relevant(event, EditorMode::Normal, &context));
    }

    #[test]
    fn execute_request_should_handle_empty_request() {
        let mut app_state = AppState::new();
        // Empty request text - should result in error message

        let mut services = Services::new();
        // No HTTP service configured

        let mut context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        let cmd = ExecuteRequestCommand::new();
        let result = cmd.execute(
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
            &mut context,
        );

        assert!(result.is_ok());
        let actions = result.unwrap();
        assert_eq!(actions.len(), 1);
        assert!(matches!(
            actions[0],
            PostCommandAction::StatusBarUpdateRequired
        ));

        // Should have set an error message
        assert!(context
            .app_state
            .get_status_message()
            .expect("Status message should be set")
            .contains("No request to execute"));
    }

    #[test]
    fn execute_request_should_handle_no_http_service() {
        let mut app_state = AppState::new();
        app_state
            .pane_manager
            .set_request_content("GET https://httpbin.org/get");

        let mut services = Services::new();
        // No HTTP service configured

        let mut context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        let cmd = ExecuteRequestCommand::new();
        let result = cmd.execute(
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
            &mut context,
        );

        assert!(result.is_ok());
        let actions = result.unwrap();
        assert_eq!(actions.len(), 1);
        assert!(matches!(
            actions[0],
            PostCommandAction::StatusBarUpdateRequired
        ));

        // Should have set an error message about HTTP service
        assert!(context
            .app_state
            .get_status_message()
            .expect("Status message should be set")
            .contains("HTTP service not available"));
    }

    #[test]
    fn execute_request_should_handle_invalid_request_format() {
        let mut app_state = AppState::new();
        app_state.pane_manager.set_request_content("GET"); // Missing URL

        let mut services = Services::new();

        let mut context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        let cmd = ExecuteRequestCommand::new();
        let result = cmd.execute(
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
            &mut context,
        );

        assert!(result.is_ok());
        let actions = result.unwrap();
        assert_eq!(actions.len(), 1);
        assert!(matches!(
            actions[0],
            PostCommandAction::StatusBarUpdateRequired
        ));

        // Should have set an error message
        // Since there's no HTTP service configured, it should show that error first
        let status_message = context
            .app_state
            .get_status_message()
            .expect("Status message should be set");
        assert!(status_message.contains("HTTP service not available"));
    }

    #[test]
    fn execute_request_should_return_correct_command_name() {
        let cmd = ExecuteRequestCommand::new();
        assert_eq!(cmd.name(), "ExecuteRequest");
    }

    #[test]
    fn execute_request_should_not_be_relevant_for_ctrl_enter() {
        let context = create_test_context();
        let cmd = ExecuteRequestCommand::new();
        let event = KeyEvent::new(KeyCode::Enter, KeyModifiers::CONTROL);

        assert!(!cmd.is_relevant(event, EditorMode::Normal, &context));
    }

    #[test]
    fn execute_request_should_not_be_relevant_for_shift_enter() {
        let context = create_test_context();
        let cmd = ExecuteRequestCommand::new();
        let event = KeyEvent::new(KeyCode::Enter, KeyModifiers::SHIFT);

        assert!(!cmd.is_relevant(event, EditorMode::Normal, &context));
    }

    #[test]
    fn execute_request_should_not_be_relevant_for_alt_enter() {
        let context = create_test_context();
        let cmd = ExecuteRequestCommand::new();
        let event = KeyEvent::new(KeyCode::Enter, KeyModifiers::ALT);

        assert!(!cmd.is_relevant(event, EditorMode::Normal, &context));
    }

    #[test]
    fn execute_request_should_not_be_relevant_for_combined_modifiers() {
        let context = create_test_context();
        let cmd = ExecuteRequestCommand::new();
        let event = KeyEvent::new(KeyCode::Enter, KeyModifiers::CONTROL | KeyModifiers::SHIFT);

        assert!(!cmd.is_relevant(event, EditorMode::Normal, &context));
    }
}

// Auto-register this command using the dynamic discovery system
register_command!(ExecuteRequestCommand, "ExecuteRequest");
