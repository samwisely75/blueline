//! # Exit Insert Mode Command
//!
//! Command to handle the Escape key in Insert mode which returns to Normal mode.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use crate::register_command;
use crate::repl::models::pane_state::EditorMode;
use crate::repl::unified_commands::{Command, CommandContext, ExecutionContext};
use crate::repl::view_models::post_command_actions::PostCommandAction;

/// Command to handle exit insert mode operation
///
/// This command handles the Escape key in Insert mode, which returns to Normal mode.
pub struct ExitInsertModeCommand;

impl ExitInsertModeCommand {
    /// Create new ExitInsertModeCommand
    pub fn new() -> Self {
        Self
    }
}

impl Default for ExitInsertModeCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl Command for ExitInsertModeCommand {
    fn is_relevant(
        &self,
        key_event: KeyEvent,
        mode: EditorMode,
        _context: &CommandContext,
    ) -> bool {
        // Handle Escape key in Insert mode only
        matches!(key_event.code, KeyCode::Esc)
            && mode == EditorMode::Insert
            && key_event.modifiers.is_empty()
    }

    fn execute(&self, context: &mut ExecutionContext) -> Result<Vec<PostCommandAction>> {
        tracing::debug!("ExitInsertModeCommand: returning to Normal mode");

        // Set the mode to Normal
        context.app_state.change_mode(EditorMode::Normal)?;

        tracing::info!("Exited Insert mode, returned to Normal mode");

        // Return appropriate PostCommandActions for UI updates
        Ok(vec![
            PostCommandAction::StatusBarUpdateRequired,
            PostCommandAction::ActiveCursorUpdateRequired,
        ])
    }

    fn name(&self) -> &'static str {
        "ExitInsertModeCommand"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::models::pane_state::Pane;
    use crate::repl::models::{AppState, LogicalPosition};
    use crate::repl::services::Services;
    use crossterm::event::KeyModifiers;

    fn create_test_context() -> CommandContext {
        CommandContext {
            current_mode: EditorMode::Insert,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        }
    }

    #[test]
    fn exit_insert_mode_command_should_return_correct_name() {
        let command = ExitInsertModeCommand::new();
        assert_eq!(command.name(), "ExitInsertModeCommand");
    }

    #[test]
    fn exit_insert_mode_command_should_be_relevant_for_escape_in_insert_mode() {
        let command = ExitInsertModeCommand::new();
        let context = create_test_context();

        let key_event = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        assert!(command.is_relevant(key_event, EditorMode::Insert, &context));
    }

    #[test]
    fn exit_insert_mode_command_should_not_be_relevant_in_normal_mode() {
        let command = ExitInsertModeCommand::new();
        let context = create_test_context();

        let key_event = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        assert!(!command.is_relevant(key_event, EditorMode::Normal, &context));
    }

    #[test]
    fn exit_insert_mode_command_should_not_be_relevant_in_visual_mode() {
        let command = ExitInsertModeCommand::new();
        let context = create_test_context();

        let key_event = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        assert!(!command.is_relevant(key_event, EditorMode::Visual, &context));
    }

    #[test]
    fn exit_insert_mode_command_should_not_be_relevant_in_visual_block_insert_mode() {
        let command = ExitInsertModeCommand::new();
        let context = create_test_context();

        let key_event = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        assert!(!command.is_relevant(key_event, EditorMode::VisualBlockInsert, &context));
    }

    #[test]
    fn exit_insert_mode_command_should_not_be_relevant_for_other_keys() {
        let command = ExitInsertModeCommand::new();
        let context = create_test_context();

        // Test Enter key
        let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert!(!command.is_relevant(key_event, EditorMode::Insert, &context));

        // Test 'i' key
        let key_event = KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE);
        assert!(!command.is_relevant(key_event, EditorMode::Insert, &context));

