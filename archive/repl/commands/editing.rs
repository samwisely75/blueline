//! # Text Editing Commands
//!
//! This module contains commands for text manipulation in insert mode,
//! such as inserting characters, new lines, and deleting characters.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::repl::{
    commands::{Command, CommandResult, MvvmCommand},
    model::{AppState, EditorMode, Pane},
    view_model::ViewModel,
};

/// Command for inserting characters (any printable character in insert mode)
pub struct InsertCharCommand;

impl InsertCharCommand {}

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

                    // Update display cache after content change
                    if let Err(e) = state.update_display_cache_auto() {
                        eprintln!("Failed to update display cache: {}", e);
                    }

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

// ============================================================================
// 🚀 MVVM Implementation for InsertCharCommand
// ============================================================================

impl MvvmCommand for InsertCharCommand {
    fn is_relevant(&self, view_model: &ViewModel, event: &KeyEvent) -> bool {
        // Check if we're in Insert mode and Request pane
        use crate::repl::model::{EditorMode, Pane};
        
        match event.code {
            KeyCode::Char(ch) => {
                // Only handle printable characters (no control modifiers)
                !event.modifiers.contains(KeyModifiers::CONTROL)
                    && view_model.editor.mode == EditorMode::Insert
                    && view_model.editor.current_pane == Pane::Request
            }
            _ => false,
        }
    }
    
    fn execute(&self, event: KeyEvent, view_model: &mut ViewModel) -> Result<bool> {
        if let KeyCode::Char(ch) = event.code {
            // Delegate to ViewModel - all the complex logic is handled there!
            view_model.insert_char(ch)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    fn name(&self) -> &'static str {
        "InsertChar"
    }
}

/// Command for inserting new lines (Enter in insert mode)
pub struct InsertNewLineCommand;

impl InsertNewLineCommand {}

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

            // Update display cache after content change
            if let Err(e) = state.update_display_cache_auto() {
                eprintln!("Failed to update display cache: {}", e);
            }

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

impl DeleteCharCommand {}

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

            let mut content_changed = false;
            let line = &mut state.request_buffer.lines[state.request_buffer.cursor_line];
            if state.request_buffer.cursor_col > 0 && state.request_buffer.cursor_col <= line.len()
            {
                line.remove(state.request_buffer.cursor_col - 1);
                state.request_buffer.cursor_col -= 1;
                content_changed = true;
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
                content_changed = true;
            }

            if content_changed {
                // Update display cache after content change
                if let Err(e) = state.update_display_cache_auto() {
                    eprintln!("Failed to update display cache: {}", e);
                }
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
    fn insert_char_command_should_be_relevant_in_insert_mode() {
        let command = InsertCharCommand;
        let mut state = create_test_app_state();
        state.mode = EditorMode::Insert;
        let event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);

        assert!(Command::is_relevant(&command, &state, &event));
        assert_eq!(Command::name(&command), "InsertChar");
    }

    #[test]
    fn insert_char_command_should_not_be_relevant_in_normal_mode() {
        let command = InsertCharCommand;
        let state = create_test_app_state();
        let event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);

