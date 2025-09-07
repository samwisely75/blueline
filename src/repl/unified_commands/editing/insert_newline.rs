//! # Insert NewLine Command
//!
//! This command handles newline insertion (Enter key) in Insert and VisualBlockInsert modes.
//! It inserts a newline character (\n) at the current cursor position, effectively
//! creating a new line in the text.

use anyhow::Result;
use crossterm::event::KeyEvent;

use crate::register_command;
use crate::repl::models::pane_state::{EditorMode, Pane};
use crate::repl::unified_commands::{Command, CommandContext, ExecutionContext};
use crate::repl::view_models::post_command_actions::PostCommandAction;
use crossterm::event::{KeyCode, KeyModifiers};

/// Command to handle newline insertion (Enter key)
///
/// This command handles the insertion of newline characters when the editor
/// is in Insert or VisualBlockInsert mode. It responds to the Enter key
/// and inserts a newline character (\n) at the current cursor position.
pub struct InsertNewLineCommand;

impl InsertNewLineCommand {
    /// Create new InsertNewLineCommand
    pub fn new() -> Self {
        Self
    }
}

impl Default for InsertNewLineCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl Command for InsertNewLineCommand {
    fn is_relevant(&self, key_event: KeyEvent, mode: EditorMode, context: &CommandContext) -> bool {
        // Handle Enter key in Insert and VisualBlockInsert modes
        // Must be in Insert or VisualBlockInsert mode, in Request pane, not read-only
        let correct_mode = matches!(mode, EditorMode::Insert | EditorMode::VisualBlockInsert);
        let correct_pane = context.current_pane == Pane::Request;
        let not_read_only = !context.is_read_only;
        let is_enter = matches!(key_event.code, KeyCode::Enter);
        let no_modifiers = key_event.modifiers == KeyModifiers::NONE;

        correct_mode && correct_pane && not_read_only && is_enter && no_modifiers
    }

    fn execute(
        &self,
        _key_event: KeyEvent,
        context: &mut ExecutionContext,
    ) -> Result<Vec<PostCommandAction>> {
        tracing::debug!("InsertNewLineCommand: Inserting newline character");

        // Insert the newline character
        context.app_state.insert_char('\n')?;

        // Return appropriate post-command actions for UI updates
        Ok(vec![
            PostCommandAction::CurrentAreaRedrawRequired,
            PostCommandAction::ActiveCursorUpdateRequired,
        ])
    }

    fn name(&self) -> &'static str {
        "InsertNewLineCommand"
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
    fn insert_newline_command_should_return_correct_name() {
        let command = InsertNewLineCommand::new();
        assert_eq!(command.name(), "InsertNewLineCommand");
    }

    #[test]
    fn insert_newline_command_should_be_relevant_for_enter_in_insert_mode() {
        let command = InsertNewLineCommand::new();
        let context = create_test_context();

        let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert!(command.is_relevant(key_event, EditorMode::Insert, &context));
    }

    #[test]
    fn insert_newline_command_should_be_relevant_in_visual_block_insert_mode() {
        let command = InsertNewLineCommand::new();
        let context = create_test_context();

        let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert!(command.is_relevant(key_event, EditorMode::VisualBlockInsert, &context));
    }

    #[test]
    fn insert_newline_command_should_not_be_relevant_in_normal_mode() {
        let command = InsertNewLineCommand::new();
        let context = create_test_context();

        let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert!(!command.is_relevant(key_event, EditorMode::Normal, &context));
    }

    #[test]
    fn insert_newline_command_should_not_be_relevant_in_visual_mode() {
        let command = InsertNewLineCommand::new();
        let context = create_test_context();

        let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert!(!command.is_relevant(key_event, EditorMode::Visual, &context));
    }

    #[test]
    fn insert_newline_command_should_not_be_relevant_in_command_mode() {
        let command = InsertNewLineCommand::new();
        let context = create_test_context();

        let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert!(!command.is_relevant(key_event, EditorMode::Command, &context));
    }

