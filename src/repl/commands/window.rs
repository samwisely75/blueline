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
            state.status_message = "Waiting for window command".to_string();
            return Ok(true);
        }

        // Handle second step of Ctrl+W commands
        if state.pending_ctrl_w {
            match event.code {
                KeyCode::Char('w') => {
                    // Ctrl+W w - switch to next window
                    let target_pane = match state.current_pane {
                        Pane::Request => Pane::Response,
                        Pane::Response => Pane::Request,
                    };
                    state.current_pane = target_pane;
                    state.pending_ctrl_w = false;
                    state.status_message = format!("Switch pane to {:?}", target_pane);
                    return Ok(true);
                }
                KeyCode::Esc => {
                    // Cancel Ctrl+W command
                    state.pending_ctrl_w = false;
                    state.status_message = "".to_string(); // Clear status message
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

/// Command for expanding response pane (Ctrl+J)
pub struct ExpandResponsePaneCommand;

impl ExpandResponsePaneCommand {}

impl Command for ExpandResponsePaneCommand {
    fn is_relevant(&self, state: &AppState, event: &KeyEvent) -> bool {
        // Only relevant in Normal mode for Ctrl+J
        matches!(state.mode, EditorMode::Normal)
            && event.modifiers.contains(KeyModifiers::CONTROL)
            && matches!(event.code, KeyCode::Char('j'))
            && state.response_buffer.is_some() // Only when response pane exists
    }

    fn process(&self, _event: KeyEvent, state: &mut AppState) -> Result<bool> {
        // Expand response pane by shrinking request pane
        state.expand_response_pane();
        Ok(true)
    }

    fn name(&self) -> &'static str {
        "ExpandResponsePane"
    }
}

/// Command for shrinking response pane (Ctrl+K)
pub struct ShrinkResponsePaneCommand;

impl ShrinkResponsePaneCommand {}

impl Command for ShrinkResponsePaneCommand {
    fn is_relevant(&self, state: &AppState, event: &KeyEvent) -> bool {
        // Only relevant in Normal mode for Ctrl+K
        matches!(state.mode, EditorMode::Normal)
            && event.modifiers.contains(KeyModifiers::CONTROL)
            && matches!(event.code, KeyCode::Char('k'))
            && state.response_buffer.is_some() // Only when response pane exists
    }

    fn process(&self, _event: KeyEvent, state: &mut AppState) -> Result<bool> {
        // Shrink response pane by expanding request pane
        state.shrink_response_pane();
        Ok(true)
    }

    fn name(&self) -> &'static str {
        "ShrinkResponsePane"
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
        assert_eq!(state.status_message, "Waiting for window command");
    }

    #[test]
    fn switch_pane_command_should_switch_from_request_to_response() {
        let mut state = create_test_app_state();
        state.mode = EditorMode::Normal;
        state.current_pane = Pane::Request;
        state.pending_ctrl_w = true;
        state.status_message = "Waiting for window command".to_string(); // Set initial status
        let command = SwitchPaneCommand;
        let event = KeyEvent::new(KeyCode::Char('w'), KeyModifiers::NONE);

        let result = command.process(event, &mut state).unwrap();

        assert!(result);
        assert_eq!(state.current_pane, Pane::Response);
        assert!(!state.pending_ctrl_w);
        assert_eq!(state.status_message, "Switch pane to Response"); // Should show target pane
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
        assert_eq!(state.status_message, "Switch pane to Request"); // Should show target pane
    }

    #[test]
    fn switch_pane_command_should_cancel_on_escape() {
        let mut state = create_test_app_state();
        state.mode = EditorMode::Normal;
        state.pending_ctrl_w = true;
        state.status_message = "Waiting for window command".to_string(); // Set initial status
        let command = SwitchPaneCommand;
        let event = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);

        let result = command.process(event, &mut state).unwrap();

        assert!(result);
        assert!(!state.pending_ctrl_w);
        assert_eq!(state.status_message, ""); // Should be cleared
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

    #[test]
    fn expand_response_pane_command_should_be_relevant_for_ctrl_j_with_response() {
        let mut state = create_test_app_state();
        state.mode = EditorMode::Normal;
        state.set_response("Test response".to_string());
        let command = ExpandResponsePaneCommand;
        let event = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::CONTROL);

        assert!(command.is_relevant(&state, &event));
    }

    #[test]
    fn expand_response_pane_command_should_not_be_relevant_without_response() {
        let mut state = create_test_app_state();
        state.mode = EditorMode::Normal;
        let command = ExpandResponsePaneCommand;
        let event = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::CONTROL);

        assert!(!command.is_relevant(&state, &event));
    }

    #[test]
    fn expand_response_pane_command_should_not_be_relevant_in_insert_mode() {
        let mut state = create_test_app_state();
        state.mode = EditorMode::Insert;
        state.set_response("Test response".to_string());
        let command = ExpandResponsePaneCommand;
        let event = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::CONTROL);

        assert!(!command.is_relevant(&state, &event));
    }

    #[test]
    fn expand_response_pane_command_should_expand_response_pane() {
        let mut state = create_test_app_state();
        state.mode = EditorMode::Normal;
        state.set_response("Test response".to_string());
        state.request_pane_height = 10;
        let command = ExpandResponsePaneCommand;
        let event = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::CONTROL);

        let result = command.process(event, &mut state).unwrap();

        assert!(result);
        assert_eq!(state.request_pane_height, 9); // Should decrease by 1
    }

    #[test]
    fn expand_response_pane_command_should_respect_minimum_request_height() {
        let mut state = create_test_app_state();
        state.mode = EditorMode::Normal;
        state.set_response("Test response".to_string());
        state.request_pane_height = 3; // At minimum
        let command = ExpandResponsePaneCommand;
        let event = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::CONTROL);

        let result = command.process(event, &mut state).unwrap();

        assert!(result);
        assert_eq!(state.request_pane_height, 3); // Should not go below minimum
    }

    #[test]
    fn shrink_response_pane_command_should_be_relevant_for_ctrl_k_with_response() {
        let mut state = create_test_app_state();
        state.mode = EditorMode::Normal;
        state.set_response("Test response".to_string());
        let command = ShrinkResponsePaneCommand;
        let event = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::CONTROL);

        assert!(command.is_relevant(&state, &event));
    }

    #[test]
    fn shrink_response_pane_command_should_not_be_relevant_without_response() {
        let mut state = create_test_app_state();
        state.mode = EditorMode::Normal;
        let command = ShrinkResponsePaneCommand;
        let event = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::CONTROL);

        assert!(!command.is_relevant(&state, &event));
    }

    #[test]
    fn shrink_response_pane_command_should_not_be_relevant_in_insert_mode() {
        let mut state = create_test_app_state();
        state.mode = EditorMode::Insert;
        state.set_response("Test response".to_string());
        let command = ShrinkResponsePaneCommand;
        let event = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::CONTROL);

        assert!(!command.is_relevant(&state, &event));
    }

    #[test]
    fn shrink_response_pane_command_should_shrink_response_pane() {
        let mut state = create_test_app_state();
        state.mode = EditorMode::Normal;
        state.set_response("Test response".to_string());
        state.request_pane_height = 10;
        let command = ShrinkResponsePaneCommand;
        let event = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::CONTROL);

        let result = command.process(event, &mut state).unwrap();

        assert!(result);
        assert_eq!(state.request_pane_height, 11); // Should increase by 1
    }

    #[test]
    fn shrink_response_pane_command_should_respect_minimum_response_height() {
        let mut state = create_test_app_state();
        state.mode = EditorMode::Normal;
        state.set_response("Test response".to_string());
        // Set request pane height to maximum (terminal height - status line - separator - min response height)
        // Terminal height is 24, minus 2 (status + separator) = 22, minus 3 (min response) = 19
        state.request_pane_height = 19; // At maximum
        let command = ShrinkResponsePaneCommand;
        let event = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::CONTROL);

        let result = command.process(event, &mut state).unwrap();

        assert!(result);
        assert_eq!(state.request_pane_height, 19); // Should not go above maximum
    }

    #[test]
    fn expand_response_pane_command_should_clamp_request_pane_cursor_when_out_of_bounds() {
        let mut state = create_test_app_state();
        state.mode = EditorMode::Normal;
        state.set_response("Test response".to_string());
        state.request_pane_height = 10;

        // Add multiple lines and position cursor beyond where it will be visible after shrinking
        for i in 0..15 {
            state.request_buffer.lines.push(format!("Line {}", i));
        }
        state.request_buffer.cursor_line = 14; // Position cursor at last line

        let command = ExpandResponsePaneCommand;
        let event = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::CONTROL);

        let result = command.process(event, &mut state).unwrap();

        assert!(result);
        assert_eq!(state.request_pane_height, 9); // Shrunk by 1
                                                  // Cursor should be clamped to last visible line (8, 0-indexed)
        assert_eq!(state.request_buffer.cursor_line, 8);
    }

    #[test]
    fn shrink_response_pane_command_should_clamp_response_pane_cursor_when_out_of_bounds() {
        let mut state = create_test_app_state();
        state.mode = EditorMode::Normal;
        state.request_pane_height = 10;

        // Create response buffer with multiple lines
        let mut response_lines = Vec::new();
        for i in 0..15 {
            response_lines.push(format!("Response line {}", i));
        }
        let response_buffer = crate::repl::model::ResponseBuffer::new(response_lines.join("\n"));
        let mut modified_response = response_buffer;
        modified_response.cursor_line = 14; // Position cursor at last line
        state.response_buffer = Some(modified_response);

        let command = ShrinkResponsePaneCommand;
        let event = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::CONTROL);

        let result = command.process(event, &mut state).unwrap();

        assert!(result);
        assert_eq!(state.request_pane_height, 11); // Expanded by 1
                                                   // Response pane height is now smaller, cursor should be clamped
        if let Some(ref response_buffer) = state.response_buffer {
            assert_eq!(response_buffer.cursor_line, 10); // Clamped to last visible line
        } else {
            panic!("Response buffer should exist");
        }
    }

    #[test]
    fn expand_response_pane_command_should_not_clamp_cursor_when_within_bounds() {
        let mut state = create_test_app_state();
        state.mode = EditorMode::Normal;
        state.set_response("Test response".to_string());
        state.request_pane_height = 10;

        // Add some lines and position cursor within bounds
        for i in 0..5 {
            state.request_buffer.lines.push(format!("Line {}", i));
        }
        state.request_buffer.cursor_line = 3; // Position cursor within visible area

        let command = ExpandResponsePaneCommand;
        let event = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::CONTROL);

        let result = command.process(event, &mut state).unwrap();

        assert!(result);
        // Cursor should remain unchanged since it's still within bounds
        assert_eq!(state.request_buffer.cursor_line, 3);
    }

    #[test]
    fn shrink_response_pane_command_should_not_clamp_cursor_when_within_bounds() {
        let mut state = create_test_app_state();
        state.mode = EditorMode::Normal;
        state.request_pane_height = 10;

        // Create response buffer with moderate content
        let response_buffer = crate::repl::model::ResponseBuffer::new(
            "Line 0\nLine 1\nLine 2\nLine 3\nLine 4".to_string(),
        );
        let mut modified_response = response_buffer;
        modified_response.cursor_line = 2; // Position cursor within bounds
        state.response_buffer = Some(modified_response);

        let command = ShrinkResponsePaneCommand;
        let event = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::CONTROL);

        let result = command.process(event, &mut state).unwrap();

        assert!(result);
        // Cursor should remain unchanged since it's still within bounds
        if let Some(ref response_buffer) = state.response_buffer {
            assert_eq!(response_buffer.cursor_line, 2);
        } else {
            panic!("Response buffer should exist");
        }
    }
}
