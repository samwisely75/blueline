//! # HTTP Request Commands
//!
//! Commands for executing HTTP requests and related operations

use crate::repl::events::EditorMode;
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
        matches!(event.code, KeyCode::Enter)
            && context.state().current_mode == EditorMode::Normal
            // Allow execution from both Request and Response panes - user should be able to execute from either
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
        matches!(event.code, KeyCode::Enter)
            && context.state.current_mode == EditorMode::Normal
            // Allow execution from both Request and Response panes - user should be able to execute from either
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

// TODO: Update tests for new event-driven API
/*
#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::events::EditorMode;
    use crossterm::event::KeyModifiers;

    fn create_test_key_event(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::empty())
    }

    #[test]
    fn execute_request_should_be_relevant_for_enter_in_normal_mode() {
        let vm = ViewModel::new(); // Starts in Normal mode, Request pane
        let cmd = ExecuteRequestCommand;
        let event = create_test_key_event(KeyCode::Enter);

        assert!(cmd.is_relevant(&vm, &event));
    }

    #[test]
    fn execute_request_should_not_be_relevant_in_insert_mode() {
        let mut vm = ViewModel::new();
        vm.change_mode(EditorMode::Insert).unwrap();
        let cmd = ExecuteRequestCommand;
        let event = create_test_key_event(KeyCode::Enter);

        assert!(!cmd.is_relevant(&vm, &event));
    }

    #[test]
    fn execute_request_should_not_be_relevant_in_response_pane() {
        let mut vm = ViewModel::new();
        vm.switch_pane(Pane::Response).unwrap();
        let cmd = ExecuteRequestCommand;
        let event = create_test_key_event(KeyCode::Enter);

        assert!(!cmd.is_relevant(&vm, &event));
    }
}
}
*/
