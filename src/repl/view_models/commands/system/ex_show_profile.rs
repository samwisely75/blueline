//! # Ex Show Profile Command
//!
//! Handles the `:show profile` ex command for displaying current profile information.
//! This command triggers the existing ShowProfileCommand to display profile details in the status bar.

use crate::repl::models::pane_state::EditorMode;
use crate::repl::view_models::commands::{Command, CommandContext, ExecutionContext};
use crate::repl::view_models::post_command_actions::PostCommandAction;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

/// Command for handling the `:show profile` ex command
pub struct ExShowProfileCommand;

impl Command for ExShowProfileCommand {
    fn is_relevant(&self, key_event: KeyEvent, mode: EditorMode, context: &CommandContext) -> bool {
        // Check if Enter key is pressed in Command mode
        if key_event.code != KeyCode::Enter || mode != EditorMode::Command {
            return false;
        }

        // Check if ex command buffer contains show profile command
        let buffer = context.ex_command_buffer.trim();
        buffer == "show profile"
    }

    fn execute(
        &self,
        _key_event: KeyEvent,
        context: &mut ExecutionContext,
    ) -> Result<Vec<PostCommandAction>> {
        let command = context.app_state.get_ex_command_buffer().trim();

        if command == "show profile" {
            // Clear the command buffer and exit command mode
            context.app_state.clear_ex_command_buffer();
            let previous_mode = context.app_state.get_previous_mode();
            context.app_state.change_mode(previous_mode)?;

            // Get profile information from app state (same logic as ShowProfileCommand)
            let profile_name = context.app_state.get_profile_name();
            let profile_path = context.app_state.get_profile_path();

            // Log first to release the borrows
            tracing::info!(
                "Showing profile via ex command: {} at {}",
                profile_name,
                profile_path
            );

            // Format and set status message
            let message = format!("[{profile_name}] in {profile_path}");
            context.app_state.set_status_message(message);

            // Return action for status bar update
            Ok(vec![PostCommandAction::StatusBarUpdateRequired])
        } else {
            // This shouldn't happen given our is_relevant check, but handle gracefully
            tracing::warn!(
                "ExShowProfileCommand executed with unexpected buffer: {}",
                command
            );
            Ok(vec![])
        }
    }

    fn name(&self) -> &'static str {
        "ExShowProfileCommand"
    }
}

// Register the command using inventory system
inventory::submit!(
    crate::repl::view_models::commands::dynamic_registry::CommandEntry {
        name: "ExShowProfileCommand",
        factory: || Box::new(ExShowProfileCommand),
    }
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::models::{pane_state::Pane, AppState};
    use crate::repl::services::Services;
    use crossterm::event::{KeyCode, KeyModifiers};

    #[test]
    fn ex_show_profile_command_should_return_correct_name() {
        let command = ExShowProfileCommand;
        assert_eq!(command.name(), "ExShowProfileCommand");
    }

    #[test]
    fn ex_show_profile_command_should_be_relevant_for_show_profile_in_command_mode() {
        let command = ExShowProfileCommand;
        let context = CommandContext {
            current_mode: EditorMode::Command,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: "show profile".to_string(),
        };

        let enter_key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert!(command.is_relevant(enter_key, EditorMode::Command, &context));
    }

    #[test]
    fn ex_show_profile_command_should_not_be_relevant_for_other_commands() {
        let command = ExShowProfileCommand;
        let context = CommandContext {
            current_mode: EditorMode::Command,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: "q".to_string(),
        };

        let enter_key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert!(!command.is_relevant(enter_key, EditorMode::Command, &context));
    }

    #[test]
    fn ex_show_profile_command_should_not_be_relevant_for_non_enter_keys() {
        let command = ExShowProfileCommand;
        let context = CommandContext {
            current_mode: EditorMode::Command,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: "show profile".to_string(),
        };

        let other_key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        assert!(!command.is_relevant(other_key, EditorMode::Command, &context));
    }

    #[test]
    fn ex_show_profile_command_should_not_be_relevant_in_normal_mode() {
        let command = ExShowProfileCommand;
        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: "show profile".to_string(),
        };

        let enter_key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert!(!command.is_relevant(enter_key, EditorMode::Normal, &context));
    }

    #[test]
    fn ex_show_profile_command_should_execute_successfully() {
        let command = ExShowProfileCommand;
        let mut app_state = AppState::new();
        let mut services = Services::new();

        // Set up ex command buffer
        app_state.set_ex_command_buffer("show profile".to_string());

        let mut context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        // Execute the command
        let result = command.execute(
            KeyEvent::new(KeyCode::Null, KeyModifiers::empty()),
            &mut context,
        );

        assert!(result.is_ok());
        let actions = result.unwrap();
        assert_eq!(actions.len(), 1);
        assert!(matches!(
            actions[0],
            PostCommandAction::StatusBarUpdateRequired
        ));

        // Verify command buffer was cleared
        assert!(context.app_state.get_ex_command_buffer().is_empty());
    }

    #[test]
    fn ex_show_profile_command_should_handle_unexpected_buffer_gracefully() {
        let command = ExShowProfileCommand;
        let mut app_state = AppState::new();
        let mut services = Services::new();

        // Set unexpected command buffer
        app_state.set_ex_command_buffer("unexpected".to_string());

        let mut context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        // Execute the command
        let result = command.execute(
            KeyEvent::new(KeyCode::Null, KeyModifiers::empty()),
            &mut context,
        );

        assert!(result.is_ok());
        let actions = result.unwrap();
        assert_eq!(actions.len(), 0); // No actions for unexpected buffer
    }

    #[test]
    fn ex_show_profile_command_should_trim_buffer_content() {
        let command = ExShowProfileCommand;
        let context = CommandContext {
            current_mode: EditorMode::Command,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: "  show profile  ".to_string(), // With whitespace
        };

        let enter_key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert!(command.is_relevant(enter_key, EditorMode::Command, &context));
    }
}
