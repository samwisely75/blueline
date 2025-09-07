//! # Ex Set Clipboard Command
//!
//! Handles `:set clipboard on`, `:set clipboard off` and `:set clipboard!` (toggle) ex commands
//! for controlling system clipboard integration.
//! This follows the unified command system approach for ex commands.

use crate::register_command;
use crate::repl::models::pane_state::EditorMode;
use crate::repl::view_models::commands::{Command, CommandContext, ExecutionContext};
use crate::repl::view_models::post_command_actions::PostCommandAction;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

/// Command for handling set clipboard ex commands (:set clipboard on/off, :set clipboard!)
pub struct ExSetClipboardCommand;

impl ExSetClipboardCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ExSetClipboardCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl Command for ExSetClipboardCommand {
    fn is_relevant(&self, key_event: KeyEvent, mode: EditorMode, context: &CommandContext) -> bool {
        // Check if Enter key is pressed in Command mode
        if key_event.code != KeyCode::Enter || mode != EditorMode::Command {
            return false;
        }

        // Check if ex command buffer contains clipboard commands
        let buffer = context.ex_command_buffer.trim();
        matches!(
            buffer,
            "set clipboard on" | "set clipboard off" | "set clipboard!"
        )
    }

    fn execute(
        &self,
        _key_event: KeyEvent,
        context: &mut ExecutionContext,
    ) -> Result<Vec<PostCommandAction>> {
        let command = context.app_state.get_ex_command_buffer().trim();

        match command {
            "set clipboard on" => {
                // Clear the command buffer and exit command mode
                context.app_state.clear_ex_command_buffer();
                let previous_mode = context.app_state.get_previous_mode();
                context.app_state.change_mode(previous_mode)?;

                // Enable clipboard integration
                context.services.yank.set_clipboard_enabled(true)?;
                context
                    .app_state
                    .set_status_message("Clipboard integration enabled");

                tracing::info!("Clipboard integration enabled");

                Ok(vec![PostCommandAction::StatusBarUpdateRequired])
            }
            "set clipboard off" => {
                // Clear the command buffer and exit command mode
                context.app_state.clear_ex_command_buffer();
                let previous_mode = context.app_state.get_previous_mode();
                context.app_state.change_mode(previous_mode)?;

                // Disable clipboard integration
                context.services.yank.set_clipboard_enabled(false)?;
                context
                    .app_state
                    .set_status_message("Clipboard integration disabled");

                tracing::info!("Clipboard integration disabled");

                Ok(vec![PostCommandAction::StatusBarUpdateRequired])
            }
            "set clipboard!" => {
                // Clear the command buffer and exit command mode
                context.app_state.clear_ex_command_buffer();
                let previous_mode = context.app_state.get_previous_mode();
                context.app_state.change_mode(previous_mode)?;

                // Toggle clipboard integration
                let current_enabled = context.services.yank.is_clipboard_enabled();
                let new_enabled = !current_enabled;
                context.services.yank.set_clipboard_enabled(new_enabled)?;

                let message = if new_enabled {
                    "Clipboard integration enabled"
                } else {
                    "Clipboard integration disabled"
                };
                context.app_state.set_status_message(message);

                tracing::info!("Clipboard integration toggled to: {}", new_enabled);

                Ok(vec![PostCommandAction::StatusBarUpdateRequired])
            }
            _ => {
                // Should not reach here due to is_relevant check
                Ok(vec![])
            }
        }
    }

    fn name(&self) -> &'static str {
        "ExSetClipboardCommand"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::models::app_state::AppState;
    use crate::repl::models::pane_state::EditorMode;
    use crate::repl::services::Services;
    use crate::repl::view_models::commands::{CommandContext, ExecutionContext};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use serial_test::serial;

    fn create_test_command_context(ex_command: &str) -> CommandContext {
        use crate::repl::models::pane_state::Pane;

        CommandContext {
            current_mode: EditorMode::Command,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: ex_command.to_string(),
        }
    }

    fn create_test_execution_context() -> (AppState, Services) {
        let app_state = AppState::new();
        let services = Services::new();
        (app_state, services)
    }

    #[test]
    fn test_is_relevant_with_clipboard_on_command() {
        let command = ExSetClipboardCommand;
        let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let context = create_test_command_context("set clipboard on");

        let result = command.is_relevant(key_event, EditorMode::Command, &context);
        assert!(result);
    }