    #[test]
    fn insert_newline_command_should_not_be_relevant_for_non_enter_keys() {
        let command = InsertNewLineCommand::new();
        let context = create_test_context();

        // Test various non-Enter keys
        let key_event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        assert!(!command.is_relevant(key_event, EditorMode::Insert, &context));

        let key_event = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
        assert!(!command.is_relevant(key_event, EditorMode::Insert, &context));

        let key_event = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);
        assert!(!command.is_relevant(key_event, EditorMode::Insert, &context));
    }

    #[test]
    fn insert_newline_command_should_not_be_relevant_with_modifiers() {
        let command = InsertNewLineCommand::new();
        let context = create_test_context();

        // Test Enter with various modifiers
        let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::SHIFT);
        assert!(!command.is_relevant(key_event, EditorMode::Insert, &context));

        let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::CONTROL);
        assert!(!command.is_relevant(key_event, EditorMode::Insert, &context));

        let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::ALT);
        assert!(!command.is_relevant(key_event, EditorMode::Insert, &context));
    }

    #[test]
    fn insert_newline_command_should_not_be_relevant_in_response_pane() {
        let command = InsertNewLineCommand::new();
        let mut context = create_test_context();
        context.current_pane = Pane::Response;

        let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert!(!command.is_relevant(key_event, EditorMode::Insert, &context));
    }

    #[test]
    fn insert_newline_command_should_not_be_relevant_when_read_only() {
        let command = InsertNewLineCommand::new();
        let mut context = create_test_context();
        context.is_read_only = true;

        let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert!(!command.is_relevant(key_event, EditorMode::Insert, &context));
    }

    #[test]
    fn insert_newline_command_should_handle_various_editor_modes() {
        let command = InsertNewLineCommand::new();
        let context = create_test_context();
        let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);

        // Test all modes
        assert!(command.is_relevant(key_event, EditorMode::Insert, &context));
        assert!(command.is_relevant(key_event, EditorMode::VisualBlockInsert, &context));
        assert!(!command.is_relevant(key_event, EditorMode::Normal, &context));
        assert!(!command.is_relevant(key_event, EditorMode::Visual, &context));
        assert!(!command.is_relevant(key_event, EditorMode::VisualLine, &context));
        assert!(!command.is_relevant(key_event, EditorMode::VisualBlock, &context));
        assert!(!command.is_relevant(key_event, EditorMode::Command, &context));
    }

    #[test]
    fn insert_newline_command_should_create_default_instance() {
        let command = InsertNewLineCommand;
        assert_eq!(command.name(), "InsertNewLineCommand");
    }

    #[test]
    fn insert_newline_command_execute_should_insert_newline() {
        let command = InsertNewLineCommand::new();
        let mut app_state = crate::repl::models::AppState::new();
        let mut services = crate::repl::services::Services::new();

        // Set to Insert mode
        app_state.change_mode(EditorMode::Insert).unwrap();

        let mut exec_context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        // Create a key event for Enter
        let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);

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

    #[test]
    fn insert_newline_command_should_match_key_code_correctly() {
        let command = InsertNewLineCommand::new();
        let context = create_test_context();

        // Helper function to test key relevance
        let is_enter_key = |key_code: KeyCode| {
            let key_event = KeyEvent::new(key_code, KeyModifiers::NONE);
            command.is_relevant(key_event, EditorMode::Insert, &context)
        };

        // Only Enter should be relevant
        assert!(is_enter_key(KeyCode::Enter));
        assert!(!is_enter_key(KeyCode::Char('\n'))); // Not the same as Enter key
        assert!(!is_enter_key(KeyCode::Char('\r')));
        assert!(!is_enter_key(KeyCode::Esc));
        assert!(!is_enter_key(KeyCode::Tab));
    }

    #[test]
    fn insert_newline_command_should_handle_modifier_combinations() {
        let command = InsertNewLineCommand::new();
        let context = create_test_context();

        // Test all modifier combinations with Enter
        let modifiers = [
            KeyModifiers::NONE,
            KeyModifiers::SHIFT,
            KeyModifiers::CONTROL,
            KeyModifiers::ALT,
            KeyModifiers::SHIFT | KeyModifiers::CONTROL,
            KeyModifiers::SHIFT | KeyModifiers::ALT,
            KeyModifiers::CONTROL | KeyModifiers::ALT,
            KeyModifiers::SHIFT | KeyModifiers::CONTROL | KeyModifiers::ALT,
        ];

        for modifier in modifiers.iter() {
            let key_event = KeyEvent::new(KeyCode::Enter, *modifier);
            let should_be_relevant = *modifier == KeyModifiers::NONE;
            
            assert_eq!(
                command.is_relevant(key_event, EditorMode::Insert, &context),
                should_be_relevant,
                "Failed for modifier: {modifier:?}"
            );
        }
    }

    #[test]
    fn insert_newline_command_should_respect_pane_restrictions() {
        let command = InsertNewLineCommand::new();
        let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);

        // Test all panes
        let panes = [Pane::Request, Pane::Response];

        for pane in panes.iter() {
            let mut context = create_test_context();
            context.current_pane = *pane;

            let should_be_relevant = *pane == Pane::Request;
            
            assert_eq!(
                command.is_relevant(key_event, EditorMode::Insert, &context),
                should_be_relevant,
                "Failed for pane: {pane:?}"
            );
        }
    }
}

// Auto-register this command using the inventory system
register_command!(InsertNewLineCommand, "InsertNewLineCommand");