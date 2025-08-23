//! # HTTP Request Commands
//!
//! Commands for executing HTTP requests and related operations

use crate::repl::events::{EditorMode, Pane};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use super::{
    Command, CommandContext, CommandEvent, HttpClientAccess, HttpCommand, HttpCommandContext,
};

/// Execute HTTP request (Enter in normal mode)
pub struct ExecuteRequestCommand;

impl HttpCommand for ExecuteRequestCommand {
    fn is_relevant(&self, context: &HttpCommandContext, event: &KeyEvent) -> bool {
        let is_enter = matches!(event.code, KeyCode::Enter);
        let is_normal_mode = context.state().current_mode == EditorMode::Normal;
        let no_modifiers = event.modifiers.is_empty();
        let is_request_pane = context.state().current_pane == Pane::Request;

        let is_relevant = is_enter && is_normal_mode && no_modifiers && is_request_pane;

        tracing::debug!(
            "ExecuteRequestCommand(Http).is_relevant(): enter={}, normal_mode={}, no_modifiers={}, request_pane={}, result={}",
            is_enter, is_normal_mode, no_modifiers, is_request_pane, is_relevant
        );

        if is_enter && !is_relevant {
            tracing::info!(
                "ExecuteRequestCommand(Http) rejected Enter: mode={:?}, modifiers={:?}, pane={:?}",
                context.state().current_mode,
                event.modifiers,
                context.state().current_pane
            );
        }

        is_relevant
    }

    fn execute(&self, _event: KeyEvent, context: &HttpCommandContext) -> Result<Vec<CommandEvent>> {
        let request_text = &context.state().request_text;

        // Simple parsing for tests - real implementation is in HttpExecuteCommand
        let lines: Vec<&str> = request_text.lines().collect();
        if lines.is_empty() || lines[0].trim().is_empty() {
            return Ok(vec![CommandEvent::NoAction]);
        }

        let parts: Vec<&str> = lines[0].split_whitespace().collect();
        if parts.len() < 2 {
            return Ok(vec![CommandEvent::NoAction]);
        }

        let method = parts[0].to_uppercase();
        let url = parts[1].to_string();

        // Check if HTTP client is available
        if context.http_client().is_some() {
            // Create HTTP request event for tests
            let event = CommandEvent::http_request_with_headers(
                method,
                url,
                Vec::new(), // Empty headers
                None,
            );
            Ok(vec![event])
        } else {
            // No HTTP client available
            Ok(vec![CommandEvent::NoAction])
        }
    }

    fn name(&self) -> &'static str {
        "ExecuteRequest"
    }
}

impl Command for ExecuteRequestCommand {
    fn is_relevant(&self, context: &CommandContext, event: &KeyEvent) -> bool {
        let is_enter = matches!(event.code, KeyCode::Enter);
        let is_normal_mode = context.state.current_mode == EditorMode::Normal;
        let no_modifiers = event.modifiers.is_empty();
        let is_request_pane = context.state.current_pane == Pane::Request;

        let is_relevant = is_enter && is_normal_mode && no_modifiers && is_request_pane;

        tracing::debug!(
            "ExecuteRequestCommand.is_relevant(): enter={}, normal_mode={}, no_modifiers={}, request_pane={}, result={}",
            is_enter, is_normal_mode, no_modifiers, is_request_pane, is_relevant
        );

        if is_enter && !is_relevant {
            tracing::info!(
                "ExecuteRequestCommand rejected Enter: mode={:?}, modifiers={:?}, pane={:?}",
                context.state.current_mode,
                event.modifiers,
                context.state.current_pane
            );
        }

        is_relevant
    }

    fn execute(&self, _event: KeyEvent, context: &CommandContext) -> Result<Vec<CommandEvent>> {
        let request_text = &context.state.request_text;

        // Simple parsing for tests - real implementation is in HttpExecuteCommand
        let lines: Vec<&str> = request_text.lines().collect();
        if lines.is_empty() || lines[0].trim().is_empty() {
            return Ok(vec![CommandEvent::NoAction]);
        }

        let parts: Vec<&str> = lines[0].split_whitespace().collect();
        if parts.len() < 2 {
            return Ok(vec![CommandEvent::NoAction]);
        }

        let method = parts[0].to_uppercase();
        let url = parts[1].to_string();

        // Create HTTP request event for tests
        let event = CommandEvent::http_request_with_headers(
            method,
            url,
            Vec::new(), // Empty headers
            None,
        );
        Ok(vec![event])
    }

    fn name(&self) -> &'static str {
        "ExecuteRequest"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::commands::ViewModelSnapshot;
    use crate::repl::events::{EditorMode, LogicalPosition, Pane};
    use crossterm::event::KeyModifiers;

