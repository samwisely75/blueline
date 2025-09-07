//! # Insert Tab Command
//!
//! This command handles tab insertion in Insert and VisualBlockInsert modes.
//! It processes Tab key presses and inserts either tab characters or spaces
//! based on the expand_tab setting.

use anyhow::Result;
use crossterm::event::KeyEvent;

use crate::register_command;
use crate::repl::models::pane_state::{EditorMode, Pane};
use crate::repl::view_models::commands::{Command, CommandContext, ExecutionContext};
use crate::repl::view_models::post_command_actions::PostCommandAction;
use crossterm::event::KeyCode;

/// Command to handle tab insertion
///
/// This command handles the insertion of tab characters or spaces when the Tab key
/// is pressed in Insert or VisualBlockInsert mode. The behavior depends on the
/// expand_tab setting - if true, spaces are inserted, otherwise a tab character.
pub struct InsertTabCommand;

impl InsertTabCommand {
    /// Create new InsertTabCommand
    pub fn new() -> Self {
        Self
    }
}

impl Default for InsertTabCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl Command for InsertTabCommand {
    fn is_relevant(&self, key_event: KeyEvent, mode: EditorMode, context: &CommandContext) -> bool {
        // Handle Tab key input in Insert mode
        // Must be in Insert or VisualBlockInsert mode, in Request pane, not read-only
        let correct_mode = matches!(mode, EditorMode::Insert | EditorMode::VisualBlockInsert);
        let correct_pane = context.current_pane == Pane::Request;
        let not_read_only = !context.is_read_only;

        // Check if it's a Tab key without any modifiers
        let is_tab = matches!(key_event.code, KeyCode::Tab);
        let no_modifiers = key_event.modifiers.is_empty();

        correct_mode && correct_pane && not_read_only && is_tab && no_modifiers
    }

    fn execute(
        &self,
        _key_event: KeyEvent,
        context: &mut ExecutionContext,
    ) -> Result<Vec<PostCommandAction>> {
        tracing::debug!("InsertTabCommand: Inserting tab");

        // Get tab settings from the app state
        let expand_tab = context.app_state.pane_manager().get_expand_tab();
        let tab_width = context.app_state.pane_manager().get_tab_width();

        // Insert based on expand_tab setting
        if expand_tab {
            // Insert spaces instead of tab
            let spaces = " ".repeat(tab_width);
            for ch in spaces.chars() {
                context.app_state.insert_char(ch)?;
            }
            tracing::debug!("InsertTabCommand: Inserted {} spaces", tab_width);
        } else {
            // Insert actual tab character
            context.app_state.insert_char('\t')?;
            tracing::debug!("InsertTabCommand: Inserted tab character");
        }

        // Return appropriate post-command actions for UI updates
        Ok(vec![
            PostCommandAction::CurrentAreaRedrawRequired,
            PostCommandAction::ActiveCursorUpdateRequired,
        ])
    }

    fn name(&self) -> &'static str {
        "InsertTabCommand"
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
    fn insert_tab_command_should_return_correct_name() {
        let command = InsertTabCommand::new();
        assert_eq!(command.name(), "InsertTabCommand");
    }

    #[test]
    fn insert_tab_command_should_be_relevant_for_tab_key_in_insert_mode() {
        let command = InsertTabCommand::new();
        let context = create_test_context();

        let key_event = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
        assert!(command.is_relevant(key_event, EditorMode::Insert, &context));
    }

    #[test]
    fn insert_tab_command_should_be_relevant_in_visual_block_insert_mode() {
        let command = InsertTabCommand::new();
        let context = create_test_context();

        let key_event = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
        assert!(command.is_relevant(key_event, EditorMode::VisualBlockInsert, &context));
    }

    #[test]
    fn insert_tab_command_should_not_be_relevant_in_normal_mode() {
        let command = InsertTabCommand::new();
        let context = create_test_context();

        let key_event = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
        assert!(!command.is_relevant(key_event, EditorMode::Normal, &context));
    }

    #[test]
    fn insert_tab_command_should_not_be_relevant_in_visual_mode() {
        let command = InsertTabCommand::new();
        let context = create_test_context();

        let key_event = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
        assert!(!command.is_relevant(key_event, EditorMode::Visual, &context));
    }

    #[test]
    fn insert_tab_command_should_not_be_relevant_with_modifiers() {
        let command = InsertTabCommand::new();
        let context = create_test_context();

        // Tab with Shift
        let key_event = KeyEvent::new(KeyCode::Tab, KeyModifiers::SHIFT);
        assert!(!command.is_relevant(key_event, EditorMode::Insert, &context));

        // Tab with Control
        let key_event = KeyEvent::new(KeyCode::Tab, KeyModifiers::CONTROL);
        assert!(!command.is_relevant(key_event, EditorMode::Insert, &context));

        // Tab with Alt
        let key_event = KeyEvent::new(KeyCode::Tab, KeyModifiers::ALT);
        assert!(!command.is_relevant(key_event, EditorMode::Insert, &context));
    }

    #[test]
    fn insert_tab_command_should_not_be_relevant_for_non_tab_keys() {
        let command = InsertTabCommand::new();
        let context = create_test_context();

        // Char key
        let key_event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        assert!(!command.is_relevant(key_event, EditorMode::Insert, &context));

        // Enter key
        let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert!(!command.is_relevant(key_event, EditorMode::Insert, &context));

        // Arrow key
        let key_event = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
        assert!(!command.is_relevant(key_event, EditorMode::Insert, &context));
    }

    #[test]
    fn insert_tab_command_should_not_be_relevant_in_response_pane() {
        let command = InsertTabCommand::new();
        let mut context = create_test_context();
        context.current_pane = Pane::Response;

        let key_event = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
        assert!(!command.is_relevant(key_event, EditorMode::Insert, &context));
    }

    #[test]
    fn insert_tab_command_should_not_be_relevant_when_read_only() {
        let command = InsertTabCommand::new();
        let mut context = create_test_context();
        context.is_read_only = true;

        let key_event = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
        assert!(!command.is_relevant(key_event, EditorMode::Insert, &context));
    }

    #[test]
    fn insert_tab_command_should_create_default_instance() {
        let command = InsertTabCommand;
        assert_eq!(command.name(), "InsertTabCommand");
    }

    #[test]
    fn insert_tab_command_execute_should_insert_tab() {
        let command = InsertTabCommand::new();
        let mut app_state = crate::repl::models::AppState::new();
        let mut services = crate::repl::services::Services::new();

        // Set to Insert mode
        app_state.change_mode(EditorMode::Insert).unwrap();

        let mut exec_context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        // Create a key event for Tab
        let key_event = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);

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
register_command!(InsertTabCommand, "InsertTabCommand");
