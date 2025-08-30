//! # Move Cursor Left Command
//!
//! Unified command for moving cursor left using 'h' key or Left arrow key.
//! Replaces the legacy MoveCursorLeftCommand from the navigation.rs module.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::repl::models::events::view_events::ViewEvent;
use crate::repl::models::pane_state::EditorMode;
use crate::repl::unified_commands::{Command, CommandContext, ExecutionContext};

/// Command to move cursor left
///
/// Handles:
/// - 'h' key in navigation modes (Normal, Visual, VisualLine, VisualBlock) without modifiers
/// - Left arrow key without Shift or Control modifiers
pub struct MoveCursorLeftCommand;

impl MoveCursorLeftCommand {
    /// Create new MoveCursorLeftCommand
    pub fn new() -> Self {
        Self
    }

    /// Check if the current mode supports navigation
    fn is_navigation_mode(mode: EditorMode) -> bool {
        matches!(
            mode,
            EditorMode::Normal | EditorMode::Visual | EditorMode::VisualLine | EditorMode::VisualBlock
        )
    }
}

impl Command for MoveCursorLeftCommand {
    fn is_relevant(&self, key_event: KeyEvent, mode: EditorMode, context: &CommandContext) -> bool {
        // Only relevant in request pane (not read-only response pane)
        if context.is_read_only {
            return false;
        }

        match key_event.code {
            KeyCode::Char('h') => {
                Self::is_navigation_mode(mode) && key_event.modifiers.is_empty()
            }
            KeyCode::Left => {
                !key_event.modifiers.contains(KeyModifiers::SHIFT)
                    && !key_event.modifiers.contains(KeyModifiers::CONTROL)
            }
            _ => false,
        }
    }

    fn execute(&self, context: &mut ExecutionContext) -> Result<Vec<ViewEvent>> {
        // Move cursor left using AppState's method
        context.app_state.move_cursor_left()?;

        // Return appropriate ViewEvents for cursor movement
        Ok(vec![
            ViewEvent::ActiveCursorUpdateRequired,
            ViewEvent::PositionIndicatorUpdateRequired,
        ])
    }

    fn name(&self) -> &'static str {
        "MoveCursorLeft"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    use crate::repl::models::pane_state::{EditorMode, Pane};
    use crate::repl::models::AppState;
    use crate::repl::services::Services;
    use crate::repl::unified_commands::{CommandContext, ExecutionContext};

    fn create_test_context() -> CommandContext {
        CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
        }
    }

    fn create_test_key_event(key: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent::new(key, modifiers)
    }

    #[test]
    fn move_cursor_left_should_be_relevant_for_h_in_normal_mode() {
        let command = MoveCursorLeftCommand::new();
        let context = create_test_context();
        let event = create_test_key_event(KeyCode::Char('h'), KeyModifiers::NONE);

        assert!(command.is_relevant(event, EditorMode::Normal, &context));
    }

    #[test]
    fn move_cursor_left_should_be_relevant_for_h_in_visual_mode() {
        let command = MoveCursorLeftCommand::new();
        let context = create_test_context();
        let event = create_test_key_event(KeyCode::Char('h'), KeyModifiers::NONE);

        assert!(command.is_relevant(event, EditorMode::Visual, &context));
    }

    #[test]
    fn move_cursor_left_should_be_relevant_for_h_in_visual_line_mode() {
        let command = MoveCursorLeftCommand::new();
        let context = create_test_context();
        let event = create_test_key_event(KeyCode::Char('h'), KeyModifiers::NONE);

        assert!(command.is_relevant(event, EditorMode::VisualLine, &context));
    }

    #[test]
    fn move_cursor_left_should_be_relevant_for_h_in_visual_block_mode() {
        let command = MoveCursorLeftCommand::new();
        let context = create_test_context();
        let event = create_test_key_event(KeyCode::Char('h'), KeyModifiers::NONE);

        assert!(command.is_relevant(event, EditorMode::VisualBlock, &context));
    }

    #[test]
    fn move_cursor_left_should_be_relevant_for_left_arrow() {
        let command = MoveCursorLeftCommand::new();
        let context = create_test_context();
        let event = create_test_key_event(KeyCode::Left, KeyModifiers::NONE);

        assert!(command.is_relevant(event, EditorMode::Normal, &context));
    }

    #[test]
    fn move_cursor_left_should_not_be_relevant_for_h_with_modifiers() {
        let command = MoveCursorLeftCommand::new();
        let context = create_test_context();
        let event = create_test_key_event(KeyCode::Char('h'), KeyModifiers::CONTROL);

        assert!(!command.is_relevant(event, EditorMode::Normal, &context));
    }

    #[test]
    fn move_cursor_left_should_not_be_relevant_for_left_with_shift() {
        let command = MoveCursorLeftCommand::new();
        let context = create_test_context();
        let event = create_test_key_event(KeyCode::Left, KeyModifiers::SHIFT);

        assert!(!command.is_relevant(event, EditorMode::Normal, &context));
    }

    #[test]
    fn move_cursor_left_should_not_be_relevant_for_left_with_control() {
        let command = MoveCursorLeftCommand::new();
        let context = create_test_context();
        let event = create_test_key_event(KeyCode::Left, KeyModifiers::CONTROL);

        assert!(!command.is_relevant(event, EditorMode::Normal, &context));
    }

    #[test]
    fn move_cursor_left_should_not_be_relevant_in_insert_mode() {
        let command = MoveCursorLeftCommand::new();
        let context = create_test_context();
        let event = create_test_key_event(KeyCode::Char('h'), KeyModifiers::NONE);

        assert!(!command.is_relevant(event, EditorMode::Insert, &context));
    }

    #[test]
    fn move_cursor_left_should_not_be_relevant_in_read_only_pane() {
        let command = MoveCursorLeftCommand::new();
        let mut context = create_test_context();
        context.is_read_only = true;
        let event = create_test_key_event(KeyCode::Char('h'), KeyModifiers::NONE);

        assert!(!command.is_relevant(event, EditorMode::Normal, &context));
    }

    #[test]
    fn move_cursor_left_should_not_be_relevant_for_other_keys() {
        let command = MoveCursorLeftCommand::new();
        let context = create_test_context();
        let event = create_test_key_event(KeyCode::Char('j'), KeyModifiers::NONE);

        assert!(!command.is_relevant(event, EditorMode::Normal, &context));
    }

    #[test]
    fn move_cursor_left_should_return_correct_name() {
        let command = MoveCursorLeftCommand::new();
        assert_eq!(command.name(), "MoveCursorLeft");
    }

    #[test]
    fn move_cursor_left_should_emit_cursor_update_events() {
        let command = MoveCursorLeftCommand::new();
        let mut app_state = AppState::new();
        let mut services = Services::new();
        let mut context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        // Set up some content to move cursor within
        context.app_state.insert_text("hello world").expect("Failed to insert text");

        let result = command.execute(&mut context).expect("Command execution failed");

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], ViewEvent::ActiveCursorUpdateRequired);
        assert_eq!(result[1], ViewEvent::PositionIndicatorUpdateRequired);
    }
}