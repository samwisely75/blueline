//! # Scroll Right Command
//!
//! Command to scroll the current pane horizontally to the right.
//! This handles Shift+Right and Ctrl+Right key combinations for horizontal scrolling.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::register_command;
use crate::repl::models::pane_state::EditorMode;
use crate::repl::unified_commands::{Command, CommandContext, ExecutionContext};
use crate::repl::view_models::post_command_actions::PostCommandAction;

/// Command to scroll the current pane right by a fixed amount
///
/// This command:
/// 1. Handles Right arrow key with Shift or Control modifiers
/// 2. Scrolls the current pane horizontally to the right by 5 columns
/// 3. Uses AppState's scroll_current_horizontally() method for business logic
/// 4. Works in all editor modes
/// 5. Emits viewport update events for UI refresh
pub struct ScrollRightCommand;

impl ScrollRightCommand {
    /// Create new ScrollRightCommand
    pub fn new() -> Self {
        Self
    }

    /// Amount to scroll horizontally (columns)
    const SCROLL_AMOUNT: usize = 5;
}

impl Command for ScrollRightCommand {
    fn is_relevant(
        &self,
        key_event: KeyEvent,
        _mode: EditorMode,
        _context: &CommandContext,
    ) -> bool {
        // Only handle Right arrow with Shift or Control modifiers
        matches!(key_event.code, KeyCode::Right)
            && (key_event.modifiers.contains(KeyModifiers::SHIFT)
                || key_event.modifiers.contains(KeyModifiers::CONTROL))
    }

    fn execute(&self, context: &mut ExecutionContext) -> Result<Vec<PostCommandAction>> {
        // Use PaneManager's scroll_current_horizontally business logic
        // Direction: 1 (right), Amount: 5 columns
        let events = context
            .app_state
            .pane_manager
            .scroll_current_horizontally(1, Self::SCROLL_AMOUNT);

        tracing::debug!(
            "ScrollRightCommand executed, scrolled right by {} columns, generated {} events",
            Self::SCROLL_AMOUNT,
            events.len()
        );

        Ok(events)
    }

    fn name(&self) -> &'static str {
        "ScrollRightCommand"
    }
}

impl Default for ScrollRightCommand {
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
    fn scroll_right_should_be_relevant_for_shift_right() {
        let command = ScrollRightCommand::new();
        let key_event = create_key_event(KeyCode::Right, KeyModifiers::SHIFT);
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
    fn scroll_right_should_be_relevant_for_ctrl_right() {
        let command = ScrollRightCommand::new();
        let key_event = create_key_event(KeyCode::Right, KeyModifiers::CONTROL);
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
    fn scroll_right_should_work_in_all_modes() {
        let command = ScrollRightCommand::new();
        let key_event = create_key_event(KeyCode::Right, KeyModifiers::SHIFT);
        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        // Should work in all editor modes
        assert!(command.is_relevant(key_event, EditorMode::Normal, &context));
        assert!(command.is_relevant(key_event, EditorMode::Insert, &context));
        assert!(command.is_relevant(key_event, EditorMode::Visual, &context));
        assert!(command.is_relevant(key_event, EditorMode::VisualLine, &context));
        assert!(command.is_relevant(key_event, EditorMode::VisualBlock, &context));
        assert!(command.is_relevant(key_event, EditorMode::VisualBlockInsert, &context));
        assert!(command.is_relevant(key_event, EditorMode::Command, &context));
    }

    #[test]
    fn scroll_right_should_not_be_relevant_for_plain_right_arrow() {
        let command = ScrollRightCommand::new();
        let key_event = create_key_event(KeyCode::Right, KeyModifiers::NONE);
        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        // Plain Right arrow should not trigger scroll (handled by MoveRightCommand)
        assert!(!command.is_relevant(key_event, EditorMode::Normal, &context));
    }

    #[test]
    fn scroll_right_should_not_be_relevant_for_alt_right() {
        let command = ScrollRightCommand::new();
        let key_event = create_key_event(KeyCode::Right, KeyModifiers::ALT);
        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        // Alt+Right might be used for other purposes
        assert!(!command.is_relevant(key_event, EditorMode::Normal, &context));
    }

    #[test]
    fn scroll_right_should_not_be_relevant_for_other_keys() {
        let command = ScrollRightCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        // Test other keys with same modifiers
        assert!(!command.is_relevant(
            create_key_event(KeyCode::Left, KeyModifiers::SHIFT),
            EditorMode::Normal,
            &context
        ));
        assert!(!command.is_relevant(
            create_key_event(KeyCode::Up, KeyModifiers::CONTROL),
            EditorMode::Normal,
            &context
        ));
        assert!(!command.is_relevant(
            create_key_event(KeyCode::Down, KeyModifiers::SHIFT),
            EditorMode::Normal,
            &context
        ));

        // Test character keys with modifiers
        assert!(!command.is_relevant(
            create_key_event(KeyCode::Char('l'), KeyModifiers::SHIFT),
            EditorMode::Normal,
            &context
        ));
        assert!(!command.is_relevant(
            create_key_event(KeyCode::Char('h'), KeyModifiers::CONTROL),
            EditorMode::Normal,
            &context
        ));
    }

    #[test]
    fn scroll_right_should_work_with_combined_modifiers() {
        let command = ScrollRightCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        // Test with both Shift and Control modifiers
        let combined_modifiers = KeyModifiers::SHIFT | KeyModifiers::CONTROL;
        let key_event = create_key_event(KeyCode::Right, combined_modifiers);
        assert!(command.is_relevant(key_event, EditorMode::Normal, &context));
    }

    #[test]
    fn scroll_right_should_work_in_read_only_panes() {
        let command = ScrollRightCommand::new();
        let key_event = create_key_event(KeyCode::Right, KeyModifiers::SHIFT);
        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Response,
            is_read_only: true,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        // Scrolling should work in read-only panes
        assert!(command.is_relevant(key_event, EditorMode::Normal, &context));
    }

    #[test]
    fn scroll_right_execute_should_handle_horizontal_scrolling() {
        let command = ScrollRightCommand::new();
        let (mut app_state, mut services) = create_execution_context();

        // Ensure we're in Normal mode
        app_state.change_mode(EditorMode::Normal).unwrap();

        let mut context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        let result = command.execute(&mut context);

        // Should succeed
        assert!(result.is_ok());
        let events = result.unwrap();

        // Should generate at least one event for scroll update
        // The exact number depends on PaneManager implementation
        assert!(!events.is_empty());
    }

    #[test]
    fn scroll_right_constant_should_match_legacy_behavior() {
        // Legacy ScrollRightCommand used amount=5 in CommandEvent::cursor_move_by
        assert_eq!(ScrollRightCommand::SCROLL_AMOUNT, 5);
    }

    #[test]
    fn command_name_should_return_correct_name() {
        let command = ScrollRightCommand::new();
        assert_eq!(command.name(), "ScrollRightCommand");
    }

    #[test]
    fn default_should_create_new_instance() {
        let command = ScrollRightCommand;
        assert_eq!(command.name(), "ScrollRightCommand");
    }
}

// Auto-register this command using the inventory system
register_command!(ScrollRightCommand, "ScrollRightCommand");
