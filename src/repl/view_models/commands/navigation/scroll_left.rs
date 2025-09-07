//! # Scroll Left Command
//!
//! Command to scroll viewport left horizontally.
//! Handles Shift+Left and Ctrl+Left key combinations.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::register_command;
use crate::repl::models::pane_state::EditorMode;
use crate::repl::view_models::commands::{Command, CommandContext, ExecutionContext};
use crate::repl::view_models::post_command_actions::PostCommandAction;

/// Command to scroll viewport left horizontally
///
/// This command:
/// 1. Handles Shift+Left and Ctrl+Left key combinations
/// 2. Scrolls the viewport left by 5 characters at a time
/// 3. Uses AppState's scroll_current_horizontally() method for business logic
/// 4. Works in all editor modes
/// 5. Emits CurrentAreaScrollChanged and ActiveCursorUpdateRequired events
pub struct ScrollLeftCommand;

impl ScrollLeftCommand {
    /// Create new ScrollLeftCommand
    pub fn new() -> Self {
        Self
    }
}

impl Command for ScrollLeftCommand {
    fn is_relevant(
        &self,
        key_event: KeyEvent,
        _mode: EditorMode,
        _context: &CommandContext,
    ) -> bool {
        // Handle Shift+Left or Ctrl+Left key combinations
        matches!(key_event.code, KeyCode::Left)
            && (key_event.modifiers.contains(KeyModifiers::SHIFT)
                || key_event.modifiers.contains(KeyModifiers::CONTROL))
    }

    fn execute(
        &self,
        _key_event: KeyEvent,
        context: &mut ExecutionContext,
    ) -> Result<Vec<PostCommandAction>> {
        // Scroll left horizontally by 5 characters (direction = -1, amount = 5)
        let actions = context
            .app_state
            .pane_manager
            .scroll_current_horizontally(-1, 5);
        Ok(actions)
    }

    fn name(&self) -> &'static str {
        "ScrollLeft"
    }
}

impl Default for ScrollLeftCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::models::app_state::AppState;
    use crate::repl::models::pane_state::{EditorMode, Pane};
    use crate::repl::services::Services;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn create_test_context() -> (AppState, Services) {
        let app_state = AppState::new();
        let services = Services::new();
        (app_state, services)
    }

    #[test]
    fn scroll_left_command_is_relevant_for_shift_left() {
        let command = ScrollLeftCommand::new();
        let key_event = KeyEvent::new(KeyCode::Left, KeyModifiers::SHIFT);
        let mode = EditorMode::Normal;
        let context = CommandContext {
            current_mode: mode,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        assert!(command.is_relevant(key_event, mode, &context));
    }

    #[test]
    fn scroll_left_command_is_relevant_for_ctrl_left() {
        let command = ScrollLeftCommand::new();
        let key_event = KeyEvent::new(KeyCode::Left, KeyModifiers::CONTROL);
        let mode = EditorMode::Normal;
        let context = CommandContext {
            current_mode: mode,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        assert!(command.is_relevant(key_event, mode, &context));
    }

    #[test]
    fn scroll_left_command_not_relevant_for_plain_left() {
        let command = ScrollLeftCommand::new();
        let key_event = KeyEvent::new(KeyCode::Left, KeyModifiers::NONE);
        let mode = EditorMode::Normal;
        let context = CommandContext {
            current_mode: mode,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        assert!(!command.is_relevant(key_event, mode, &context));
    }

    #[test]
    fn scroll_left_command_not_relevant_for_other_keys() {
        let command = ScrollLeftCommand::new();
        let key_event = KeyEvent::new(KeyCode::Right, KeyModifiers::SHIFT);
        let mode = EditorMode::Normal;
        let context = CommandContext {
            current_mode: mode,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        assert!(!command.is_relevant(key_event, mode, &context));
    }

    #[test]
    fn scroll_left_command_execute_returns_scroll_actions() {
        let command = ScrollLeftCommand::new();
        let (mut app_state, mut services) = create_test_context();

        let mut execution_context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        let result = command.execute(
            KeyEvent::new(KeyCode::Null, KeyModifiers::empty()),
            &mut execution_context,
        );
        assert!(result.is_ok());

        let actions = result.unwrap();
        // The scroll_current_horizontally method should return at least CurrentAreaScrollChanged
        assert!(!actions.is_empty());

        // Check that we get the expected action type
        assert!(actions
            .iter()
            .any(|action| matches!(action, PostCommandAction::CurrentAreaScrollChanged { .. })));
    }

    #[test]
    fn scroll_left_command_name_is_correct() {
        let command = ScrollLeftCommand::new();
        assert_eq!(command.name(), "ScrollLeft");
    }

    #[test]
    fn scroll_left_command_default_implementation() {
        let command = ScrollLeftCommand;
        assert_eq!(command.name(), "ScrollLeft");
    }

    #[test]
    fn scroll_left_command_is_relevant_in_all_modes() {
        let command = ScrollLeftCommand::new();
        let key_event = KeyEvent::new(KeyCode::Left, KeyModifiers::SHIFT);
        let context = CommandContext {
            current_mode: EditorMode::Normal, // This will be overridden in the loop
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        // Test various modes
        let modes = [
            EditorMode::Normal,
            EditorMode::Insert,
            EditorMode::Visual,
            EditorMode::VisualLine,
            EditorMode::VisualBlock,
            EditorMode::Command,
        ];

        for mode in modes {
            assert!(
                command.is_relevant(key_event, mode, &context),
                "ScrollLeftCommand should be relevant in {mode:?} mode"
            );
        }
    }
}

// Register command for dynamic discovery
register_command!(ScrollLeftCommand, "ScrollLeft");
