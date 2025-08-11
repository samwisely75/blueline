//! # HTTP Request Commands
//!
//! Commands for executing HTTP requests and related operations

use crate::repl::events::{EditorMode, Pane};
use crate::repl::utils::parse_request_from_text;
use anyhow::Result;
use bluenote::HttpRequestArgs;
use crossterm::event::{KeyCode, KeyEvent};
use std::collections::HashMap;

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
        let session_headers = HashMap::new(); // Could be passed via context in the future

        // Parse the request from the buffer
        match parse_request_from_text(request_text, &session_headers) {
            Ok((request_args, _url_str)) => {
                // Check if HTTP client is available
                if context.http_client().is_some() {
                    // Create HTTP request event
                    let event = CommandEvent::http_request_with_headers(
                        request_args.method().unwrap_or(&"GET".to_string()).clone(),
                        request_args
                            .url_path()
                            .map(|p| p.to_string())
                            .unwrap_or_else(|| "".to_string()),
                        request_args
                            .headers()
                            .iter()
                            .map(|(k, v)| (k.clone(), v.clone()))
                            .collect(),
                        request_args.body().cloned(),
                    );
                    Ok(vec![event])
                } else {
                    // No HTTP client available - could be an error event in future
                    Ok(vec![CommandEvent::NoAction])
                }
            }
            Err(error_msg) => {
                // Could create an error event type in the future
                tracing::warn!("Failed to parse HTTP request: {}", error_msg);
                Ok(vec![CommandEvent::NoAction])
            }
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
        let session_headers = HashMap::new(); // Could be passed via context in the future

        // Parse the request from the buffer
        match parse_request_from_text(request_text, &session_headers) {
            Ok((request_args, _url_str)) => {
                // Create HTTP request event - note we don't have HTTP client in basic context
                // Controller will need to handle this with HTTP client
                let event = CommandEvent::http_request_with_headers(
                    request_args.method().unwrap_or(&"GET".to_string()).clone(),
                    request_args
                        .url_path()
                        .map(|p| p.to_string())
                        .unwrap_or_else(|| "".to_string()),
                    request_args
                        .headers()
                        .iter()
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect(),
                    request_args.body().cloned(),
                );
                Ok(vec![event])
            }
            Err(error_msg) => {
                // Could create an error event type in the future
                tracing::warn!("Failed to parse HTTP request: {}", error_msg);
                Ok(vec![CommandEvent::NoAction])
            }
        }
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
                verbose: false,
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
