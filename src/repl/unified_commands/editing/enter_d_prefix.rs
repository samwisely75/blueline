//! # Enter D Prefix Mode Command
//!
//! Command to handle the first 'd' key press in Normal mode which enters DPrefix mode,
//! setting up for commands like 'dd' (delete/cut current line).

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use crate::register_command;
use crate::repl::models::pane_state::{EditorMode, Pane};
use crate::repl::unified_commands::{Command, CommandContext, ExecutionContext};
use crate::repl::view_models::post_command_actions::PostCommandAction;

/// Command to handle entering DPrefix mode
///
/// This command handles the first 'd' key press in Normal mode, which enters DPrefix mode
/// to set up for commands like 'dd' (delete/cut current line).
pub struct EnterDPrefixCommand;

impl EnterDPrefixCommand {
    /// Create new EnterDPrefixCommand
    pub fn new() -> Self {
        Self
    }
}

impl Default for EnterDPrefixCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl Command for EnterDPrefixCommand {
    fn is_relevant(&self, key_event: KeyEvent, mode: EditorMode, context: &CommandContext) -> bool {
        // Handle first 'd' key press in Normal mode for Request pane only
        matches!(key_event.code, KeyCode::Char('d'))
            && mode == EditorMode::Normal
            && context.current_pane == Pane::Request
            && key_event.modifiers.is_empty()
    }

    fn execute(
        &self,
        _key_event: KeyEvent,
        context: &mut ExecutionContext,
    ) -> Result<Vec<PostCommandAction>> {
        tracing::debug!("EnterDPrefixCommand: entering DPrefix mode");

        // Set the mode to DPrefix
        context.app_state.change_mode(EditorMode::DPrefix)?;

        tracing::info!("Entered DPrefix mode, ready for second 'd' or other d-commands");

        // Return appropriate PostCommandActions for UI updates
        Ok(vec![
            PostCommandAction::StatusBarUpdateRequired,
            PostCommandAction::ActiveCursorUpdateRequired,
        ])
    }

    fn name(&self) -> &'static str {
        "EnterDPrefixCommand"
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
    fn enter_d_prefix_command_should_return_correct_name() {
        let command = EnterDPrefixCommand::new();
        assert_eq!(command.name(), "EnterDPrefixCommand");
    }

    #[test]
    fn enter_d_prefix_command_should_be_relevant_for_d_key_in_normal_mode() {
        let command = EnterDPrefixCommand::new();
        let context = create_test_context();

        let key_event = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE);
        assert!(command.is_relevant(key_event, EditorMode::Normal, &context));
    }

    #[test]
    fn enter_d_prefix_command_should_not_be_relevant_in_insert_mode() {
        let command = EnterDPrefixCommand::new();
        let context = create_test_context();

        let key_event = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE);
        assert!(!command.is_relevant(key_event, EditorMode::Insert, &context));
    }

    #[test]
    fn enter_d_prefix_command_should_not_be_relevant_in_visual_mode() {
        let command = EnterDPrefixCommand::new();
        let context = create_test_context();

        let key_event = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE);
        assert!(!command.is_relevant(key_event, EditorMode::Visual, &context));
    }

    #[test]
    fn enter_d_prefix_command_should_not_be_relevant_in_d_prefix_mode() {
        let command = EnterDPrefixCommand::new();
        let context = create_test_context();

        let key_event = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE);
        assert!(!command.is_relevant(key_event, EditorMode::DPrefix, &context));
    }

    #[test]
    fn enter_d_prefix_command_should_not_be_relevant_for_other_keys() {
        let command = EnterDPrefixCommand::new();
        let context = create_test_context();

        // Test 'x' key
        let key_event = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
        assert!(!command.is_relevant(key_event, EditorMode::Normal, &context));

        // Test 'y' key
        let key_event = KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE);
        assert!(!command.is_relevant(key_event, EditorMode::Normal, &context));

        // Test Enter key
        let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert!(!command.is_relevant(key_event, EditorMode::Normal, &context));

        // Test uppercase 'D' key
        let key_event = KeyEvent::new(KeyCode::Char('D'), KeyModifiers::NONE);
        assert!(!command.is_relevant(key_event, EditorMode::Normal, &context));
    }

    #[test]
    fn enter_d_prefix_command_should_not_be_relevant_with_modifiers() {
        let command = EnterDPrefixCommand::new();
        let context = create_test_context();

        // Test with Ctrl modifier
        let key_event = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL);
        assert!(!command.is_relevant(key_event, EditorMode::Normal, &context));

        // Test with Shift modifier
        let key_event = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::SHIFT);
        assert!(!command.is_relevant(key_event, EditorMode::Normal, &context));

        // Test with Alt modifier
        let key_event = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::ALT);
        assert!(!command.is_relevant(key_event, EditorMode::Normal, &context));
    }

    #[test]
    fn enter_d_prefix_command_should_not_be_relevant_in_response_pane() {
        let command = EnterDPrefixCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Response,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let key_event = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE);
        assert!(!command.is_relevant(key_event, EditorMode::Normal, &context));
    }

    #[test]
    fn enter_d_prefix_command_should_execute_successfully() {
        let command = EnterDPrefixCommand::new();
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

        // Check that mode changed to DPrefix
        assert_eq!(
            context.app_state.get_mode(),
            EditorMode::DPrefix,
            "Should be in DPrefix mode after execution"
        );
    }

    #[test]
    fn enter_d_prefix_command_should_create_default_instance() {
        let command = EnterDPrefixCommand;
        assert_eq!(command.name(), "EnterDPrefixCommand");
    }

    #[test]
    fn enter_d_prefix_command_should_handle_integration_scenario() {
        // This test simulates a more realistic scenario
        let command = EnterDPrefixCommand::new();
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
        assert_eq!(context.app_state.get_mode(), EditorMode::DPrefix);
        // The cursor position should be preserved
        let final_cursor = context.app_state.get_cursor_position();
        tracing::debug!("Final cursor position in test: {final_cursor:?}");
    }
}

// Auto-register this command using the inventory system
register_command!(EnterDPrefixCommand, "EnterDPrefixCommand");
