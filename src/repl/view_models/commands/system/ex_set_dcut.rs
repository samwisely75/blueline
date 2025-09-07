//! # Ex Set DCut Command
//!
//! Handles `:set dcut on`, `:set dcut off` and `:set dcut!` (toggle) ex commands
//! for controlling dcut behavior settings.
//! This follows the unified command system approach for ex commands.

use crate::repl::models::pane_state::EditorMode;
use crate::repl::view_models::commands::{Command, CommandContext, ExecutionContext};
use crate::repl::view_models::post_command_actions::PostCommandAction;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

/// Command for handling set dcut ex commands (:set dcut on/off, :set dcut!)
pub struct ExSetDCutCommand;

impl Command for ExSetDCutCommand {
    fn is_relevant(&self, key_event: KeyEvent, mode: EditorMode, context: &CommandContext) -> bool {
        // Check if Enter key is pressed in Command mode
        if key_event.code != KeyCode::Enter || mode != EditorMode::Command {
            return false;
        }

        // Check if ex command buffer contains dcut commands
        let buffer = context.ex_command_buffer.trim();
        matches!(buffer, "set dcut on" | "set dcut off" | "set dcut!")
    }

    fn execute(
        &self,
        _key_event: KeyEvent,
        context: &mut ExecutionContext,
    ) -> Result<Vec<PostCommandAction>> {
        let command = context.app_state.get_ex_command_buffer().trim();

        match command {
            "set dcut on" => {
                // Clear the command buffer and exit command mode
                context.app_state.clear_ex_command_buffer();
                let previous_mode = context.app_state.get_previous_mode();
                context.app_state.change_mode(previous_mode)?;

                // Enable dcut
                context.app_state.set_dcut_enabled(true);

                // Set status message
                context
                    .app_state
                    .set_status_message("DCut enabled (delete operations will yank)".to_string());

                Ok(vec![PostCommandAction::StatusBarUpdateRequired])
            }
            "set dcut off" => {
                // Clear the command buffer and exit command mode
                context.app_state.clear_ex_command_buffer();
                let previous_mode = context.app_state.get_previous_mode();
                context.app_state.change_mode(previous_mode)?;

                // Disable dcut
                context.app_state.set_dcut_enabled(false);

                // Set status message
                context.app_state.set_status_message(
                    "DCut disabled (delete operations will not yank)".to_string(),
                );

                Ok(vec![PostCommandAction::StatusBarUpdateRequired])
            }
            "set dcut!" => {
                // Clear the command buffer and exit command mode
                context.app_state.clear_ex_command_buffer();
                let previous_mode = context.app_state.get_previous_mode();
                context.app_state.change_mode(previous_mode)?;

                // Toggle dcut state
                let current_dcut = context.app_state.is_dcut_enabled();
                context.app_state.set_dcut_enabled(!current_dcut);

                // Set status message based on new state
                let status = if !current_dcut {
                    "DCut enabled (delete operations will yank)"
                } else {
                    "DCut disabled (delete operations will not yank)"
                };
                context.app_state.set_status_message(status.to_string());

                Ok(vec![PostCommandAction::StatusBarUpdateRequired])
            }
            _ => {
                // This shouldn't happen given our is_relevant check, but handle gracefully
                tracing::warn!(
                    "ExSetDCutCommand executed with unexpected buffer: {}",
                    command
                );
                Ok(vec![])
            }
        }
    }

    fn name(&self) -> &'static str {
        "ExSetDCutCommand"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::models::pane_state::EditorMode;
    use crate::repl::view_models::commands::CommandContext;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn create_test_context(buffer_content: &str) -> CommandContext {
        CommandContext {
            current_mode: EditorMode::Command,
            current_pane: crate::repl::models::pane_state::Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: buffer_content.to_string(),
        }
    }

    #[test]
    fn test_is_relevant_with_set_dcut_on() {
        let command = ExSetDCutCommand;
        let context = create_test_context("set dcut on");
        let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);

        assert!(command.is_relevant(key_event, EditorMode::Command, &context));
    }

    #[test]
    fn test_is_relevant_with_set_dcut_off() {
        let command = ExSetDCutCommand;
        let context = create_test_context("set dcut off");
        let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);

        assert!(command.is_relevant(key_event, EditorMode::Command, &context));
    }

    #[test]
    fn test_is_relevant_with_set_dcut_toggle() {
        let command = ExSetDCutCommand;
        let context = create_test_context("set dcut!");
        let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);

        assert!(command.is_relevant(key_event, EditorMode::Command, &context));
    }

    #[test]
    fn test_is_relevant_with_whitespace() {
        let command = ExSetDCutCommand;
        let context = create_test_context("  set dcut on  ");
        let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);

        assert!(command.is_relevant(key_event, EditorMode::Command, &context));
    }

    #[test]
    fn test_is_not_relevant_with_different_command() {
        let command = ExSetDCutCommand;
        let context = create_test_context("set wrap on");
        let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);

        assert!(!command.is_relevant(key_event, EditorMode::Command, &context));
    }

    #[test]
    fn test_is_not_relevant_with_wrong_key() {
        let command = ExSetDCutCommand;
        let context = create_test_context("set dcut on");
        let key_event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);

        assert!(!command.is_relevant(key_event, EditorMode::Command, &context));
    }

    #[test]
    fn test_is_not_relevant_with_wrong_mode() {
        let command = ExSetDCutCommand;
        let context = create_test_context("set dcut on");
        let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);

        assert!(!command.is_relevant(key_event, EditorMode::Normal, &context));
    }

    #[test]
    fn test_is_not_relevant_with_invalid_dcut_command() {
        let command = ExSetDCutCommand;
        let test_cases = vec!["set dcut", "set dcut toggle", "dcut on", "set cut on"];

        for case in test_cases {
            let context = create_test_context(case);
            let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
            assert!(
                !command.is_relevant(key_event, EditorMode::Command, &context),
                "Should not be relevant for: {case}"
            );
        }
    }

    // Note: Execution tests are complex due to lifetime management issues
    // with ExecutionContext. The business logic is well-tested through
    // integration tests and existing ex command manager system.

    #[test]
    fn test_command_name() {
        let command = ExSetDCutCommand;
        assert_eq!(command.name(), "ExSetDCutCommand");
    }
}

// Register the command
inventory::submit!(
    crate::repl::view_models::commands::dynamic_registry::CommandEntry {
        name: "ExSetDCutCommand",
        factory: || Box::new(ExSetDCutCommand),
    }
);
