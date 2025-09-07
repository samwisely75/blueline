//! # Delete Character Command
//!
//! This command handles character deletion (backspace) in Insert and VisualBlockInsert modes.
//! It processes the Backspace key and deletes the character before the cursor position.
//! This is the primary method for deleting text while typing in insert modes.

use anyhow::Result;
use crossterm::event::KeyEvent;

use crate::register_command;
use crate::repl::models::pane_state::{EditorMode, Pane};
use crate::repl::view_models::commands::{Command, CommandContext, ExecutionContext};
use crate::repl::view_models::post_command_actions::PostCommandAction;
use crossterm::event::KeyCode;

/// Command to handle character deletion (backspace)
///
/// This command handles the deletion of characters when the Backspace key is pressed
/// in Insert or VisualBlockInsert mode. It deletes the character before the cursor
/// position, which is the standard backspace behavior expected by users.
pub struct DeleteCharCommand;

impl DeleteCharCommand {
    /// Create new DeleteCharCommand
    pub fn new() -> Self {
        Self
    }
}

impl Default for DeleteCharCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl Command for DeleteCharCommand {
    fn is_relevant(&self, key_event: KeyEvent, mode: EditorMode, context: &CommandContext) -> bool {
        // Handle backspace key in Insert mode
        // Must be in Insert or VisualBlockInsert mode, in Request pane, not read-only
        let correct_mode = matches!(mode, EditorMode::Insert | EditorMode::VisualBlockInsert);
        let correct_pane = context.current_pane == Pane::Request;
        let not_read_only = !context.is_read_only;

        // Check if it's the Backspace key without any modifiers
        let is_backspace = matches!(key_event.code, KeyCode::Backspace);
        let no_modifiers = key_event.modifiers.is_empty();

        correct_mode && correct_pane && not_read_only && is_backspace && no_modifiers
    }

    fn execute(
        &self,
        _key_event: KeyEvent,
        context: &mut ExecutionContext,
    ) -> Result<Vec<PostCommandAction>> {
        tracing::debug!("DeleteCharCommand: Deleting character before cursor");

        // Delete character before cursor (backspace behavior)
        context.app_state.delete_char_before_cursor()?;

        // Return appropriate post-command actions for UI updates
        Ok(vec![
            PostCommandAction::CurrentAreaRedrawRequired,
            PostCommandAction::ActiveCursorUpdateRequired,
        ])
    }

    fn name(&self) -> &'static str {
        "DeleteCharCommand"
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
    fn delete_char_command_should_return_correct_name() {
        let command = DeleteCharCommand::new();
        assert_eq!(command.name(), "DeleteCharCommand");
    }

    #[test]
    fn delete_char_command_should_be_relevant_for_backspace_in_insert_mode() {
        let command = DeleteCharCommand::new();
        let context = create_test_context();

        let key_event = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);
        assert!(command.is_relevant(key_event, EditorMode::Insert, &context));
    }

    #[test]
    fn delete_char_command_should_be_relevant_in_visual_block_insert_mode() {
        let command = DeleteCharCommand::new();
        let context = create_test_context();

        let key_event = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);
        assert!(command.is_relevant(key_event, EditorMode::VisualBlockInsert, &context));
    }

    #[test]
    fn delete_char_command_should_not_be_relevant_in_normal_mode() {
        let command = DeleteCharCommand::new();
        let context = create_test_context();

        let key_event = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);
        assert!(!command.is_relevant(key_event, EditorMode::Normal, &context));
    }

    #[test]
    fn delete_char_command_should_not_be_relevant_in_visual_mode() {
        let command = DeleteCharCommand::new();
        let context = create_test_context();

        let key_event = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);
        assert!(!command.is_relevant(key_event, EditorMode::Visual, &context));
    }

    #[test]
    fn delete_char_command_should_not_be_relevant_with_modifiers() {
        let command = DeleteCharCommand::new();
        let context = create_test_context();

        // Ctrl+Backspace
        let key_event = KeyEvent::new(KeyCode::Backspace, KeyModifiers::CONTROL);
        assert!(!command.is_relevant(key_event, EditorMode::Insert, &context));

        // Shift+Backspace
        let key_event = KeyEvent::new(KeyCode::Backspace, KeyModifiers::SHIFT);
        assert!(!command.is_relevant(key_event, EditorMode::Insert, &context));

        // Alt+Backspace
        let key_event = KeyEvent::new(KeyCode::Backspace, KeyModifiers::ALT);
        assert!(!command.is_relevant(key_event, EditorMode::Insert, &context));
    }

    #[test]
    fn delete_char_command_should_not_be_relevant_for_non_backspace_keys() {
        let command = DeleteCharCommand::new();
        let context = create_test_context();

        // Delete key (different from backspace)
        let key_event = KeyEvent::new(KeyCode::Delete, KeyModifiers::NONE);
        assert!(!command.is_relevant(key_event, EditorMode::Insert, &context));

        // Regular character
        let key_event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        assert!(!command.is_relevant(key_event, EditorMode::Insert, &context));

        // Enter key
        let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert!(!command.is_relevant(key_event, EditorMode::Insert, &context));
    }

    #[test]
    fn delete_char_command_should_not_be_relevant_in_response_pane() {
        let command = DeleteCharCommand::new();
        let mut context = create_test_context();
        context.current_pane = Pane::Response;

        let key_event = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);
        assert!(!command.is_relevant(key_event, EditorMode::Insert, &context));
    }

    #[test]
    fn delete_char_command_should_not_be_relevant_when_read_only() {
        let command = DeleteCharCommand::new();
        let mut context = create_test_context();
        context.is_read_only = true;

        let key_event = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);
        assert!(!command.is_relevant(key_event, EditorMode::Insert, &context));
    }

    #[test]
    fn delete_char_command_should_create_default_instance() {
        let command = DeleteCharCommand;
        assert_eq!(command.name(), "DeleteCharCommand");
    }

    #[test]
    fn delete_char_command_execute_should_delete_character() {
        let command = DeleteCharCommand::new();
        let mut app_state = crate::repl::models::AppState::new();
        let mut services = crate::repl::services::Services::new();

        // Set to Insert mode and add some text first
        app_state.change_mode(EditorMode::Insert).unwrap();
        app_state.insert_char('a').unwrap();
        app_state.insert_char('b').unwrap();

        let mut exec_context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        // Create a key event for Backspace
        let key_event = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);

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
register_command!(DeleteCharCommand, "DeleteCharCommand");