    #[test]
    fn test_is_relevant_with_clipboard_off_command() {
        let command = ExSetClipboardCommand;
        let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let context = create_test_command_context("set clipboard off");

        let result = command.is_relevant(key_event, EditorMode::Command, &context);
        assert!(result);
    }

    #[test]
    fn test_is_relevant_with_clipboard_toggle_command() {
        let command = ExSetClipboardCommand;
        let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let context = create_test_command_context("set clipboard!");

        let result = command.is_relevant(key_event, EditorMode::Command, &context);
        assert!(result);
    }

    #[test]
    fn test_is_relevant_with_non_enter_key() {
        let command = ExSetClipboardCommand;
        let key_event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        let context = create_test_command_context("set clipboard on");

        let result = command.is_relevant(key_event, EditorMode::Command, &context);
        assert!(!result);
    }

    #[test]
    fn test_is_relevant_with_wrong_mode() {
        let command = ExSetClipboardCommand;
        let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let context = create_test_command_context("set clipboard on");

        let result = command.is_relevant(key_event, EditorMode::Normal, &context);
        assert!(!result);
    }

    #[test]
    fn test_is_relevant_with_unrelated_command() {
        let command = ExSetClipboardCommand;
        let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let context = create_test_command_context("set number on");

        let result = command.is_relevant(key_event, EditorMode::Command, &context);
        assert!(!result);
    }

    #[test]
    #[serial]
    fn test_execute_set_clipboard_on() {
        let command = ExSetClipboardCommand;
        let (mut app_state, mut services) = create_test_execution_context();
        app_state.set_ex_command_buffer("set clipboard on".to_string());
        let mut execution_context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        let result = command.execute(
            KeyEvent::new(KeyCode::Null, KeyModifiers::empty()),
            &mut execution_context,
        );
        assert!(result.is_ok());

        let actions = result.unwrap();
        assert_eq!(actions.len(), 1);
        assert!(matches!(
            actions[0],
            PostCommandAction::StatusBarUpdateRequired
        ));

        // Check that clipboard is enabled
        assert!(execution_context.services.yank.is_clipboard_enabled());
    }

    #[test]
    #[serial]
    fn test_execute_set_clipboard_off() {
        let command = ExSetClipboardCommand;
        let (mut app_state, mut services) = create_test_execution_context();
        app_state.set_ex_command_buffer("set clipboard off".to_string());
        let mut execution_context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        // First enable clipboard
        execution_context
            .services
            .yank
            .set_clipboard_enabled(true)
            .unwrap();
        assert!(execution_context.services.yank.is_clipboard_enabled());

        let result = command.execute(
            KeyEvent::new(KeyCode::Null, KeyModifiers::empty()),
            &mut execution_context,
        );
        assert!(result.is_ok());

        let actions = result.unwrap();
        assert_eq!(actions.len(), 1);
        assert!(matches!(
            actions[0],
            PostCommandAction::StatusBarUpdateRequired
        ));

        // Check that clipboard is disabled
        assert!(!execution_context.services.yank.is_clipboard_enabled());
    }

    #[test]
    #[serial]
    fn test_execute_set_clipboard_toggle() {
        let command = ExSetClipboardCommand;
        let (mut app_state, mut services) = create_test_execution_context();
        app_state.set_ex_command_buffer("set clipboard!".to_string());
        let mut execution_context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        // Initially disabled
        assert!(!execution_context.services.yank.is_clipboard_enabled());

        // First toggle should enable
        let result = command.execute(
            KeyEvent::new(KeyCode::Null, KeyModifiers::empty()),
            &mut execution_context,
        );
        assert!(result.is_ok());
        assert!(execution_context.services.yank.is_clipboard_enabled());

        // Set up for second toggle
        execution_context
            .app_state
            .set_ex_command_buffer("set clipboard!".to_string());

        // Second toggle should disable
        let result = command.execute(
            KeyEvent::new(KeyCode::Null, KeyModifiers::empty()),
            &mut execution_context,
        );
        assert!(result.is_ok());
        assert!(!execution_context.services.yank.is_clipboard_enabled());
    }

    #[test]
    fn test_command_name() {
        let command = ExSetClipboardCommand;
        assert_eq!(command.name(), "ExSetClipboardCommand");
    }
}

// Auto-register this command using the inventory system
register_command!(ExSetClipboardCommand, "ExSetClipboardCommand");
