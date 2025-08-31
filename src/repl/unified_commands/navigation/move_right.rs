//! # Move Right Command
//!
//! Command to move cursor right in Normal and Visual modes.
//! This handles 'l' key presses and right arrow keys.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::register_command;
use crate::repl::models::events::view_events::ViewEvent;
use crate::repl::models::pane_state::EditorMode;
use crate::repl::unified_commands::{Command, CommandContext, ExecutionContext};

/// Command to move cursor right by one character position
///
/// This command:
/// 1. Handles 'l' key in Normal and Visual modes without modifiers
/// 2. Handles Right arrow key without Shift or Control modifiers
/// 3. Uses AppState's move_cursor_right() method for business logic
/// 4. Works in both writable and read-only panes
/// 5. Supports visual mode selection extension
pub struct MoveRightCommand;

impl MoveRightCommand {
    /// Create new MoveRightCommand
    pub fn new() -> Self {
        Self
    }

    /// Check if the given mode supports cursor movement
    fn is_movement_mode(mode: EditorMode) -> bool {
        matches!(
            mode,
            EditorMode::Normal
                | EditorMode::Visual
                | EditorMode::VisualLine
                | EditorMode::VisualBlock
        )
    }
}

impl Command for MoveRightCommand {
    fn is_relevant(
        &self,
        key_event: KeyEvent,
        mode: EditorMode,
        _context: &CommandContext,
    ) -> bool {
        match key_event.code {
            // 'l' key only works in navigation modes (Vim behavior)
            KeyCode::Char('l') => Self::is_movement_mode(mode) && key_event.modifiers.is_empty(),
            // Right arrow key works in all modes (standard editor behavior)
            KeyCode::Right => {
                !key_event.modifiers.contains(KeyModifiers::SHIFT)
                    && !key_event.modifiers.contains(KeyModifiers::CONTROL)
            }
            _ => false,
        }
    }

    fn execute(&self, context: &mut ExecutionContext) -> Result<Vec<ViewEvent>> {
        // Use PaneManager's cursor movement business logic that returns ViewEvents
        let events = context.app_state.pane_manager.move_cursor_right();

        tracing::debug!(
            "MoveRightCommand executed, generated {} events",
            events.len()
        );

        Ok(events)
    }

    fn name(&self) -> &'static str {
        "MoveRightCommand"
    }
}

