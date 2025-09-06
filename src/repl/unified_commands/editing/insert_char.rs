//! # Insert Character Command
//!
//! This command handles character insertion in Insert and VisualBlockInsert modes.
//! It processes printable characters typed by the user and inserts them at the
//! current cursor position.

use anyhow::Result;
use crossterm::event::KeyEvent;

use crate::register_command;
use crate::repl::models::pane_state::EditorMode;
use crate::repl::unified_commands::{Command, CommandContext, ExecutionContext};
use crate::repl::view_models::post_command_actions::PostCommandAction;

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
        // TEMPORARY: Return false to let legacy system handle this
        // The unified command system has an architectural limitation:
        // execute() doesn't receive KeyEvent, so we can't access the character
        // Until this is fixed, we must use the legacy system
        _ = key_event;
        _ = mode;
        _ = context;
        false
    }

    fn execute(&self, _context: &mut ExecutionContext) -> Result<Vec<PostCommandAction>> {
        // In the unified command system, we can't access the KeyEvent directly
        // This is an architectural limitation that needs to be addressed
        // For now, we return empty as the legacy system will handle it

        // TODO: The unified command system needs to be updated to pass KeyEvent
        // to the execute method so we can access the character to insert

        tracing::warn!(
            "InsertCharCommand: Cannot access character from KeyEvent in unified command system"
        );

        // Return empty - the legacy system will handle this for now
        Ok(vec![])
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

    // Note: execute() test is limited due to architectural constraints
    // The unified command system doesn't pass KeyEvent to execute()
    // so we can't fully test character insertion without refactoring
}

// Auto-register this command using the inventory system
register_command!(InsertCharCommand, "InsertCharCommand");
