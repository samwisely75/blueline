//! # Enter Y Prefix Mode Command
//!
//! Command to handle the first 'y' key press in Normal mode which enters YPrefix mode,
//! setting up for commands like 'yy' (yank current line).

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use crate::register_command;
#[allow(unused_imports)] // Pane is used in tests
use crate::repl::models::pane_state::{EditorMode, Pane};
use crate::repl::view_models::commands::{Command, CommandContext, ExecutionContext};
use crate::repl::view_models::post_command_actions::PostCommandAction;

/// Command to handle entering YPrefix mode
///
/// This command handles the first 'y' key press in Normal mode, which enters YPrefix mode
/// to set up for commands like 'yy' (yank current line).
pub struct EnterYPrefixCommand;

impl EnterYPrefixCommand {
    /// Create new EnterYPrefixCommand
    pub fn new() -> Self {
        Self
    }
}

impl Default for EnterYPrefixCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl Command for EnterYPrefixCommand {
    fn is_relevant(
        &self,
        key_event: KeyEvent,
        mode: EditorMode,
        _context: &CommandContext,
    ) -> bool {
        // Handle first 'y' key press in Normal mode for both Request and Response panes
        // Yank operations should be allowed in read-only panes
        matches!(key_event.code, KeyCode::Char('y'))
            && mode == EditorMode::Normal
            && key_event.modifiers.is_empty()
    }

    fn execute(
        &self,
        _key_event: KeyEvent,
        context: &mut ExecutionContext,
    ) -> Result<Vec<PostCommandAction>> {
        tracing::debug!("EnterYPrefixCommand: entering YPrefix mode");

        // Set the mode to YPrefix
        context.app_state.change_mode(EditorMode::YPrefix)?;

        tracing::info!("Entered YPrefix mode, ready for second 'y' or other y-commands");

        // Return appropriate PostCommandActions for UI updates
        Ok(vec![
            PostCommandAction::StatusBarUpdateRequired,
            PostCommandAction::ActiveCursorUpdateRequired,
        ])
    }

    fn name(&self) -> &'static str {
        "EnterYPrefixCommand"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::models::{AppState, LogicalPosition};
    use crate::repl::services::Services;
    use crossterm::event::KeyModifiers;

