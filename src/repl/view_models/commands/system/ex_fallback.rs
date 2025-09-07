//! # Ex Fallback Command
//!
//! Handles any unrecognized ex commands by exiting command mode and returning to previous mode.
//! This ensures that pressing Enter with an unknown command doesn't leave the user stuck in Command mode.

use crate::repl::models::pane_state::EditorMode;
use crate::repl::view_models::commands::{Command, CommandContext, ExecutionContext};
use crate::repl::view_models::post_command_actions::PostCommandAction;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

/// Command for handling unrecognized ex commands
///
/// This is a fallback handler that:
/// 1. Only activates when Enter is pressed in Command mode
/// 2. Has lowest priority (checked after all specific ex commands)
/// 3. Clears the command buffer and returns to previous mode
/// 4. Logs a warning about the unknown command
pub struct ExFallbackCommand;

impl Command for ExFallbackCommand {
    fn is_relevant(&self, key_event: KeyEvent, mode: EditorMode, context: &CommandContext) -> bool {
        // Only handle Enter key in Command mode when the buffer doesn't match known commands
        if key_event.code != KeyCode::Enter || mode != EditorMode::Command {
            return false;
        }

        // Check if this is a known command - if so, don't handle it
        let buffer = context.ex_command_buffer.trim();

        // List of known ex commands that have their own handlers
        let known_commands = [
            "q",
            "q!",
            "quit",
            "quit!",
            "w",
            "w!",
            "write",
            "write!",
            "wq",
            "wq!",
            "set wrap on",
            "set wrap off",
            "set wrap!",
            "set number on",
            "set number off",
            "set number!",
            "set expandtab on",
            "set expandtab off",
            "set expandtab!",
            "set clipboard on",
            "set clipboard off",
            "set clipboard!",
            "set dcut on",
            "set dcut off",
            "set dcut!",
            "profile",
        ];

        // Also check for patterns like "set tabstop N" and goto line commands
        if buffer.starts_with("set tabstop ") {
            return false;
        }

        // Check for goto line commands (numbers or $)
        if buffer.chars().all(|c| c.is_ascii_digit()) || buffer == "$" {
            return false;
        }

        // Handle both unknown commands and empty buffer
        !known_commands.contains(&buffer)
    }

    fn execute(
        &self,
        _key_event: KeyEvent,
        context: &mut ExecutionContext,
    ) -> Result<Vec<PostCommandAction>> {
        let command = context.app_state.get_ex_command_buffer().trim();

        // Log the unknown command for debugging
        if !command.is_empty() {
            tracing::warn!("Unknown ex command: {}", command);
            // Don't set a status message for unknown commands - just clear the bar
        }

        // Clear the command buffer and exit command mode
        context.app_state.clear_ex_command_buffer();
        context.app_state.clear_status_message(); // Clear any existing status message
        let previous_mode = context.app_state.get_previous_mode();
        context.app_state.change_mode(previous_mode)?;

        Ok(vec![
            PostCommandAction::StatusBarUpdateRequired,
            PostCommandAction::FullRedrawRequired,
        ])
    }

    fn name(&self) -> &'static str {
        "ExFallbackCommand"
    }
}

// Register the command with very low priority
inventory::submit!(
    crate::repl::view_models::commands::dynamic_registry::CommandEntry {
        name: "ExFallbackCommand",
        factory: || Box::new(ExFallbackCommand),
    }
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::models::pane_state::Pane;
    use crossterm::event::KeyModifiers;

    #[test]
    fn fallback_command_should_be_relevant_for_enter_in_command_mode() {
        let command = ExFallbackCommand;
        let context = CommandContext {
            current_mode: EditorMode::Command,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: "unknown".to_string(),
        };

        let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert!(command.is_relevant(key_event, EditorMode::Command, &context));
    }

    #[test]
    fn fallback_command_should_not_be_relevant_in_normal_mode() {
        let command = ExFallbackCommand;
        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert!(!command.is_relevant(key_event, EditorMode::Normal, &context));
    }

    #[test]
    fn fallback_command_should_not_be_relevant_for_other_keys() {
        let command = ExFallbackCommand;
        let context = CommandContext {
            current_mode: EditorMode::Command,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: "unknown".to_string(),
        };

        let key_event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        assert!(!command.is_relevant(key_event, EditorMode::Command, &context));
    }

    #[test]
    fn fallback_command_should_return_correct_name() {
        let command = ExFallbackCommand;
        assert_eq!(command.name(), "ExFallbackCommand");
    }
}