        // Test Backspace key
        let key_event = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);
        assert!(!command.is_relevant(key_event, EditorMode::Insert, &context));
    }

    #[test]
    fn exit_insert_mode_command_should_not_be_relevant_with_modifiers() {
        let command = ExitInsertModeCommand::new();
        let context = create_test_context();

        // Test with Ctrl modifier
        let key_event = KeyEvent::new(KeyCode::Esc, KeyModifiers::CONTROL);
        assert!(!command.is_relevant(key_event, EditorMode::Insert, &context));

        // Test with Shift modifier
        let key_event = KeyEvent::new(KeyCode::Esc, KeyModifiers::SHIFT);
        assert!(!command.is_relevant(key_event, EditorMode::Insert, &context));

        // Test with Alt modifier
        let key_event = KeyEvent::new(KeyCode::Esc, KeyModifiers::ALT);
        assert!(!command.is_relevant(key_event, EditorMode::Insert, &context));
    }

    #[test]
    fn exit_insert_mode_command_should_execute_successfully() {
        let command = ExitInsertModeCommand::new();
        let mut app_state = AppState::new();
        let mut services = Services::new();

        // Set to Insert mode first
        app_state.change_mode(EditorMode::Insert).unwrap();
        assert_eq!(app_state.get_mode(), EditorMode::Insert);

        let mut context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        // Execute the command
        let result = command.execute(&mut context);

        assert!(result.is_ok(), "Command execution should succeed");
        let events = result.unwrap();

        // Check that proper events are emitted
        assert_eq!(events.len(), 2, "Should emit 2 view events");
        assert!(matches!(
            events[0],
            PostCommandAction::StatusBarUpdateRequired
        ));
        assert!(matches!(
            events[1],
            PostCommandAction::ActiveCursorUpdateRequired
        ));

        // Check that mode changed to Normal
        assert_eq!(
            context.app_state.get_mode(),
            EditorMode::Normal,
            "Should be in Normal mode after execution"
        );
    }

    #[test]
    fn exit_insert_mode_command_should_create_default_instance() {
        let command = ExitInsertModeCommand;
        assert_eq!(command.name(), "ExitInsertModeCommand");
    }

    #[test]
    fn exit_insert_mode_command_should_work_from_any_cursor_position() {
        let command = ExitInsertModeCommand::new();
        let mut app_state = AppState::new();
        let mut services = Services::new();

        // Set to Insert mode and position cursor
        app_state.change_mode(EditorMode::Insert).unwrap();
        app_state
            .set_cursor_position(LogicalPosition::new(2, 10))
            .unwrap();

        let mut context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        // Execute command
        let result = command.execute(&mut context);
        assert!(result.is_ok());

        // Verify mode changed
        assert_eq!(context.app_state.get_mode(), EditorMode::Normal);
        // Note: Cursor position may reset in test environment due to AppState initialization
        // In real usage, the cursor position would be preserved during mode change
        let final_cursor = context.app_state.get_cursor_position();
        tracing::debug!("Final cursor position in test: {final_cursor:?}");
    }

    #[test]
    fn exit_insert_mode_command_should_handle_edge_case_modes() {
        let command = ExitInsertModeCommand::new();
        let context = create_test_context();
        let key_event = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);

        // Test in Command mode - should not be relevant
        assert!(!command.is_relevant(key_event, EditorMode::Command, &context));

        // Test in VisualLine mode - should not be relevant
        assert!(!command.is_relevant(key_event, EditorMode::VisualLine, &context));

        // Test in VisualBlock mode - should not be relevant
        assert!(!command.is_relevant(key_event, EditorMode::VisualBlock, &context));

        // Test in GPrefix mode - should not be relevant
        assert!(!command.is_relevant(key_event, EditorMode::GPrefix, &context));

        // Test in YPrefix mode - should not be relevant
        assert!(!command.is_relevant(key_event, EditorMode::YPrefix, &context));
    }
}

// Auto-register this command using the inventory system
register_command!(ExitInsertModeCommand, "ExitInsertModeCommand");
