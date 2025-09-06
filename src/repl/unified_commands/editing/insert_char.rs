//! # Insert Character Command
//!
//! This command handles character insertion in Insert and VisualBlockInsert modes.
//! It processes printable characters typed by the user and inserts them at the
//! current cursor position.

use anyhow::Result;
use crossterm::event::KeyEvent;

use crate::register_command;
use crate::repl::models::pane_state::{EditorMode, Pane};
use crate::repl::unified_commands::{Command, CommandContext, ExecutionContext};
use crate::repl::view_models::post_command_actions::PostCommandAction;
use crossterm::event::{KeyCode, KeyModifiers};

/// Command to handle character insertion
///
/// This command handles the insertion of printable characters when the editor
/// is in Insert or VisualBlockInsert mode. It supports all printable characters
/// including ASCII, Unicode, and special characters like space.
pub struct InsertCharCommand;

impl InsertCharCommand {
    /// Create new InsertCharCommand
    pub fn new() -> Self {
        Self
    }
}

impl Default for InsertCharCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl Command for InsertCharCommand {
    fn is_relevant(&self, key_event: KeyEvent, mode: EditorMode, context: &CommandContext) -> bool {
        // Handle regular character input in Insert mode
        // Must be in Insert or VisualBlockInsert mode, in Request pane, not read-only
        let correct_mode = matches!(mode, EditorMode::Insert | EditorMode::VisualBlockInsert);
        let correct_pane = context.current_pane == Pane::Request;
        let not_read_only = !context.is_read_only;

        // Check if it's a character key without control/alt modifiers
        // Allow SHIFT for capital letters
        let is_char = matches!(key_event.code, KeyCode::Char(_));
        let no_control = !key_event.modifiers.contains(KeyModifiers::CONTROL);
        let no_alt = !key_event.modifiers.contains(KeyModifiers::ALT);

        correct_mode && correct_pane && not_read_only && is_char && no_control && no_alt
    }

    fn execute(
        &self,
        key_event: KeyEvent,
        context: &mut ExecutionContext,
    ) -> Result<Vec<PostCommandAction>> {
        // Extract the character from the KeyEvent
        if let KeyCode::Char(ch) = key_event.code {
            tracing::debug!("InsertCharCommand: Inserting character '{}'", ch);

            // Insert the character
            context.app_state.insert_char(ch)?;

            // Return appropriate post-command actions for UI updates
            Ok(vec![
                PostCommandAction::CurrentAreaRedrawRequired,
                PostCommandAction::ActiveCursorUpdateRequired,
            ])
        } else {
            // This shouldn't happen if is_relevant works correctly
            tracing::warn!("InsertCharCommand: No character in KeyEvent");
            Ok(vec![])
        }
    }

    fn name(&self) -> &'static str {
        "InsertCharCommand"
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
    fn insert_char_command_should_return_correct_name() {
        let command = InsertCharCommand::new();
        assert_eq!(command.name(), "InsertCharCommand");
    }

    #[test]
    fn insert_char_command_should_be_relevant_for_printable_chars_in_insert_mode() {
        let command = InsertCharCommand::new();
        let context = create_test_context();

        // Test regular ASCII character
        let key_event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        assert!(command.is_relevant(key_event, EditorMode::Insert, &context));

        // Test space character
        let key_event = KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE);
        assert!(command.is_relevant(key_event, EditorMode::Insert, &context));

