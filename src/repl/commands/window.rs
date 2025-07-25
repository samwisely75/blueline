//! # Window Commands
//!
//! This module contains command implementations for window management operations,
//! such as switching between panes and managing window layout.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::repl::{
    commands::Command,
    model::{AppState, EditorMode, Pane},
};

/// Command for switching between panes (Ctrl+W w)
pub struct SwitchPaneCommand;

impl SwitchPaneCommand {}

impl Command for SwitchPaneCommand {
    fn is_relevant(&self, state: &AppState, event: &KeyEvent) -> bool {
        // Only relevant in Normal mode and for Ctrl+W sequences or pending Ctrl+W state
        matches!(state.mode, EditorMode::Normal)
            && ((event.modifiers.contains(KeyModifiers::CONTROL)
                && matches!(event.code, KeyCode::Char('w')))
                || state.pending_ctrl_w)
    }

    fn process(&self, event: KeyEvent, state: &mut AppState) -> Result<bool> {
        // Handle Ctrl+W (first step)
        if event.modifiers.contains(KeyModifiers::CONTROL)
            && matches!(event.code, KeyCode::Char('w'))
        {
            state.pending_ctrl_w = true;
            return Ok(true);
        }

        // Handle second step of Ctrl+W commands
        if state.pending_ctrl_w {
            match event.code {
                KeyCode::Char('w') => {
                    // Ctrl+W w - switch to next window
                    state.current_pane = match state.current_pane {
                        Pane::Request => Pane::Response,
                        Pane::Response => Pane::Request,
                    };
                    state.pending_ctrl_w = false;
                    return Ok(true);
                }
                KeyCode::Esc => {
                    // Cancel Ctrl+W command
                    state.pending_ctrl_w = false;
                    return Ok(true);
                }
                _ => {
                    // Invalid Ctrl+W command
                    state.pending_ctrl_w = false;
                    state.status_message = "Invalid window command".to_string();
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    fn name(&self) -> &'static str {
        "SwitchPane"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyCode;

    /// Create a test AppState for command testing
    fn create_test_app_state() -> AppState {
        AppState::new((80, 24), false)
    }

    #[test]
    fn switch_pane_command_should_be_relevant_for_ctrl_w_in_normal_mode() {
        let mut state = create_test_app_state();
        state.mode = EditorMode::Normal;
        let command = SwitchPaneCommand;
        let event = KeyEvent::new(KeyCode::Char('w'), KeyModifiers::CONTROL);

        assert!(command.is_relevant(&state, &event));
    }

    #[test]
    fn switch_pane_command_should_not_be_relevant_in_insert_mode() {
        let mut state = create_test_app_state();
        state.mode = EditorMode::Insert;
        let command = SwitchPaneCommand;
        let event = KeyEvent::new(KeyCode::Char('w'), KeyModifiers::CONTROL);

        assert!(!command.is_relevant(&state, &event));
    }

    #[test]
    fn switch_pane_command_should_set_pending_ctrl_w_on_first_step() {
        let mut state = create_test_app_state();
        state.mode = EditorMode::Normal;
        let command = SwitchPaneCommand;
        let event = KeyEvent::new(KeyCode::Char('w'), KeyModifiers::CONTROL);

        let result = command.process(event, &mut state).unwrap();

        assert!(result);
        assert!(state.pending_ctrl_w);
    }

    #[test]
    fn switch_pane_command_should_switch_from_request_to_response() {
        let mut state = create_test_app_state();
        state.mode = EditorMode::Normal;
        state.current_pane = Pane::Request;
        state.pending_ctrl_w = true;
        let command = SwitchPaneCommand;
        let event = KeyEvent::new(KeyCode::Char('w'), KeyModifiers::NONE);

        let result = command.process(event, &mut state).unwrap();

        assert!(result);
        assert_eq!(state.current_pane, Pane::Response);
        assert!(!state.pending_ctrl_w);
    }

    #[test]
    fn switch_pane_command_should_switch_from_response_to_request() {
        let mut state = create_test_app_state();
        state.mode = EditorMode::Normal;
        state.current_pane = Pane::Response;
        state.pending_ctrl_w = true;
        let command = SwitchPaneCommand;
        let event = KeyEvent::new(KeyCode::Char('w'), KeyModifiers::NONE);

        let result = command.process(event, &mut state).unwrap();

        assert!(result);
        assert_eq!(state.current_pane, Pane::Request);
        assert!(!state.pending_ctrl_w);
    }

    #[test]
    fn switch_pane_command_should_cancel_on_escape() {
        let mut state = create_test_app_state();
        state.mode = EditorMode::Normal;
        state.pending_ctrl_w = true;
        let command = SwitchPaneCommand;
        let event = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);

        let result = command.process(event, &mut state).unwrap();

        assert!(result);
        assert!(!state.pending_ctrl_w);
    }

    #[test]
    fn switch_pane_command_should_handle_invalid_second_key() {
        let mut state = create_test_app_state();
        state.mode = EditorMode::Normal;
        state.pending_ctrl_w = true;
        let command = SwitchPaneCommand;
        let event = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);

        let result = command.process(event, &mut state).unwrap();

        assert!(result);
        assert!(!state.pending_ctrl_w);
        assert_eq!(state.status_message, "Invalid window command");
    }
}
