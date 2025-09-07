//! # End Key Command
//!
//! Command to move cursor to end of line (End key functionality).
//! Works in Normal, Visual, and Insert modes.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use crate::register_command;
use crate::repl::models::pane_state::EditorMode;
use crate::repl::view_models::commands::{Command, CommandContext, ExecutionContext};
use crate::repl::view_models::post_command_actions::PostCommandAction;

/// Command to move cursor to end of current line
///
/// This command:
/// 1. Handles End key in Normal, Visual, and Insert modes without modifiers
/// 2. Uses PaneManager's move_cursor_to_end_of_line() method for business logic
/// 3. Works in both writable and read-only panes
/// 4. Supports visual mode selection extension
pub struct EndKeyCommand;

impl EndKeyCommand {
    /// Create new EndKeyCommand
    pub fn new() -> Self {
        Self
    }
}

impl Command for EndKeyCommand {
    fn is_relevant(
        &self,
        key_event: KeyEvent,
        _mode: EditorMode,
        _context: &CommandContext,
    ) -> bool {
        // End key works in all modes without any modifiers
        matches!(key_event.code, KeyCode::End) && key_event.modifiers.is_empty()
    }

    fn execute(
        &self,
        _key_event: KeyEvent,
        context: &mut ExecutionContext,
    ) -> Result<Vec<PostCommandAction>> {
        // Use PaneManager's cursor movement business logic that returns PostCommandActions
        let actions = context.app_state.pane_manager.move_cursor_to_end_of_line();

        tracing::debug!(
            "EndKeyCommand executed, generated {} actions",
            actions.len()
        );

        Ok(actions)
    }

    fn name(&self) -> &'static str {
        "EndKeyCommand"
    }
}

