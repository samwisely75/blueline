//! # App Terminate Command
//!
//! Command to handle graceful application termination via Ctrl+C.
//! This command provides a clean shutdown mechanism for the application.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::register_command;
use crate::repl::models::pane_state::EditorMode;
use crate::repl::unified_commands::{Command, CommandContext, ExecutionContext};
use crate::repl::view_models::post_command_actions::PostCommandAction;

/// Command to terminate the application gracefully
///
/// This command handles Ctrl+C key combination to provide
/// graceful application shutdown with proper cleanup.
pub struct AppTerminateCommand;

impl AppTerminateCommand {
    /// Create new AppTerminateCommand
    pub fn new() -> Self {
        Self
    }
}

impl Default for AppTerminateCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl Command for AppTerminateCommand {
    fn is_relevant(
        &self,
        key_event: KeyEvent,
        _mode: EditorMode,
        _context: &CommandContext,
    ) -> bool {
        // Handle Ctrl+C key combination for app termination
        matches!(key_event.code, KeyCode::Char('c'))
            && key_event.modifiers.contains(KeyModifiers::CONTROL)
            && !key_event.modifiers.contains(KeyModifiers::SHIFT)
            && !key_event.modifiers.contains(KeyModifiers::ALT)
    }

    fn execute(&self, _context: &mut ExecutionContext) -> Result<Vec<PostCommandAction>> {
        tracing::info!("AppTerminateCommand: Received termination request (Ctrl+C)");

        // Return quit requested action for graceful shutdown
        Ok(vec![PostCommandAction::QuitRequested])
    }

    fn name(&self) -> &'static str {
        "AppTerminateCommand"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::models::pane_state::Pane;
    use crate::repl::models::AppState;
    use crate::repl::services::Services;

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
    fn app_terminate_command_should_return_correct_name() {
        let command = AppTerminateCommand::new();
        assert_eq!(command.name(), "AppTerminateCommand");
    }

    #[test]
    fn app_terminate_command_should_be_relevant_for_ctrl_c() {
        let command = AppTerminateCommand::new();
        let context = create_test_context();
        let key_event = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);

        assert!(command.is_relevant(key_event, EditorMode::Normal, &context));
    }

    #[test]
    fn app_terminate_command_should_be_relevant_in_all_modes() {
        let command = AppTerminateCommand::new();
        let context = create_test_context();
        let key_event = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);

        // Should work in all modes for emergency exit
        let modes = vec![
            EditorMode::Normal,
            EditorMode::Insert,
            EditorMode::Visual,
            EditorMode::VisualLine,
            EditorMode::VisualBlock,
            EditorMode::Command,
            EditorMode::GPrefix,
        ];

        for mode in modes {
            assert!(
                command.is_relevant(key_event, mode, &context),
                "Should be relevant in {mode:?} mode"
            );
        }
    }

    #[test]
    fn app_terminate_command_should_not_be_relevant_for_regular_c() {
        let command = AppTerminateCommand::new();
        let context = create_test_context();
        let key_event = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE);

        assert!(!command.is_relevant(key_event, EditorMode::Normal, &context));
    }

    #[test]
    fn app_terminate_command_should_not_be_relevant_for_other_ctrl_keys() {
        let command = AppTerminateCommand::new();
        let context = create_test_context();

        let other_ctrl_keys = vec![
            KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL),
            KeyEvent::new(KeyCode::Char('x'), KeyModifiers::CONTROL),
            KeyEvent::new(KeyCode::Char('z'), KeyModifiers::CONTROL),
        ];

        for key_event in other_ctrl_keys {
            assert!(
                !command.is_relevant(key_event, EditorMode::Normal, &context),
                "Should not be relevant for key: {key_event:?}"
            );
        }
    }

    #[test]
    fn app_terminate_command_should_not_be_relevant_for_modified_ctrl_c() {
        let command = AppTerminateCommand::new();
        let context = create_test_context();

        // Should not be relevant with additional modifiers
        let modified_keys = vec![
            KeyEvent::new(
                KeyCode::Char('c'),
                KeyModifiers::CONTROL | KeyModifiers::SHIFT,
            ),
            KeyEvent::new(
                KeyCode::Char('c'),
                KeyModifiers::CONTROL | KeyModifiers::ALT,
            ),
            KeyEvent::new(
                KeyCode::Char('c'),
                KeyModifiers::CONTROL | KeyModifiers::SHIFT | KeyModifiers::ALT,
            ),
        ];

        for key_event in modified_keys {
            assert!(
                !command.is_relevant(key_event, EditorMode::Normal, &context),
                "Should not be relevant for modified key: {key_event:?}"
            );
        }
    }

    #[test]
    fn app_terminate_command_should_work_in_read_only_panes() {
        let command = AppTerminateCommand::new();
        let read_only_context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Response,
            is_read_only: true,
            has_selection: false,
            ex_command_buffer: String::new(),
        };
        let key_event = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);

        // Should work even in read-only panes for emergency exit
        assert!(command.is_relevant(key_event, EditorMode::Normal, &read_only_context));
    }

    #[test]
    fn app_terminate_execute_should_return_quit_requested() {
        let command = AppTerminateCommand::new();
        let mut app_state = AppState::new();
        let mut services = Services::new();

        let mut context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        let result = command.execute(&mut context);

        assert!(result.is_ok());
        let events = result.unwrap();
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], PostCommandAction::QuitRequested));
    }

    #[test]
    fn default_should_create_new_instance() {
        let command = AppTerminateCommand;
        assert_eq!(command.name(), "AppTerminateCommand");
    }
}

// Auto-register this command using the inventory system
register_command!(AppTerminateCommand, "AppTerminateCommand");
