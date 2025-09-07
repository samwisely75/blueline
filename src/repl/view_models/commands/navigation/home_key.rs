//! # Home Key Command
//!
//! Command to move cursor to beginning of line (Home key functionality).
//! Works in Normal, Visual, and Insert modes.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use crate::register_command;
use crate::repl::models::pane_state::EditorMode;
use crate::repl::view_models::commands::{Command, CommandContext, ExecutionContext};
use crate::repl::view_models::post_command_actions::PostCommandAction;

/// Command to move cursor to beginning of current line
///
/// This command:
/// 1. Handles Home key in Normal, Visual, and Insert modes without modifiers
/// 2. Uses PaneManager's move_cursor_to_start_of_line() method for business logic
/// 3. Works in both writable and read-only panes
/// 4. Supports visual mode selection extension
pub struct HomeKeyCommand;

impl HomeKeyCommand {
    /// Create new HomeKeyCommand
    pub fn new() -> Self {
        Self
    }
}

impl Command for HomeKeyCommand {
    fn is_relevant(
        &self,
        key_event: KeyEvent,
        _mode: EditorMode,
        _context: &CommandContext,
    ) -> bool {
        // Home key works in all modes without any modifiers
        matches!(key_event.code, KeyCode::Home) && key_event.modifiers.is_empty()
    }

    fn execute(
        &self,
        _key_event: KeyEvent,
        context: &mut ExecutionContext,
    ) -> Result<Vec<PostCommandAction>> {
        // Move cursor to start of line
        context
            .app_state
            .pane_manager
            .move_cursor_to_start_of_line();

        tracing::debug!("HomeKeyCommand executed");

        // Generate view update events based on what this command did
        Ok(vec![
            PostCommandAction::ActiveCursorUpdateRequired,
            PostCommandAction::PositionIndicatorUpdateRequired,
        ])
    }

    fn name(&self) -> &'static str {
        "HomeKeyCommand"
    }
}

impl Default for HomeKeyCommand {
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
    fn home_key_command_should_be_relevant_for_home_key() {
        let context = create_test_command_context(EditorMode::Normal);
        let cmd = HomeKeyCommand::new();
        let event = create_test_key_event(KeyCode::Home, KeyModifiers::empty());

        assert!(cmd.is_relevant(event, EditorMode::Normal, &context));
    }

    #[test]
    fn home_key_command_should_be_relevant_in_insert_mode() {
        let context = create_test_command_context(EditorMode::Insert);
        let cmd = HomeKeyCommand::new();
        let event = create_test_key_event(KeyCode::Home, KeyModifiers::empty());

        assert!(cmd.is_relevant(event, EditorMode::Insert, &context));
    }

    #[test]
    fn home_key_command_should_be_relevant_in_visual_mode() {
        let context = create_test_command_context(EditorMode::Visual);
        let cmd = HomeKeyCommand::new();
        let event = create_test_key_event(KeyCode::Home, KeyModifiers::empty());

        assert!(cmd.is_relevant(event, EditorMode::Visual, &context));
    }

    #[test]
    fn home_key_command_should_not_be_relevant_with_ctrl_modifier() {
        let context = create_test_command_context(EditorMode::Normal);
        let cmd = HomeKeyCommand::new();
        let event = create_test_key_event(KeyCode::Home, KeyModifiers::CONTROL);

        assert!(!cmd.is_relevant(event, EditorMode::Normal, &context));
    }

    #[test]
    fn home_key_command_should_not_be_relevant_with_shift_modifier() {
        let context = create_test_command_context(EditorMode::Normal);
        let cmd = HomeKeyCommand::new();
        let event = create_test_key_event(KeyCode::Home, KeyModifiers::SHIFT);

        assert!(!cmd.is_relevant(event, EditorMode::Normal, &context));
    }

    #[test]
    fn home_key_command_should_not_be_relevant_for_other_keys() {
        let context = create_test_command_context(EditorMode::Normal);
        let cmd = HomeKeyCommand::new();

        // Test various non-Home keys
        let test_keys = [
            KeyCode::End,
            KeyCode::Left,
            KeyCode::Right,
            KeyCode::Up,
            KeyCode::Down,
            KeyCode::Char('h'),
            KeyCode::Char('0'),
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
    fn home_key_command_execute_should_return_actions() {
        let (mut app_state, mut services) = create_execution_context();
        let mut context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };
        let cmd = HomeKeyCommand::new();
        let event = create_test_key_event(KeyCode::Home, KeyModifiers::empty());

        let result = cmd.execute(event, &mut context);
        assert!(result.is_ok());

        let actions = result.unwrap();
        // The exact number of actions depends on implementation,
        // but it should return at least one action
        assert!(!actions.is_empty());
    }

    #[test]
    fn home_key_command_name_should_be_correct() {
        let cmd = HomeKeyCommand::new();
        assert_eq!(cmd.name(), "HomeKeyCommand");
    }

    #[test]
    fn home_key_command_should_work_in_command_mode() {
        let context = create_test_command_context(EditorMode::Command);
        let cmd = HomeKeyCommand::new();
        let event = create_test_key_event(KeyCode::Home, KeyModifiers::empty());

        assert!(cmd.is_relevant(event, EditorMode::Command, &context));
    }

    #[test]
    fn home_key_command_should_work_in_visual_line_mode() {
        let context = create_test_command_context(EditorMode::VisualLine);
        let cmd = HomeKeyCommand::new();
        let event = create_test_key_event(KeyCode::Home, KeyModifiers::empty());

        assert!(cmd.is_relevant(event, EditorMode::VisualLine, &context));
    }

    #[test]
    fn home_key_command_should_work_in_visual_block_mode() {
        let context = create_test_command_context(EditorMode::VisualBlock);
        let cmd = HomeKeyCommand::new();
        let event = create_test_key_event(KeyCode::Home, KeyModifiers::empty());

        assert!(cmd.is_relevant(event, EditorMode::VisualBlock, &context));
    }

    #[test]
    fn home_key_command_should_not_be_relevant_with_alt_modifier() {
        let context = create_test_command_context(EditorMode::Normal);
        let cmd = HomeKeyCommand::new();
        let event = create_test_key_event(KeyCode::Home, KeyModifiers::ALT);

        assert!(!cmd.is_relevant(event, EditorMode::Normal, &context));
    }

    #[test]
    fn home_key_command_should_not_be_relevant_with_multiple_modifiers() {
        let context = create_test_command_context(EditorMode::Normal);
        let cmd = HomeKeyCommand::new();
        let event =
            create_test_key_event(KeyCode::Home, KeyModifiers::CONTROL | KeyModifiers::SHIFT);

        assert!(!cmd.is_relevant(event, EditorMode::Normal, &context));
    }

    #[test]
    fn home_key_command_execute_should_handle_errors_gracefully() {
        let (mut app_state, mut services) = create_execution_context();
        let mut context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };
        let cmd = HomeKeyCommand::new();
        let event = create_test_key_event(KeyCode::Home, KeyModifiers::empty());

        // Even with potentially problematic state, execute should not panic
        let result = cmd.execute(event, &mut context);

        // Result should be Ok (even if returning empty actions)
        assert!(result.is_ok());
    }

    #[test]
    fn home_key_command_default_should_work() {
        let cmd1 = HomeKeyCommand::new();
        let cmd2 = HomeKeyCommand;

        assert_eq!(cmd1.name(), cmd2.name());
    }
}

// Register the command using the dynamic discovery system
register_command!(HomeKeyCommand, "HomeKeyCommand");