impl Default for EndKeyCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::models::pane_state::EditorMode;
    use crate::repl::models::pane_state::Pane;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn create_test_key_event(key_code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent::new(key_code, modifiers)
    }

    fn create_test_command_context(mode: EditorMode) -> CommandContext {
        CommandContext {
            current_mode: mode,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        }
    }

    fn create_execution_context() -> (
        crate::repl::models::app_state::AppState,
        crate::repl::services::Services,
    ) {
        let app_state = crate::repl::models::app_state::AppState::new();
        let services = crate::repl::services::Services::new();
        (app_state, services)
    }

    #[test]
    fn end_key_command_should_be_relevant_for_end_key() {
        let context = create_test_command_context(EditorMode::Normal);
        let cmd = EndKeyCommand::new();
        let event = create_test_key_event(KeyCode::End, KeyModifiers::empty());

        assert!(cmd.is_relevant(event, EditorMode::Normal, &context));
    }

    #[test]
    fn end_key_command_should_be_relevant_in_insert_mode() {
        let context = create_test_command_context(EditorMode::Insert);
        let cmd = EndKeyCommand::new();
        let event = create_test_key_event(KeyCode::End, KeyModifiers::empty());

        assert!(cmd.is_relevant(event, EditorMode::Insert, &context));
    }

    #[test]
    fn end_key_command_should_be_relevant_in_visual_mode() {
        let context = create_test_command_context(EditorMode::Visual);
        let cmd = EndKeyCommand::new();
        let event = create_test_key_event(KeyCode::End, KeyModifiers::empty());

        assert!(cmd.is_relevant(event, EditorMode::Visual, &context));
    }

    #[test]
    fn end_key_command_should_not_be_relevant_with_ctrl_modifier() {
        let context = create_test_command_context(EditorMode::Normal);
        let cmd = EndKeyCommand::new();
        let event = create_test_key_event(KeyCode::End, KeyModifiers::CONTROL);

        assert!(!cmd.is_relevant(event, EditorMode::Normal, &context));
    }

    #[test]
    fn end_key_command_should_not_be_relevant_with_shift_modifier() {
        let context = create_test_command_context(EditorMode::Normal);
        let cmd = EndKeyCommand::new();
        let event = create_test_key_event(KeyCode::End, KeyModifiers::SHIFT);

        assert!(!cmd.is_relevant(event, EditorMode::Normal, &context));
    }

    #[test]
    fn end_key_command_should_not_be_relevant_for_other_keys() {
        let context = create_test_command_context(EditorMode::Normal);
        let cmd = EndKeyCommand::new();

        // Test various non-End keys
        let test_keys = [
            KeyCode::Home,
            KeyCode::Left,
            KeyCode::Right,
            KeyCode::Up,
            KeyCode::Down,
            KeyCode::Char('l'),
            KeyCode::Char('$'),
        ];

        for key in test_keys {
            let event = create_test_key_event(key, KeyModifiers::empty());
            assert!(
                !cmd.is_relevant(event, EditorMode::Normal, &context),
                "Command should not be relevant for key: {key:?}"
            );
        }
    }

    #[test]
    fn end_key_command_execute_should_return_actions() {
        let (mut app_state, mut services) = create_execution_context();
        let mut context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };
        let cmd = EndKeyCommand::new();
        let event = create_test_key_event(KeyCode::End, KeyModifiers::empty());

        let result = cmd.execute(event, &mut context);
        assert!(result.is_ok());

        let actions = result.unwrap();
        // The exact number of actions depends on implementation,
        // but it should return at least one action
        assert!(!actions.is_empty());
    }

    #[test]
    fn end_key_command_name_should_be_correct() {
        let cmd = EndKeyCommand::new();
        assert_eq!(cmd.name(), "EndKeyCommand");
    }

    #[test]
    fn end_key_command_should_work_in_command_mode() {
        let context = create_test_command_context(EditorMode::Command);
        let cmd = EndKeyCommand::new();
        let event = create_test_key_event(KeyCode::End, KeyModifiers::empty());

        assert!(cmd.is_relevant(event, EditorMode::Command, &context));
    }

    #[test]
    fn end_key_command_should_work_in_visual_line_mode() {
        let context = create_test_command_context(EditorMode::VisualLine);
        let cmd = EndKeyCommand::new();
        let event = create_test_key_event(KeyCode::End, KeyModifiers::empty());

        assert!(cmd.is_relevant(event, EditorMode::VisualLine, &context));
    }

    #[test]
    fn end_key_command_should_work_in_visual_block_mode() {
        let context = create_test_command_context(EditorMode::VisualBlock);
        let cmd = EndKeyCommand::new();
        let event = create_test_key_event(KeyCode::End, KeyModifiers::empty());

        assert!(cmd.is_relevant(event, EditorMode::VisualBlock, &context));
    }

    #[test]
    fn end_key_command_should_not_be_relevant_with_alt_modifier() {
        let context = create_test_command_context(EditorMode::Normal);
        let cmd = EndKeyCommand::new();
        let event = create_test_key_event(KeyCode::End, KeyModifiers::ALT);

        assert!(!cmd.is_relevant(event, EditorMode::Normal, &context));
    }

    #[test]
    fn end_key_command_should_not_be_relevant_with_multiple_modifiers() {
        let context = create_test_command_context(EditorMode::Normal);
        let cmd = EndKeyCommand::new();
        let event =
            create_test_key_event(KeyCode::End, KeyModifiers::CONTROL | KeyModifiers::SHIFT);

        assert!(!cmd.is_relevant(event, EditorMode::Normal, &context));
    }

    #[test]
    fn end_key_command_execute_should_handle_errors_gracefully() {
        let (mut app_state, mut services) = create_execution_context();
        let mut context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };
        let cmd = EndKeyCommand::new();
        let event = create_test_key_event(KeyCode::End, KeyModifiers::empty());

        // Even with potentially problematic state, execute should not panic
        let result = cmd.execute(event, &mut context);

        // Result should be Ok (even if returning empty actions)
        assert!(result.is_ok());
    }

    #[test]
    fn end_key_command_default_should_work() {
        let cmd1 = EndKeyCommand::new();
        let cmd2 = EndKeyCommand;

        assert_eq!(cmd1.name(), cmd2.name());
    }

    #[test]
    fn end_key_command_should_not_interfere_with_vim_dollar_command() {
        // Ensure End key doesn't conflict with Vim's '$' command
        let context = create_test_command_context(EditorMode::Normal);
        let cmd = EndKeyCommand::new();
        let dollar_event = create_test_key_event(KeyCode::Char('$'), KeyModifiers::empty());

        assert!(!cmd.is_relevant(dollar_event, EditorMode::Normal, &context));
    }
}

// Register the command using the dynamic discovery system
register_command!(EndKeyCommand, "EndKeyCommand");
