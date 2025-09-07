//! # Delete Character at Cursor Command
//!
//! This command handles character deletion at the cursor position (Delete key) in Insert
//! and VisualBlockInsert modes. It processes the Delete key and deletes the character
//! AT/AFTER the cursor position, which is forward delete behavior.
//!
//! This is different from DeleteCharCommand which handles Backspace (deletes before cursor).

use anyhow::Result;
use crossterm::event::KeyEvent;

use crate::register_command;
use crate::repl::models::pane_state::{EditorMode, Pane};
use crate::repl::view_models::commands::{Command, CommandContext, ExecutionContext};
use crate::repl::view_models::post_command_actions::PostCommandAction;
use crossterm::event::KeyCode;

/// Command to handle character deletion at cursor position (Delete key)
///
/// This command handles the deletion of characters when the Delete key is pressed
/// in Insert or VisualBlockInsert mode. It deletes the character at/after the cursor
/// position, which is the standard forward delete behavior expected by users.
/// This is the counterpart to DeleteCharCommand which handles Backspace.
pub struct DeleteCharAtCursorCommand;

impl DeleteCharAtCursorCommand {
    /// Create new DeleteCharAtCursorCommand
    pub fn new() -> Self {
        Self
    }
}

impl Default for DeleteCharAtCursorCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl Command for DeleteCharAtCursorCommand {
    fn is_relevant(&self, key_event: KeyEvent, mode: EditorMode, context: &CommandContext) -> bool {
        // Handle Delete key in Insert mode
        // Must be in Insert or VisualBlockInsert mode, in Request pane, not read-only
        let correct_mode = matches!(mode, EditorMode::Insert | EditorMode::VisualBlockInsert);
        let correct_pane = context.current_pane == Pane::Request;
        let not_read_only = !context.is_read_only;

        // Check if it's the Delete key without any modifiers
        let is_delete = matches!(key_event.code, KeyCode::Delete);
        let no_modifiers = key_event.modifiers.is_empty();

        correct_mode && correct_pane && not_read_only && is_delete && no_modifiers
    }

    fn execute(
        &self,
        _key_event: KeyEvent,
        context: &mut ExecutionContext,
    ) -> Result<Vec<PostCommandAction>> {
        tracing::debug!("DeleteCharAtCursorCommand: Deleting character at cursor position");

        // Delete character at/after cursor (forward delete behavior)
        context.app_state.delete_char_after_cursor()?;

        // Return appropriate post-command actions for UI updates
        Ok(vec![
            PostCommandAction::CurrentAreaRedrawRequired,
            PostCommandAction::ActiveCursorUpdateRequired,
        ])
    }

    fn name(&self) -> &'static str {
        "DeleteCharAtCursorCommand"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::models::pane_state::{EditorMode, Pane};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

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
    fn delete_char_at_cursor_command_should_return_correct_name() {
        let command = DeleteCharAtCursorCommand::new();
        assert_eq!(command.name(), "DeleteCharAtCursorCommand");
    }

    #[test]
    fn delete_char_at_cursor_command_should_be_relevant_for_delete_in_insert_mode() {
        let command = DeleteCharAtCursorCommand::new();
        let context = create_test_context();

        let key_event = KeyEvent::new(KeyCode::Delete, KeyModifiers::NONE);
        assert!(command.is_relevant(key_event, EditorMode::Insert, &context));
    }

    #[test]
    fn delete_char_at_cursor_command_should_be_relevant_in_visual_block_insert_mode() {
        let command = DeleteCharAtCursorCommand::new();
        let context = create_test_context();

        let key_event = KeyEvent::new(KeyCode::Delete, KeyModifiers::NONE);
        assert!(command.is_relevant(key_event, EditorMode::VisualBlockInsert, &context));
    }

    #[test]
    fn delete_char_at_cursor_command_should_not_be_relevant_in_normal_mode() {
        let command = DeleteCharAtCursorCommand::new();
        let context = create_test_context();

        let key_event = KeyEvent::new(KeyCode::Delete, KeyModifiers::NONE);
        assert!(!command.is_relevant(key_event, EditorMode::Normal, &context));
    }

