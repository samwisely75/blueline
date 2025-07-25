//! # Command Line Commands  
//!
//! This module contains commands for handling colon-prefixed vi commands
//! like :q, :w, :x, and HTTP-specific commands for request execution.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::repl::{
    command::{Command, CommandResult},
    model::{AppState, EditorMode, Pane, ResponseBuffer},
};

/// Command for typing characters in command mode
pub struct CommandModeInputCommand;

impl CommandModeInputCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Command for CommandModeInputCommand {
    fn is_relevant(&self, state: &AppState, _event: &KeyEvent) -> bool {
        // Only relevant in Command mode
        matches!(state.mode, EditorMode::Command)
    }

    fn process(&self, event: KeyEvent, state: &mut AppState) -> Result<bool> {
        match event.code {
            KeyCode::Char(ch) if event.modifiers == KeyModifiers::NONE => {
                // Add character to command buffer
                state.command_buffer.push(ch);
                state.status_message = format!(":{}", state.command_buffer);
                Ok(true)
            }
            KeyCode::Backspace => {
                // Remove last character from command buffer
                if !state.command_buffer.is_empty() {
                    state.command_buffer.pop();
                    state.status_message = format!(":{}", state.command_buffer);
                } else {
                    // If buffer is empty, exit command mode
                    state.mode = EditorMode::Normal;
                    state.status_message = "".to_string();
                }
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn name(&self) -> &'static str {
        "CommandModeInput"
    }
}

/// Command for executing commands in command mode (Enter)
pub struct ExecuteCommandCommand;

impl ExecuteCommandCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Command for ExecuteCommandCommand {
    fn is_relevant(&self, state: &AppState, _event: &KeyEvent) -> bool {
        // Only relevant in Command mode
        matches!(state.mode, EditorMode::Command)
    }

    fn process(&self, event: KeyEvent, state: &mut AppState) -> Result<bool> {
        if matches!(event.code, KeyCode::Enter) {
            let command = state.command_buffer.trim();

            // Handle basic vim commands
            match command {
                "q" | "quit" => {
                    match state.current_pane {
                        Pane::Request => {
                            // In Request pane, quit the application
                            state.should_quit = true;
                            state.status_message = "Goodbye!".to_string();
                        }
                        Pane::Response => {
                            // In Response pane, close the response and maximize request
                            state.response_buffer = None;
                            state.current_pane = Pane::Request;
                            state.status_message = "Response pane closed".to_string();
                        }
                    }
                }
                "q!" | "quit!" => {
                    // Force quit the application regardless of pane
                    state.should_quit = true;
                    state.status_message = "Goodbye!".to_string();
                }
                "w" | "write" => {
                    // TODO: Save functionality
                    state.status_message = "Save not implemented yet".to_string();
                }
                "wq" => {
                    // TODO: Save and quit
                    state.status_message = "Save and quit not implemented yet".to_string();
                }
                "x" | "execute" => {
                    // Request execution - set a flag to trigger async execution
                    state.execute_request_flag = true;
                    state.status_message = "Executing request...".to_string();
                }
                "" => {
                    // Empty command, just exit command mode
                    state.status_message = "".to_string();
                }
                _ => {
                    state.status_message = format!("Unknown command: {}", command);
                }
            }

            // Exit command mode
            state.mode = EditorMode::Normal;
            state.command_buffer.clear();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn name(&self) -> &'static str {
        "ExecuteCommand"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::model::{AppState, ResponseBuffer};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    /// Create a test AppState for command testing
    fn create_test_app_state() -> AppState {
        AppState::new((80, 24), false)
    }

    #[test]
    fn command_mode_input_command_should_be_relevant_in_command_mode() {
        let command = CommandModeInputCommand::new();
        let mut state = create_test_app_state();
        state.mode = EditorMode::Command;
        let event = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);

        assert!(command.is_relevant(&state, &event));
        assert_eq!(command.name(), "CommandModeInput");
    }

    #[test]
    fn command_mode_input_should_add_characters_to_buffer() {
        let command = CommandModeInputCommand::new();
        let mut state = create_test_app_state();
        state.mode = EditorMode::Command;
        let event = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);

        let result = command.process(event, &mut state).unwrap();

        assert!(result);
        assert_eq!(state.command_buffer, "q");
        assert_eq!(state.status_message, ":q");
    }

    #[test]
    fn command_mode_input_should_handle_backspace() {
        let command = CommandModeInputCommand::new();
        let mut state = create_test_app_state();
        state.mode = EditorMode::Command;
        state.command_buffer = "qu".to_string();
        state.status_message = ":qu".to_string();
        let event = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);

        let result = command.process(event, &mut state).unwrap();

        assert!(result);
        assert_eq!(state.command_buffer, "q");
        assert_eq!(state.status_message, ":q");
    }

    #[test]
    fn command_mode_input_should_exit_command_mode_when_buffer_empty_and_backspace() {
        let command = CommandModeInputCommand::new();
        let mut state = create_test_app_state();
        state.mode = EditorMode::Command;
        state.command_buffer = "".to_string();
        let event = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);

        let result = command.process(event, &mut state).unwrap();

        assert!(result);
        assert_eq!(state.mode, EditorMode::Normal);
        assert_eq!(state.status_message, "");
    }

