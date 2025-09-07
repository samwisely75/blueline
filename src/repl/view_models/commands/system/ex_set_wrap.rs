//! # Ex Set Wrap Command
//!
//! Handles `:set wrap on`, `:set wrap off` and `:set wrap!` (toggle) ex commands
//! for controlling word wrap display settings.
//! This follows the unified command system approach for ex commands.

use crate::repl::models::pane_state::EditorMode;
use crate::repl::view_models::commands::{Command, CommandContext, ExecutionContext};
use crate::repl::view_models::post_command_actions::PostCommandAction;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

/// Command for handling set wrap ex commands (:set wrap on/off, :set wrap!)
pub struct ExSetWrapCommand;

impl Command for ExSetWrapCommand {
    fn is_relevant(&self, key_event: KeyEvent, mode: EditorMode, context: &CommandContext) -> bool {
        // Check if Enter key is pressed in Command mode
        if key_event.code != KeyCode::Enter || mode != EditorMode::Command {
            return false;
        }

        // Check if ex command buffer contains wrap commands
        let buffer = context.ex_command_buffer.trim();
        matches!(buffer, "set wrap on" | "set wrap off" | "set wrap!")
    }

    fn execute(
        &self,
        _key_event: KeyEvent,
        context: &mut ExecutionContext,
    ) -> Result<Vec<PostCommandAction>> {
        let command = context.app_state.get_ex_command_buffer().trim();

        match command {
            "set wrap on" => {
                // Clear the command buffer and exit command mode
                context.app_state.clear_ex_command_buffer();
                let previous_mode = context.app_state.get_previous_mode();
                context.app_state.change_mode(previous_mode)?;

                // Enable word wrap
                context.app_state.pane_manager.set_wrap_enabled(true);
                let visibility_events = context
                    .app_state
                    .pane_manager
                    .rebuild_display_caches_and_sync();
                let mut events = vec![PostCommandAction::FullRedrawRequired];
                events.extend(visibility_events);
                Ok(events)
            }
            "set wrap off" => {
                // Clear the command buffer and exit command mode
                context.app_state.clear_ex_command_buffer();
                let previous_mode = context.app_state.get_previous_mode();
                context.app_state.change_mode(previous_mode)?;

                // Disable word wrap
                context.app_state.pane_manager.set_wrap_enabled(false);
                let visibility_events = context
                    .app_state
                    .pane_manager
                    .rebuild_display_caches_and_sync();
                let mut events = vec![PostCommandAction::FullRedrawRequired];
                events.extend(visibility_events);
                Ok(events)
            }
            "set wrap!" => {
                // Clear the command buffer and exit command mode
                context.app_state.clear_ex_command_buffer();
                let previous_mode = context.app_state.get_previous_mode();
                context.app_state.change_mode(previous_mode)?;

                // Toggle word wrap state
                let current_wrap = context.app_state.pane_manager.is_wrap_enabled();
                context
                    .app_state
                    .pane_manager
                    .set_wrap_enabled(!current_wrap);
                let visibility_events = context
                    .app_state
                    .pane_manager
                    .rebuild_display_caches_and_sync();
                let mut events = vec![PostCommandAction::FullRedrawRequired];
                events.extend(visibility_events);
                Ok(events)
            }
            _ => {
                // This shouldn't happen given our is_relevant check, but handle gracefully
                tracing::warn!(
                    "ExSetWrapCommand executed with unexpected buffer: {}",
                    command
                );
                Ok(vec![])
            }
        }
    }

    fn name(&self) -> &'static str {
        "ExSetWrapCommand"
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
    fn test_is_relevant_with_set_wrap_on() {
        let command = ExSetWrapCommand;
        let context = create_test_context("set wrap on");
        let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);

        assert!(command.is_relevant(key_event, EditorMode::Command, &context));
    }

    #[test]
    fn test_is_relevant_with_set_wrap_off() {
        let command = ExSetWrapCommand;
        let context = create_test_context("set wrap off");
        let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);

        assert!(command.is_relevant(key_event, EditorMode::Command, &context));
    }

    #[test]
    fn test_is_relevant_with_set_wrap_toggle() {
        let command = ExSetWrapCommand;
        let context = create_test_context("set wrap!");
        let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);

        assert!(command.is_relevant(key_event, EditorMode::Command, &context));
    }

    #[test]
    fn test_is_relevant_with_whitespace() {
        let command = ExSetWrapCommand;
        let context = create_test_context("  set wrap on  ");
        let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);

        assert!(command.is_relevant(key_event, EditorMode::Command, &context));
    }

    #[test]
    fn test_is_not_relevant_with_different_command() {
        let command = ExSetWrapCommand;
        let context = create_test_context("set number on");
        let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);

        assert!(!command.is_relevant(key_event, EditorMode::Command, &context));
    }

    #[test]
    fn test_is_not_relevant_with_wrong_key() {
        let command = ExSetWrapCommand;
        let context = create_test_context("set wrap on");
        let key_event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);

        assert!(!command.is_relevant(key_event, EditorMode::Command, &context));
    }

    #[test]
    fn test_is_not_relevant_with_wrong_mode() {
        let command = ExSetWrapCommand;
        let context = create_test_context("set wrap on");
        let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);

        assert!(!command.is_relevant(key_event, EditorMode::Normal, &context));
    }

    #[test]
    fn test_is_not_relevant_with_invalid_wrap_command() {
        let command = ExSetWrapCommand;
        let test_cases = vec!["set wrap", "set wrap toggle", "wrap on", "set line_wrap on"];

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
        let command = ExSetWrapCommand;
        assert_eq!(command.name(), "ExSetWrapCommand");
    }
}

// Register the command
inventory::submit!(
    crate::repl::view_models::commands::dynamic_registry::CommandEntry {
        name: "ExSetWrapCommand",
        factory: || Box::new(ExSetWrapCommand),
    }
);