        // Test number
        let key_event = KeyEvent::new(KeyCode::Char('5'), KeyModifiers::NONE);
        assert!(command.is_relevant(key_event, EditorMode::Insert, &context));
    }

    #[test]
    fn insert_char_command_should_be_relevant_for_unicode_chars() {
        let command = InsertCharCommand::new();
        let context = create_test_context();

        // Test Japanese characters
        let key_event = KeyEvent::new(KeyCode::Char('あ'), KeyModifiers::NONE);
        assert!(command.is_relevant(key_event, EditorMode::Insert, &context));

        let key_event = KeyEvent::new(KeyCode::Char('漢'), KeyModifiers::NONE);
        assert!(command.is_relevant(key_event, EditorMode::Insert, &context));

        // Test emoji
        let key_event = KeyEvent::new(KeyCode::Char('😀'), KeyModifiers::NONE);
        assert!(command.is_relevant(key_event, EditorMode::Insert, &context));
    }

    #[test]
    fn insert_char_command_should_be_relevant_for_capital_letters_with_shift() {
        let command = InsertCharCommand::new();
        let context = create_test_context();

        // Capital letters often come with SHIFT modifier
        let key_event = KeyEvent::new(KeyCode::Char('A'), KeyModifiers::SHIFT);
        assert!(command.is_relevant(key_event, EditorMode::Insert, &context));

        let key_event = KeyEvent::new(KeyCode::Char('Z'), KeyModifiers::SHIFT);
        assert!(command.is_relevant(key_event, EditorMode::Insert, &context));
    }

    #[test]
    fn insert_char_command_should_be_relevant_in_visual_block_insert_mode() {
        let command = InsertCharCommand::new();
        let context = create_test_context();

        let key_event = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
        assert!(command.is_relevant(key_event, EditorMode::VisualBlockInsert, &context));
    }

    #[test]
    fn insert_char_command_should_not_be_relevant_in_normal_mode() {
        let command = InsertCharCommand::new();
        let context = create_test_context();

        let key_event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        assert!(!command.is_relevant(key_event, EditorMode::Normal, &context));
    }

    #[test]
    fn insert_char_command_should_not_be_relevant_in_visual_mode() {
        let command = InsertCharCommand::new();
        let context = create_test_context();

        let key_event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        assert!(!command.is_relevant(key_event, EditorMode::Visual, &context));
    }

    #[test]
    fn insert_char_command_should_not_be_relevant_for_control_chars() {
        let command = InsertCharCommand::new();
        let context = create_test_context();

        // Ctrl+C
        let key_event = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert!(!command.is_relevant(key_event, EditorMode::Insert, &context));

        // Ctrl+A
        let key_event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);
        assert!(!command.is_relevant(key_event, EditorMode::Insert, &context));
    }

    #[test]
    fn insert_char_command_should_not_be_relevant_for_non_char_keys() {
        let command = InsertCharCommand::new();
        let context = create_test_context();

        // Arrow keys
        let key_event = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
        assert!(!command.is_relevant(key_event, EditorMode::Insert, &context));

        // Enter key
        let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert!(!command.is_relevant(key_event, EditorMode::Insert, &context));

        // Tab key
        let key_event = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
        assert!(!command.is_relevant(key_event, EditorMode::Insert, &context));
    }

    #[test]
    fn insert_char_command_should_not_be_relevant_in_response_pane() {
        let command = InsertCharCommand::new();
        let mut context = create_test_context();
        context.current_pane = Pane::Response;

        let key_event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        assert!(!command.is_relevant(key_event, EditorMode::Insert, &context));
    }

    #[test]
    fn insert_char_command_should_handle_special_characters() {
        let command = InsertCharCommand::new();
        let context = create_test_context();

        // Punctuation
        let key_event = KeyEvent::new(KeyCode::Char('!'), KeyModifiers::NONE);
        assert!(command.is_relevant(key_event, EditorMode::Insert, &context));

        let key_event = KeyEvent::new(KeyCode::Char('.'), KeyModifiers::NONE);
        assert!(command.is_relevant(key_event, EditorMode::Insert, &context));

        // Symbols
        let key_event = KeyEvent::new(KeyCode::Char('@'), KeyModifiers::NONE);
        assert!(command.is_relevant(key_event, EditorMode::Insert, &context));

        let key_event = KeyEvent::new(KeyCode::Char('#'), KeyModifiers::NONE);
        assert!(command.is_relevant(key_event, EditorMode::Insert, &context));
    }

    #[test]
    fn insert_char_command_should_create_default_instance() {
        let command = InsertCharCommand;
        assert_eq!(command.name(), "InsertCharCommand");
    }

    #[test]
    fn insert_char_command_execute_should_insert_character() {
        // Now that execute() receives KeyEvent, we can test character insertion
        let command = InsertCharCommand::new();
        let mut app_state = crate::repl::models::AppState::new();
        let mut services = crate::repl::services::Services::new();

        // Set to Insert mode
        app_state.change_mode(EditorMode::Insert).unwrap();

        let mut exec_context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        // Create a key event for character 'a'
        let key_event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);

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
register_command!(InsertCharCommand, "InsertCharCommand");