    #[test]
    fn delete_char_at_cursor_command_should_not_be_relevant_in_visual_mode() {
        let command = DeleteCharAtCursorCommand::new();
        let context = create_test_context();

        let key_event = KeyEvent::new(KeyCode::Delete, KeyModifiers::NONE);
        assert!(!command.is_relevant(key_event, EditorMode::Visual, &context));
    }

    #[test]
    fn delete_char_at_cursor_command_should_not_be_relevant_with_modifiers() {
        let command = DeleteCharAtCursorCommand::new();
        let context = create_test_context();

        // Ctrl+Delete
        let key_event = KeyEvent::new(KeyCode::Delete, KeyModifiers::CONTROL);
        assert!(!command.is_relevant(key_event, EditorMode::Insert, &context));

        // Shift+Delete
        let key_event = KeyEvent::new(KeyCode::Delete, KeyModifiers::SHIFT);
        assert!(!command.is_relevant(key_event, EditorMode::Insert, &context));

        // Alt+Delete
        let key_event = KeyEvent::new(KeyCode::Delete, KeyModifiers::ALT);
        assert!(!command.is_relevant(key_event, EditorMode::Insert, &context));
    }

    #[test]
    fn delete_char_at_cursor_command_should_not_be_relevant_for_non_delete_keys() {
        let command = DeleteCharAtCursorCommand::new();
        let context = create_test_context();

        // Backspace key (different from delete)
        let key_event = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);
        assert!(!command.is_relevant(key_event, EditorMode::Insert, &context));

        // Regular character
        let key_event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        assert!(!command.is_relevant(key_event, EditorMode::Insert, &context));

        // Enter key
        let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert!(!command.is_relevant(key_event, EditorMode::Insert, &context));
    }

    #[test]
    fn delete_char_at_cursor_command_should_not_be_relevant_in_response_pane() {
        let command = DeleteCharAtCursorCommand::new();
        let mut context = create_test_context();
        context.current_pane = Pane::Response;

        let key_event = KeyEvent::new(KeyCode::Delete, KeyModifiers::NONE);
        assert!(!command.is_relevant(key_event, EditorMode::Insert, &context));
    }

    #[test]
    fn delete_char_at_cursor_command_should_not_be_relevant_when_read_only() {
        let command = DeleteCharAtCursorCommand::new();
        let mut context = create_test_context();
        context.is_read_only = true;

        let key_event = KeyEvent::new(KeyCode::Delete, KeyModifiers::NONE);
        assert!(!command.is_relevant(key_event, EditorMode::Insert, &context));
    }

    #[test]
    fn delete_char_at_cursor_command_should_create_default_instance() {
        let command = DeleteCharAtCursorCommand;
        assert_eq!(command.name(), "DeleteCharAtCursorCommand");
    }

    #[test]
    fn delete_char_at_cursor_command_execute_should_delete_character() {
        let command = DeleteCharAtCursorCommand::new();
        let mut app_state = crate::repl::models::AppState::new();
        let mut services = crate::repl::services::Services::new();

        // Set to Insert mode and add some text first
        app_state.change_mode(EditorMode::Insert).unwrap();
        app_state.insert_char('a').unwrap();
        app_state.insert_char('b').unwrap();
        // Move cursor back to be between 'a' and 'b' to test forward delete
        app_state.move_cursor_left().unwrap();

        let mut exec_context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        // Create a key event for Delete
        let key_event = KeyEvent::new(KeyCode::Delete, KeyModifiers::NONE);

        // Execute the command
        let result = command.execute(key_event, &mut exec_context);

        assert!(result.is_ok());
        let events = result.unwrap();

        // Should return UI update events
        assert_eq!(events.len(), 2);
        assert!(events
            .iter()
            .any(|e| matches!(e, PostCommandAction::CurrentAreaRedrawRequired)));
        assert!(events
            .iter()
            .any(|e| matches!(e, PostCommandAction::ActiveCursorUpdateRequired)));
    }
}

// Auto-register this command using the inventory system
register_command!(DeleteCharAtCursorCommand, "DeleteCharAtCursorCommand");