impl Default for MoveRightCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::models::pane_state::Pane;
    use crate::repl::models::AppState;
    use crate::repl::services::Services;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn create_execution_context() -> (AppState, Services) {
        let app_state = AppState::new();
        let services = Services::new();
        (app_state, services)
    }

    fn create_key_event(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent::new(code, modifiers)
    }

    #[test]
    fn move_right_should_be_relevant_for_l_key_in_navigation_modes() {
        let command = MoveRightCommand::new();
        let key_event = create_key_event(KeyCode::Char('l'), KeyModifiers::NONE);
        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        // Test in Normal mode
        assert!(command.is_relevant(key_event, EditorMode::Normal, &context));

        // Test in Visual modes
        assert!(command.is_relevant(key_event, EditorMode::Visual, &context));
        assert!(command.is_relevant(key_event, EditorMode::VisualLine, &context));
        assert!(command.is_relevant(key_event, EditorMode::VisualBlock, &context));
    }

    #[test]
    fn move_right_should_be_relevant_for_right_arrow_without_modifiers() {
        let command = MoveRightCommand::new();
        let key_event = create_key_event(KeyCode::Right, KeyModifiers::NONE);
        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        assert!(command.is_relevant(key_event, EditorMode::Normal, &context));
    }

    #[test]
    fn move_right_should_allow_right_arrow_with_alt_modifier() {
        let command = MoveRightCommand::new();
        let key_event = create_key_event(KeyCode::Right, KeyModifiers::ALT);
        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        assert!(command.is_relevant(key_event, EditorMode::Normal, &context));
    }

    #[test]
    fn move_right_should_not_be_relevant_for_right_arrow_with_shift_or_control() {
        let command = MoveRightCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        // Test Shift+Right (used for selection/scrolling)
        let shift_right = create_key_event(KeyCode::Right, KeyModifiers::SHIFT);
        assert!(!command.is_relevant(shift_right, EditorMode::Normal, &context));

        // Test Ctrl+Right (used for word navigation/scrolling)
        let ctrl_right = create_key_event(KeyCode::Right, KeyModifiers::CONTROL);
        assert!(!command.is_relevant(ctrl_right, EditorMode::Normal, &context));
    }

    #[test]
    fn move_right_l_key_should_not_be_relevant_in_non_movement_modes() {
        let command = MoveRightCommand::new();
        let l_key_event = create_key_event(KeyCode::Char('l'), KeyModifiers::NONE);
        let context = CommandContext {
            current_mode: EditorMode::Insert,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        // 'l' key should not work in non-movement modes (Vim behavior)
        assert!(!command.is_relevant(l_key_event, EditorMode::Insert, &context));
        assert!(!command.is_relevant(l_key_event, EditorMode::VisualBlockInsert, &context));
        assert!(!command.is_relevant(l_key_event, EditorMode::Command, &context));
    }

    #[test]
    fn move_right_arrow_key_should_work_in_all_modes() {
        let command = MoveRightCommand::new();
        let arrow_key_event = create_key_event(KeyCode::Right, KeyModifiers::NONE);
        let context = CommandContext {
            current_mode: EditorMode::Insert,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        // Right arrow should work in all modes (standard editor behavior)
        assert!(command.is_relevant(arrow_key_event, EditorMode::Insert, &context));
        assert!(command.is_relevant(arrow_key_event, EditorMode::Normal, &context));
        assert!(command.is_relevant(arrow_key_event, EditorMode::Visual, &context));
        assert!(command.is_relevant(arrow_key_event, EditorMode::VisualBlockInsert, &context));
        assert!(command.is_relevant(arrow_key_event, EditorMode::Command, &context));
    }

    #[test]
    fn move_right_should_not_be_relevant_for_other_keys() {
        let command = MoveRightCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        // Test other movement keys
        assert!(!command.is_relevant(
            create_key_event(KeyCode::Char('h'), KeyModifiers::NONE),
            EditorMode::Normal,
            &context
        ));
        assert!(!command.is_relevant(
            create_key_event(KeyCode::Char('j'), KeyModifiers::NONE),
            EditorMode::Normal,
            &context
        ));
        assert!(!command.is_relevant(
            create_key_event(KeyCode::Char('k'), KeyModifiers::NONE),
            EditorMode::Normal,
            &context
        ));

        // Test non-movement keys
        assert!(!command.is_relevant(
            create_key_event(KeyCode::Char('x'), KeyModifiers::NONE),
            EditorMode::Normal,
            &context
        ));
        assert!(!command.is_relevant(
            create_key_event(KeyCode::Enter, KeyModifiers::NONE),
            EditorMode::Normal,
            &context
        ));
    }

    #[test]
    fn move_right_should_not_be_relevant_with_l_key_modifiers() {
        let command = MoveRightCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        // Test 'l' with various modifiers - should not be relevant
        assert!(!command.is_relevant(
            create_key_event(KeyCode::Char('l'), KeyModifiers::CONTROL),
            EditorMode::Normal,
            &context
        ));
        assert!(!command.is_relevant(
            create_key_event(KeyCode::Char('l'), KeyModifiers::SHIFT),
            EditorMode::Normal,
            &context
        ));
        assert!(!command.is_relevant(
            create_key_event(KeyCode::Char('l'), KeyModifiers::ALT),
            EditorMode::Normal,
            &context
        ));
    }

    #[test]
    fn move_right_should_work_in_read_only_panes() {
        let command = MoveRightCommand::new();
        let key_event = create_key_event(KeyCode::Char('l'), KeyModifiers::NONE);
        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Response,
            is_read_only: true,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        // Movement should work in read-only panes (unlike editing commands)
        assert!(command.is_relevant(key_event, EditorMode::Normal, &context));
    }

    #[test]
    fn move_right_execute_should_handle_cursor_movement() {
        let command = MoveRightCommand::new();
        let (mut app_state, mut services) = create_execution_context();

        // Ensure we're in Normal mode
        app_state.change_mode(EditorMode::Normal).unwrap();

        let mut context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        let result = command.execute(&mut context);

        // Should succeed (AppState handles the actual movement logic)
        assert!(result.is_ok());
        let events = result.unwrap();
        // AppState internally handles view events, so we expect empty return
        assert_eq!(events.len(), 0);
    }

    #[test]
    fn is_movement_mode_should_identify_correct_modes() {
        // Movement modes
        assert!(MoveRightCommand::is_movement_mode(EditorMode::Normal));
        assert!(MoveRightCommand::is_movement_mode(EditorMode::Visual));
        assert!(MoveRightCommand::is_movement_mode(EditorMode::VisualLine));
        assert!(MoveRightCommand::is_movement_mode(EditorMode::VisualBlock));

        // Non-movement modes
        assert!(!MoveRightCommand::is_movement_mode(EditorMode::Insert));
        assert!(!MoveRightCommand::is_movement_mode(
            EditorMode::VisualBlockInsert
        ));
        assert!(!MoveRightCommand::is_movement_mode(EditorMode::Command));
    }

    #[test]
    fn command_name_should_return_correct_name() {
        let command = MoveRightCommand::new();
        assert_eq!(command.name(), "MoveRightCommand");
    }

    #[test]
    fn default_should_create_new_instance() {
        let command = MoveRightCommand::new();
        assert_eq!(command.name(), "MoveRightCommand");
    }
}

// Auto-register this command using the inventory system
register_command!(MoveRightCommand, "MoveRightCommand");