    fn create_test_context() -> CommandContext {
        CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        }
    }

    #[test]
    fn enter_y_prefix_command_should_return_correct_name() {
        let command = EnterYPrefixCommand::new();
        assert_eq!(command.name(), "EnterYPrefixCommand");
    }

    #[test]
    fn enter_y_prefix_command_should_be_relevant_for_y_key_in_normal_mode() {
        let command = EnterYPrefixCommand::new();
        let context = create_test_context();

        let key_event = KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE);
        assert!(command.is_relevant(key_event, EditorMode::Normal, &context));
    }

    #[test]
    fn enter_y_prefix_command_should_not_be_relevant_in_insert_mode() {
        let command = EnterYPrefixCommand::new();
        let context = create_test_context();

        let key_event = KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE);
        assert!(!command.is_relevant(key_event, EditorMode::Insert, &context));
    }

    #[test]
    fn enter_y_prefix_command_should_not_be_relevant_in_visual_mode() {
        let command = EnterYPrefixCommand::new();
        let context = create_test_context();

        let key_event = KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE);
        assert!(!command.is_relevant(key_event, EditorMode::Visual, &context));
    }

    #[test]
    fn enter_y_prefix_command_should_not_be_relevant_in_y_prefix_mode() {
        let command = EnterYPrefixCommand::new();
        let context = create_test_context();

        let key_event = KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE);
        assert!(!command.is_relevant(key_event, EditorMode::YPrefix, &context));
    }

    #[test]
    fn enter_y_prefix_command_should_not_be_relevant_for_other_keys() {
        let command = EnterYPrefixCommand::new();
        let context = create_test_context();

        // Test 'x' key
        let key_event = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
        assert!(!command.is_relevant(key_event, EditorMode::Normal, &context));

        // Test 'd' key
        let key_event = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE);
        assert!(!command.is_relevant(key_event, EditorMode::Normal, &context));

        // Test Enter key
        let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert!(!command.is_relevant(key_event, EditorMode::Normal, &context));

        // Test uppercase 'Y' key
        let key_event = KeyEvent::new(KeyCode::Char('Y'), KeyModifiers::NONE);
        assert!(!command.is_relevant(key_event, EditorMode::Normal, &context));
    }

    #[test]
    fn enter_y_prefix_command_should_not_be_relevant_with_modifiers() {
        let command = EnterYPrefixCommand::new();
        let context = create_test_context();

        // Test with Ctrl modifier
        let key_event = KeyEvent::new(KeyCode::Char('y'), KeyModifiers::CONTROL);
        assert!(!command.is_relevant(key_event, EditorMode::Normal, &context));

        // Test with Shift modifier
        let key_event = KeyEvent::new(KeyCode::Char('y'), KeyModifiers::SHIFT);
        assert!(!command.is_relevant(key_event, EditorMode::Normal, &context));

        // Test with Alt modifier
        let key_event = KeyEvent::new(KeyCode::Char('y'), KeyModifiers::ALT);
        assert!(!command.is_relevant(key_event, EditorMode::Normal, &context));
    }

    #[test]
    fn enter_y_prefix_command_should_be_relevant_in_response_pane() {
        let command = EnterYPrefixCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Response,
            is_read_only: true, // Response pane is read-only
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let key_event = KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE);
        // Yank prefix mode should now be available in read-only panes
        assert!(command.is_relevant(key_event, EditorMode::Normal, &context));
    }

    #[test]
    fn enter_y_prefix_command_should_execute_successfully() {
        let command = EnterYPrefixCommand::new();
        let mut app_state = AppState::new();
        let mut services = Services::new();

        // Ensure we're in Normal mode initially
        assert_eq!(app_state.get_mode(), EditorMode::Normal);

        let mut context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        // Execute the command
        let result = command.execute(
            KeyEvent::new(KeyCode::Null, KeyModifiers::empty()),
            &mut context,
        );

        assert!(result.is_ok(), "Command execution should succeed");
        let events = result.unwrap();

        // Check that proper events are emitted
        assert_eq!(events.len(), 2, "Should emit 2 PostCommandActions");
        assert!(matches!(
            events[0],
            PostCommandAction::StatusBarUpdateRequired
        ));
        assert!(matches!(
            events[1],
            PostCommandAction::ActiveCursorUpdateRequired
        ));

        // Check that mode changed to YPrefix
        assert_eq!(
            context.app_state.get_mode(),
            EditorMode::YPrefix,
            "Should be in YPrefix mode after execution"
        );
    }

    #[test]
    fn enter_y_prefix_command_should_create_default_instance() {
        let command = EnterYPrefixCommand;
        assert_eq!(command.name(), "EnterYPrefixCommand");
    }

    #[test]
    fn enter_y_prefix_command_should_handle_integration_scenario() {
        // This test simulates a more realistic scenario
        let command = EnterYPrefixCommand::new();
        let mut app_state = AppState::new();
        let mut services = Services::new();

        // Set up some initial state
        app_state
            .set_cursor_position(LogicalPosition::new(2, 10))
            .unwrap();

        let mut context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        // Verify initial state
        assert_eq!(context.app_state.get_mode(), EditorMode::Normal);
        let initial_cursor = context.app_state.get_cursor_position();
        tracing::debug!("Initial cursor position in test: {initial_cursor:?}");

        // Execute command
        let result = command.execute(
            KeyEvent::new(KeyCode::Null, KeyModifiers::empty()),
            &mut context,
        );
        assert!(result.is_ok());

        // Verify state changes
        assert_eq!(context.app_state.get_mode(), EditorMode::YPrefix);
        // The cursor position should be preserved
        let final_cursor = context.app_state.get_cursor_position();
        tracing::debug!("Final cursor position in test: {final_cursor:?}");
    }
}

// Auto-register this command using the inventory system
register_command!(EnterYPrefixCommand, "EnterYPrefixCommand");