    #[test]
    fn execute_q_command_should_quit_from_request_pane() {
        // Create a test state in Request pane
        let mut state = create_test_app_state();
        state.mode = EditorMode::Command;
        state.current_pane = Pane::Request;
        state.command_buffer = "q".to_string();

        let command = ExecuteCommandCommand::new();
        let event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);

        // Execute the command
        let result = command.process(event, &mut state).unwrap();

        // Should handle the event and set quit flag
        assert!(result);
        assert!(state.should_quit);
        assert_eq!(state.mode, EditorMode::Normal);
        assert!(state.command_buffer.is_empty());
        assert_eq!(state.status_message, "Goodbye!");
    }

    #[test]
    fn execute_q_command_should_close_response_pane_from_response_pane() {
        // Create a test state with response buffer in Response pane
        let mut state = create_test_app_state();
        state.mode = EditorMode::Command;
        state.current_pane = Pane::Response;
        state.command_buffer = "q".to_string();

        // Add a response buffer
        let response_content = "HTTP/1.1 200 OK\nContent-Type: application/json".to_string();
        let response_buffer = ResponseBuffer::new(response_content);
        state.response_buffer = Some(response_buffer);

        let command = ExecuteCommandCommand::new();
        let event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);

        // Execute the command
        let result = command.process(event, &mut state).unwrap();

        // Should handle the event, close response pane, and switch to Request pane
        assert!(result);
        assert!(!state.should_quit); // Should not quit, just close response
        assert!(state.response_buffer.is_none()); // Response buffer should be cleared
        assert_eq!(state.current_pane, Pane::Request); // Should switch to Request pane
        assert_eq!(state.mode, EditorMode::Normal);
        assert!(state.command_buffer.is_empty());
        assert_eq!(state.status_message, "Response pane closed");
    }

    #[test]
    fn execute_q_force_command_should_always_quit() {
        // Test from Request pane
        let mut state = create_test_app_state();
        state.mode = EditorMode::Command;
        state.current_pane = Pane::Request;
        state.command_buffer = "q!".to_string();

        let command = ExecuteCommandCommand::new();
        let event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);

        let result = command.process(event, &mut state).unwrap();
        assert!(result);
        assert!(state.should_quit);

        // Test from Response pane
        let mut state = create_test_app_state();
        state.mode = EditorMode::Command;
        state.current_pane = Pane::Response;
        state.command_buffer = "q!".to_string();

        // Add a response buffer
        let response_content = "HTTP/1.1 200 OK".to_string();
        let response_buffer = ResponseBuffer::new(response_content);
        state.response_buffer = Some(response_buffer);

        let result = command.process(event, &mut state).unwrap();
        assert!(result);
        assert!(state.should_quit); // Should quit even from Response pane
        assert!(state.response_buffer.is_some()); // Response buffer should remain
    }

    #[test]
    fn execute_quit_command_should_work_like_q() {
        // Test "quit" command works the same as "q"
        let mut state = create_test_app_state();
        state.mode = EditorMode::Command;
        state.current_pane = Pane::Request;
        state.command_buffer = "quit".to_string();

        let command = ExecuteCommandCommand::new();
        let event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);

        let result = command.process(event, &mut state).unwrap();
        assert!(result);
        assert!(state.should_quit);
        assert_eq!(state.status_message, "Goodbye!");
    }

    #[test]
    fn execute_x_command_should_set_execution_flag() {
        let mut state = create_test_app_state();
        state.mode = EditorMode::Command;
        state.command_buffer = "x".to_string();

        let command = ExecuteCommandCommand::new();
        let event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);

        let result = command.process(event, &mut state).unwrap();
        assert!(result);
        assert!(state.execute_request_flag);
        assert_eq!(state.status_message, "Executing request...");
        assert_eq!(state.mode, EditorMode::Normal);
    }

    #[test]
    fn execute_execute_command_should_work_like_x() {
        let mut state = create_test_app_state();
        state.mode = EditorMode::Command;
        state.command_buffer = "execute".to_string();

        let command = ExecuteCommandCommand::new();
        let event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);

        let result = command.process(event, &mut state).unwrap();
        assert!(result);
        assert!(state.execute_request_flag);
        assert_eq!(state.status_message, "Executing request...");
    }

    #[test]
    fn execute_unknown_command_should_show_error() {
        let mut state = create_test_app_state();
        state.mode = EditorMode::Command;
        state.command_buffer = "unknown".to_string();

        let command = ExecuteCommandCommand::new();
        let event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);

        let result = command.process(event, &mut state).unwrap();
        assert!(result);
        assert!(!state.should_quit);
        assert_eq!(state.status_message, "Unknown command: unknown");
        assert_eq!(state.mode, EditorMode::Normal);
    }

    #[test]
    fn execute_empty_command_should_just_exit_command_mode() {
        let mut state = create_test_app_state();
        state.mode = EditorMode::Command;
        state.command_buffer = "".to_string();

        let command = ExecuteCommandCommand::new();
        let event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);

        let result = command.process(event, &mut state).unwrap();
        assert!(result);
        assert_eq!(state.mode, EditorMode::Normal);
        assert_eq!(state.status_message, "");
    }

    #[test]
    fn execute_command_command_should_be_relevant_for_enter_in_command_mode() {
        let command = ExecuteCommandCommand::new();
        let mut state = create_test_app_state();
        state.mode = EditorMode::Command;
        let event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);

        assert!(command.is_relevant(&state, &event));
        assert_eq!(command.name(), "ExecuteCommand");
    }
}
