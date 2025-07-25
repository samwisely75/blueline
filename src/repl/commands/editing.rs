//! # Editing Commands
//!
//! This module contains commands for text editing operations like insertion,
//! deletion, and mode switching.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::repl::{
    command::{Command, CommandResult},
    model::{AppState, EditorMode, Pane},
};

/// Command for entering insert mode (i, I, A)
pub struct EnterInsertModeCommand;

impl EnterInsertModeCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Command for EnterInsertModeCommand {
    fn is_relevant(&self, state: &AppState, _event: &KeyEvent) -> bool {
        // Only relevant in Normal mode, Request pane
        matches!(state.mode, EditorMode::Normal) && state.current_pane == Pane::Request
    }

    fn process(&self, event: KeyEvent, state: &mut AppState) -> Result<bool> {
        // Check if the key matches what we handle
        if event.modifiers != KeyModifiers::NONE {
            return Ok(false);
        }

        match event.code {
            KeyCode::Char('i') => {
                // Insert at current position
                state.mode = EditorMode::Insert;
                state.status_message = "-- INSERT --".to_string();
                Ok(true)
            }
            KeyCode::Char('I') => {
                // Insert at beginning of line
                state.request_buffer.cursor_col = 0;
                state.mode = EditorMode::Insert;
                state.status_message = "-- INSERT --".to_string();
                Ok(true)
            }
            KeyCode::Char('A') => {
                // Append at end of line
                if let Some(line) = state
                    .request_buffer
                    .lines
                    .get(state.request_buffer.cursor_line)
                {
                    state.request_buffer.cursor_col = line.len();
                }
                state.mode = EditorMode::Insert;
                state.status_message = "-- INSERT --".to_string();
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn name(&self) -> &'static str {
        "EnterInsertMode"
    }
}

/// Command for exiting insert mode (Esc)
pub struct ExitInsertModeCommand;

impl ExitInsertModeCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Command for ExitInsertModeCommand {
    fn is_relevant(&self, state: &AppState, _event: &KeyEvent) -> bool {
        // Only relevant in Insert mode
        matches!(state.mode, EditorMode::Insert)
    }

    fn process(&self, event: KeyEvent, state: &mut AppState) -> Result<bool> {
        if matches!(event.code, KeyCode::Esc) {
            state.mode = EditorMode::Normal;
            state.status_message = "".to_string();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn name(&self) -> &'static str {
        "ExitInsertMode"
    }
}

/// Command for inserting characters (any printable character in insert mode)
pub struct InsertCharCommand;

impl InsertCharCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Command for InsertCharCommand {
    fn is_relevant(&self, state: &AppState, _event: &KeyEvent) -> bool {
        // Only relevant in Insert mode, Request pane
        matches!(state.mode, EditorMode::Insert) && state.current_pane == Pane::Request
    }

    fn process(&self, event: KeyEvent, state: &mut AppState) -> Result<bool> {
        // Check for printable characters (no control modifiers)
        if let KeyCode::Char(ch) = event.code {
            if !event.modifiers.contains(KeyModifiers::CONTROL) {
                // Ensure we have a valid line to insert into
                if state.request_buffer.cursor_line >= state.request_buffer.lines.len() {
                    state.request_buffer.lines.push(String::new());
                }

                let line = &mut state.request_buffer.lines[state.request_buffer.cursor_line];
                if state.request_buffer.cursor_col <= line.len() {
                    line.insert(state.request_buffer.cursor_col, ch);
                    state.request_buffer.cursor_col += 1;
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    fn name(&self) -> &'static str {
        "InsertChar"
    }
}

/// Command for inserting new lines (Enter in insert mode)
pub struct InsertNewLineCommand;

impl InsertNewLineCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Command for InsertNewLineCommand {
    fn is_relevant(&self, state: &AppState, _event: &KeyEvent) -> bool {
        // Only relevant in Insert mode, Request pane
        matches!(state.mode, EditorMode::Insert) && state.current_pane == Pane::Request
    }

    fn process(&self, event: KeyEvent, state: &mut AppState) -> Result<bool> {
        if matches!(event.code, KeyCode::Enter) {
            // Ensure we have a valid line
            if state.request_buffer.cursor_line >= state.request_buffer.lines.len() {
                state.request_buffer.lines.push(String::new());
            }

            let line = &mut state.request_buffer.lines[state.request_buffer.cursor_line];
            let remainder = line.split_off(state.request_buffer.cursor_col);
            state.request_buffer.cursor_line += 1;
            state
                .request_buffer
                .lines
                .insert(state.request_buffer.cursor_line, remainder);
            state.request_buffer.cursor_col = 0;

            // Auto-scroll down if cursor goes below visible area
            let visible_height = state.get_request_pane_height();
            if state.request_buffer.cursor_line
                >= state.request_buffer.scroll_offset + visible_height
            {
                state.request_buffer.scroll_offset =
                    state.request_buffer.cursor_line - visible_height + 1;
            }

            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn name(&self) -> &'static str {
        "InsertNewLine"
    }
}

/// Command for deleting characters (Backspace in insert mode)
pub struct DeleteCharCommand;

impl DeleteCharCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Command for DeleteCharCommand {
    fn is_relevant(&self, state: &AppState, _event: &KeyEvent) -> bool {
        // Only relevant in Insert mode, Request pane
        matches!(state.mode, EditorMode::Insert) && state.current_pane == Pane::Request
    }

    fn process(&self, event: KeyEvent, state: &mut AppState) -> Result<bool> {
        if matches!(event.code, KeyCode::Backspace) {
            if state.request_buffer.cursor_line >= state.request_buffer.lines.len() {
                return Ok(false);
            }

            let line = &mut state.request_buffer.lines[state.request_buffer.cursor_line];
            if state.request_buffer.cursor_col > 0 && state.request_buffer.cursor_col <= line.len()
            {
                line.remove(state.request_buffer.cursor_col - 1);
                state.request_buffer.cursor_col -= 1;
                Ok(true)
            } else if state.request_buffer.cursor_col == 0 && state.request_buffer.cursor_line > 0 {
                // At beginning of line, join with previous line
                let current_line = state
                    .request_buffer
                    .lines
                    .remove(state.request_buffer.cursor_line);
                state.request_buffer.cursor_line -= 1;
                state.request_buffer.cursor_col =
                    state.request_buffer.lines[state.request_buffer.cursor_line].len();
                state.request_buffer.lines[state.request_buffer.cursor_line]
                    .push_str(&current_line);
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            Ok(false)
        }
    }

    fn name(&self) -> &'static str {
        "DeleteChar"
    }
}

/// Command for entering command mode (:)
pub struct EnterCommandModeCommand;

impl EnterCommandModeCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Command for EnterCommandModeCommand {
    fn is_relevant(&self, state: &AppState, _event: &KeyEvent) -> bool {
        // Only relevant in Normal mode
        matches!(state.mode, EditorMode::Normal)
    }

    fn process(&self, event: KeyEvent, state: &mut AppState) -> Result<bool> {
        if matches!(event.code, KeyCode::Char(':')) && event.modifiers == KeyModifiers::NONE {
            state.mode = EditorMode::Command;
            state.command_buffer.clear();
            state.status_message = ":".to_string();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn name(&self) -> &'static str {
        "EnterCommandMode"
    }
}

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

/// Command for canceling command mode (Esc)
pub struct CancelCommandModeCommand;

impl CancelCommandModeCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Command for CancelCommandModeCommand {
    fn is_relevant(&self, state: &AppState, _event: &KeyEvent) -> bool {
        // Only relevant in Command mode
        matches!(state.mode, EditorMode::Command)
    }

    fn process(&self, event: KeyEvent, state: &mut AppState) -> Result<bool> {
        if matches!(event.code, KeyCode::Esc) {
            // Cancel command mode and return to normal mode
            state.mode = EditorMode::Normal;
            state.command_buffer.clear();
            state.status_message = "".to_string();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn name(&self) -> &'static str {
        "CancelCommandMode"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::model::{AppState, ResponseBuffer};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn execute_q_command_should_quit_from_request_pane() {
        // Create a test state in Request pane
        let mut state = AppState::new((80, 24), false);
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
        let mut state = AppState::new((80, 24), false);
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
        let mut state = AppState::new((80, 24), false);
        state.mode = EditorMode::Command;
        state.current_pane = Pane::Request;
        state.command_buffer = "q!".to_string();

        let command = ExecuteCommandCommand::new();
        let event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);

        let result = command.process(event, &mut state).unwrap();
        assert!(result);
        assert!(state.should_quit);

        // Test from Response pane
        let mut state = AppState::new((80, 24), false);
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
        let mut state = AppState::new((80, 24), false);
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
    fn execute_unknown_command_should_show_error() {
        let mut state = AppState::new((80, 24), false);
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
}