        assert!(!Command::is_relevant(&command, &state, &event));
    }

    #[test]
    fn insert_char_should_add_character_at_cursor_position() {
        let command = InsertCharCommand;
        let mut state = create_test_app_state();
        state.mode = EditorMode::Insert;
        state.request_buffer.lines = vec!["hello".to_string()];
        state.request_buffer.cursor_col = 3;
        let event = KeyEvent::new(KeyCode::Char('X'), KeyModifiers::NONE);

        let result = command.process(event, &mut state).unwrap();

        assert!(result);
        assert_eq!(state.request_buffer.lines[0], "helXlo");
        assert_eq!(state.request_buffer.cursor_col, 4);
    }

    #[test]
    fn insert_char_should_not_handle_control_characters() {
        let command = InsertCharCommand;
        let mut state = create_test_app_state();
        state.mode = EditorMode::Insert;
        let event = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);

        let result = command.process(event, &mut state).unwrap();

        assert!(!result);
    }

    #[test]
    fn insert_char_should_create_line_if_not_exists() {
        let command = InsertCharCommand;
        let mut state = create_test_app_state();
        state.mode = EditorMode::Insert;
        state.request_buffer.lines = vec![];
        state.request_buffer.cursor_line = 0;
        state.request_buffer.cursor_col = 0;
        let event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);

        let result = command.process(event, &mut state).unwrap();

        assert!(result);
        assert_eq!(state.request_buffer.lines.len(), 1);
        assert_eq!(state.request_buffer.lines[0], "a");
        assert_eq!(state.request_buffer.cursor_col, 1);
    }

    #[test]
    fn insert_new_line_command_should_be_relevant_in_insert_mode() {
        let command = InsertNewLineCommand;
        let mut state = create_test_app_state();
        state.mode = EditorMode::Insert;
        let event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);

        assert!(command.is_relevant(&state, &event));
        assert_eq!(command.name(), "InsertNewLine");
    }

    #[test]
    fn insert_new_line_should_split_line_at_cursor() {
        let command = InsertNewLineCommand;
        let mut state = create_test_app_state();
        state.mode = EditorMode::Insert;
        state.request_buffer.lines = vec!["hello world".to_string()];
        state.request_buffer.cursor_col = 5;
        let event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);

        let result = command.process(event, &mut state).unwrap();

        assert!(result);
        assert_eq!(state.request_buffer.lines.len(), 2);
        assert_eq!(state.request_buffer.lines[0], "hello");
        assert_eq!(state.request_buffer.lines[1], " world");
        assert_eq!(state.request_buffer.cursor_line, 1);
        assert_eq!(state.request_buffer.cursor_col, 0);
    }

    #[test]
    fn insert_new_line_should_handle_auto_scroll() {
        let command = InsertNewLineCommand;
        let mut state = create_test_app_state();
        state.mode = EditorMode::Insert;
        // Set up a small visible height to trigger scrolling
        state.terminal_size = (80, 5); // Small terminal
        state.request_buffer.lines = vec!["line1".to_string(), "line2".to_string()];
        state.request_buffer.cursor_line = 1;
        state.request_buffer.cursor_col = 0;
        let event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);

        let result = command.process(event, &mut state).unwrap();

        assert!(result);
        assert_eq!(state.request_buffer.cursor_line, 2);
        // Should have scrolled due to small visible height
    }

    #[test]
    fn delete_char_command_should_be_relevant_in_insert_mode() {
        let command = DeleteCharCommand;
        let mut state = create_test_app_state();
        state.mode = EditorMode::Insert;
        let event = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);

        assert!(command.is_relevant(&state, &event));
        assert_eq!(command.name(), "DeleteChar");
    }

    #[test]
    fn delete_char_should_remove_character_before_cursor() {
        let command = DeleteCharCommand;
        let mut state = create_test_app_state();
        state.mode = EditorMode::Insert;
        state.request_buffer.lines = vec!["hello".to_string()];
        state.request_buffer.cursor_col = 3;
        let event = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);

        let result = command.process(event, &mut state).unwrap();

        assert!(result);
        assert_eq!(state.request_buffer.lines[0], "helo");
        assert_eq!(state.request_buffer.cursor_col, 2);
    }

    #[test]
    fn delete_char_should_join_lines_when_at_line_start() {
        let command = DeleteCharCommand;
        let mut state = create_test_app_state();
        state.mode = EditorMode::Insert;
        state.request_buffer.lines = vec!["hello".to_string(), "world".to_string()];
        state.request_buffer.cursor_line = 1;
        state.request_buffer.cursor_col = 0;
        let event = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);

        let result = command.process(event, &mut state).unwrap();

        assert!(result);
        assert_eq!(state.request_buffer.lines.len(), 1);
        assert_eq!(state.request_buffer.lines[0], "helloworld");
        assert_eq!(state.request_buffer.cursor_line, 0);
        assert_eq!(state.request_buffer.cursor_col, 5);
    }

    #[test]
    fn delete_char_should_not_delete_when_at_start_of_first_line() {
        let command = DeleteCharCommand;
        let mut state = create_test_app_state();
        state.mode = EditorMode::Insert;
        state.request_buffer.lines = vec!["hello".to_string()];
        state.request_buffer.cursor_line = 0;
        state.request_buffer.cursor_col = 0;
        let event = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);

        let result = command.process(event, &mut state).unwrap();

        assert!(!result);
        assert_eq!(state.request_buffer.lines[0], "hello");
        assert_eq!(state.request_buffer.cursor_col, 0);
    }

    // ============================================================================
    // 🧪 MVVM Tests for InsertCharCommand
    // ============================================================================

    #[test]
    fn mvvm_insert_char_should_be_relevant_in_insert_mode() {
        use crate::repl::view_model::ViewModel;
        use crate::repl::model::EditorMode;
        use crate::repl::commands::MvvmCommand;
        
        let command = InsertCharCommand;
        let mut view_model = ViewModel::new();
        
        // Set up insert mode
        view_model.editor.mode = EditorMode::Insert;
        
        let event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        
        // Should be relevant in insert mode
        assert!(MvvmCommand::is_relevant(&command, &view_model, &event));
        
        // Should not be relevant for control characters
        let ctrl_event = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert!(!MvvmCommand::is_relevant(&command, &view_model, &ctrl_event));
    }

    #[test]
    fn mvvm_insert_char_should_delegate_to_view_model() {
        use crate::repl::view_model::ViewModel;
        use crate::repl::model::EditorMode;
        use crate::repl::commands::MvvmCommand;
        
        let command = InsertCharCommand;
        let mut view_model = ViewModel::new();
        
        // Set up insert mode
        view_model.editor.mode = EditorMode::Insert;
        
        let event = KeyEvent::new(KeyCode::Char('X'), KeyModifiers::NONE);
        
        // Execute the command
        let result = command.execute(event, &mut view_model).unwrap();
        
        // Should be handled
        assert!(result);
        assert_eq!(MvvmCommand::name(&command), "InsertChar");
        
        // ViewModel should have handled the character insertion
        // Note: This is a simplified test - in practice, the buffer content
        // would be checked through the ViewModel's public interface
    }
}
