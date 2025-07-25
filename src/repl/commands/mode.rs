//! # Mode Transition Commands
//!
//! This module contains commands for switching between different editor modes:
//! Normal, Insert, and Command modes. These follow vi-style mode transitions.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::repl::{
    commands::{Command, CommandResult},
    model::{AppState, EditorMode, Pane},
};

/// Command for entering insert mode (i, I, A)
pub struct EnterInsertModeCommand;

impl EnterInsertModeCommand {}

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

impl ExitInsertModeCommand {}

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

/// Command for entering command mode (:)
pub struct EnterCommandModeCommand;

impl EnterCommandModeCommand {}

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

/// Command for canceling command mode (Esc)
pub struct CancelCommandModeCommand;

impl CancelCommandModeCommand {}

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
    use crate::repl::model::AppState;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    /// Create a test AppState for command testing
    fn create_test_app_state() -> AppState {
        AppState::new((80, 24), false)
    }

    #[test]
    fn enter_insert_mode_command_should_be_relevant_for_i_key_in_normal_mode() {
        let command = EnterInsertModeCommand;
        let state = create_test_app_state();
        let event = KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE);

        assert!(command.is_relevant(&state, &event));
        assert_eq!(command.name(), "EnterInsertMode");
    }

    #[test]
    fn enter_insert_mode_command_should_not_be_relevant_in_insert_mode() {
        let command = EnterInsertModeCommand;
        let mut state = create_test_app_state();
        state.mode = EditorMode::Insert;
        let event = KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE);

        assert!(!command.is_relevant(&state, &event));
    }

    #[test]
    fn enter_insert_mode_with_i_should_maintain_cursor_position() {
        let command = EnterInsertModeCommand;
        let mut state = create_test_app_state();
        state.request_buffer.cursor_col = 5;
        let event = KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE);

        let result = command.process(event, &mut state).unwrap();

        assert!(result);
        assert_eq!(state.mode, EditorMode::Insert);
        assert_eq!(state.request_buffer.cursor_col, 5); // Position unchanged
        assert_eq!(state.status_message, "-- INSERT --");
    }

    #[test]
    fn enter_insert_mode_with_capital_i_should_move_cursor_to_line_start() {
        let command = EnterInsertModeCommand;
        let mut state = create_test_app_state();
        state.request_buffer.cursor_col = 5;
        let event = KeyEvent::new(KeyCode::Char('I'), KeyModifiers::NONE);

        let result = command.process(event, &mut state).unwrap();

        assert!(result);
        assert_eq!(state.mode, EditorMode::Insert);
        assert_eq!(state.request_buffer.cursor_col, 0); // Moved to start
        assert_eq!(state.status_message, "-- INSERT --");
    }

    #[test]
    fn enter_insert_mode_with_a_should_move_cursor_to_line_end() {
        let command = EnterInsertModeCommand;
        let mut state = create_test_app_state();
        state.request_buffer.lines = vec!["hello world".to_string()];
        state.request_buffer.cursor_col = 5;
        let event = KeyEvent::new(KeyCode::Char('A'), KeyModifiers::NONE);

        let result = command.process(event, &mut state).unwrap();

        assert!(result);
        assert_eq!(state.mode, EditorMode::Insert);
        assert_eq!(state.request_buffer.cursor_col, 11); // Moved to end
        assert_eq!(state.status_message, "-- INSERT --");
    }

    #[test]
    fn exit_insert_mode_command_should_be_relevant_for_esc_in_insert_mode() {
        let command = ExitInsertModeCommand;
        let mut state = create_test_app_state();
        state.mode = EditorMode::Insert;
        let event = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);

        assert!(command.is_relevant(&state, &event));
        assert_eq!(command.name(), "ExitInsertMode");
    }

    #[test]
    fn exit_insert_mode_command_should_not_be_relevant_in_normal_mode() {
        let command = ExitInsertModeCommand;
        let state = create_test_app_state();
        let event = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);

        assert!(!command.is_relevant(&state, &event));
    }

    #[test]
    fn exit_insert_mode_should_return_to_normal_mode_and_clear_status() {
        let command = ExitInsertModeCommand;
        let mut state = create_test_app_state();
        state.mode = EditorMode::Insert;
        state.status_message = "-- INSERT --".to_string();
        let event = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);

        let result = command.process(event, &mut state).unwrap();

        assert!(result);
        assert_eq!(state.mode, EditorMode::Normal);
        assert_eq!(state.status_message, "");
    }

    #[test]
    fn enter_command_mode_command_should_be_relevant_for_colon_in_normal_mode() {
        let command = EnterCommandModeCommand;
        let state = create_test_app_state();
        let event = KeyEvent::new(KeyCode::Char(':'), KeyModifiers::NONE);

        assert!(command.is_relevant(&state, &event));
        assert_eq!(command.name(), "EnterCommandMode");
    }

    #[test]
    fn enter_command_mode_should_switch_to_command_mode_and_clear_buffer() {
        let command = EnterCommandModeCommand;
        let mut state = create_test_app_state();
        state.command_buffer = "old_command".to_string();
        let event = KeyEvent::new(KeyCode::Char(':'), KeyModifiers::NONE);

        let result = command.process(event, &mut state).unwrap();

        assert!(result);
        assert_eq!(state.mode, EditorMode::Command);
        assert!(state.command_buffer.is_empty());
        assert_eq!(state.status_message, ":");
    }

    #[test]
    fn cancel_command_mode_command_should_be_relevant_for_esc_in_command_mode() {
        let command = CancelCommandModeCommand;
        let mut state = create_test_app_state();
        state.mode = EditorMode::Command;
        let event = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);

        assert!(command.is_relevant(&state, &event));
        assert_eq!(command.name(), "CancelCommandMode");
    }

    #[test]
    fn cancel_command_mode_should_return_to_normal_mode_and_clear_state() {
        let command = CancelCommandModeCommand;
        let mut state = create_test_app_state();
        state.mode = EditorMode::Command;
        state.command_buffer = "partial_command".to_string();
        state.status_message = ":partial_command".to_string();
        let event = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);

        let result = command.process(event, &mut state).unwrap();

        assert!(result);
        assert_eq!(state.mode, EditorMode::Normal);
        assert!(state.command_buffer.is_empty());
        assert_eq!(state.status_message, "");
    }
}