    fn create_test_key_event(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::empty())
    }

    fn create_test_context() -> CommandContext {
        CommandContext {
            state: ViewModelSnapshot {
                current_mode: EditorMode::Normal,
                current_pane: Pane::Request,
                cursor_position: LogicalPosition { line: 0, column: 0 },
                request_text: String::new(),
                response_text: String::new(),
                terminal_dimensions: (80, 24),
                expand_tab: false,
                tab_width: 4,
            },
        }
    }

    #[test]
    fn execute_request_should_be_relevant_for_enter_in_normal_mode() {
        let context = create_test_context(); // Normal mode, Request pane
        let cmd = ExecuteRequestCommand;
        let event = create_test_key_event(KeyCode::Enter);

        assert!(Command::is_relevant(&cmd, &context, &event));
    }

    #[test]
    fn execute_request_should_not_be_relevant_in_insert_mode() {
        let mut context = create_test_context();
        context.state.current_mode = EditorMode::Insert;
        let cmd = ExecuteRequestCommand;
        let event = create_test_key_event(KeyCode::Enter);

        assert!(!Command::is_relevant(&cmd, &context, &event));
    }

    #[test]
    fn execute_request_should_not_be_relevant_in_visual_mode() {
        let mut context = create_test_context();
        context.state.current_mode = EditorMode::Visual;
        let cmd = ExecuteRequestCommand;
        let event = create_test_key_event(KeyCode::Enter);

        assert!(!Command::is_relevant(&cmd, &context, &event));
    }

    #[test]
    fn execute_request_should_not_be_relevant_in_command_mode() {
        let mut context = create_test_context();
        context.state.current_mode = EditorMode::Command;
        let cmd = ExecuteRequestCommand;
        let event = create_test_key_event(KeyCode::Enter);

        assert!(!Command::is_relevant(&cmd, &context, &event));
    }

    #[test]
    fn execute_request_should_not_be_relevant_in_response_pane() {
        let mut context = create_test_context();
        context.state.current_pane = Pane::Response;
        let cmd = ExecuteRequestCommand;
        let event = create_test_key_event(KeyCode::Enter);

        // Should NOT be relevant from Response pane
        assert!(!Command::is_relevant(&cmd, &context, &event));
    }

    #[test]
    fn execute_request_should_not_be_relevant_for_other_keys() {
        let context = create_test_context();
        let cmd = ExecuteRequestCommand;
        let event = create_test_key_event(KeyCode::Char('a'));

        assert!(!Command::is_relevant(&cmd, &context, &event));
    }

    #[test]
    fn execute_request_should_not_be_relevant_with_modifiers() {
        let context = create_test_context();
        let cmd = ExecuteRequestCommand;
        let event = KeyEvent::new(KeyCode::Enter, KeyModifiers::SHIFT);

        assert!(!Command::is_relevant(&cmd, &context, &event));
    }

    #[test]
    fn execute_request_should_produce_http_request_event() {
        let mut context = create_test_context();
        context.state.request_text = "GET https://httpbin.org/get".to_string();
        let cmd = ExecuteRequestCommand;
        let event = create_test_key_event(KeyCode::Enter);

        let result = Command::execute(&cmd, event, &context).unwrap();
        assert_eq!(result.len(), 1);
        assert!(matches!(
            result[0],
            CommandEvent::HttpRequestRequested { .. }
        ));
    }

    #[test]
    fn execute_request_should_return_correct_command_name() {
        let cmd = ExecuteRequestCommand;
        assert_eq!(Command::name(&cmd), "ExecuteRequest");
    }

    #[test]
    fn execute_request_should_not_be_relevant_for_ctrl_enter() {
        let context = create_test_context();
        let cmd = ExecuteRequestCommand;
        let event = KeyEvent::new(KeyCode::Enter, KeyModifiers::CONTROL);

        assert!(!Command::is_relevant(&cmd, &context, &event));
    }

    #[test]
    fn execute_request_should_not_be_relevant_for_shift_enter() {
        let context = create_test_context();
        let cmd = ExecuteRequestCommand;
        let event = KeyEvent::new(KeyCode::Enter, KeyModifiers::SHIFT);

        assert!(!Command::is_relevant(&cmd, &context, &event));
    }

    #[test]
    fn execute_request_should_not_be_relevant_for_alt_enter() {
        let context = create_test_context();
        let cmd = ExecuteRequestCommand;
        let event = KeyEvent::new(KeyCode::Enter, KeyModifiers::ALT);

        assert!(!Command::is_relevant(&cmd, &context, &event));
    }

    #[test]
    fn execute_request_should_not_be_relevant_for_combined_modifiers() {
        let context = create_test_context();
        let cmd = ExecuteRequestCommand;
        let event = KeyEvent::new(KeyCode::Enter, KeyModifiers::CONTROL | KeyModifiers::SHIFT);

        assert!(!Command::is_relevant(&cmd, &context, &event));
    }
}
