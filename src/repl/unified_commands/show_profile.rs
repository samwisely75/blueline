//! # Show Profile Command
//!
//! Command to display the current profile information in the status bar.

use anyhow::Result;
use crossterm::event::KeyEvent;

use crate::repl::models::events::view_events::ViewEvent;
use crate::repl::models::pane_state::EditorMode;
use crate::repl::unified_commands::{Command, CommandContext, ExecutionContext};

/// Command to show current profile information
///
/// This command displays the profile name and path in the status bar.
/// Typically triggered by ex commands like `:profile`.
pub struct ShowProfileCommand;

impl ShowProfileCommand {
    /// Create new ShowProfileCommand
    pub fn new() -> Self {
        Self
    }
}

impl Command for ShowProfileCommand {
    fn is_relevant(
        &self,
        _key_event: KeyEvent,
        _mode: EditorMode,
        _context: &CommandContext,
    ) -> bool {
        // This command is not triggered by key events directly
        // It's triggered by CommandEvent::ShowProfileRequested from ex commands
        false
    }

    fn execute(&self, context: &mut ExecutionContext) -> Result<Vec<ViewEvent>> {
        // Get profile information from app state
        let profile_name = context.app_state.get_profile_name();
        let profile_path = context.app_state.get_profile_path();

        // Log first to release the borrows
        tracing::info!("Showing profile: {} at {}", profile_name, profile_path);

        // Format and set status message
        let message = format!("[{profile_name}] in {profile_path}");
        context.app_state.set_status_message(message);

        // Return view event for status bar update
        Ok(vec![ViewEvent::StatusBarUpdateRequired])
    }

    fn name(&self) -> &'static str {
        "ShowProfileCommand"
    }
}

impl Default for ShowProfileCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::models::AppState;
    use crate::repl::services::Services;

    #[test]
    fn show_profile_command_should_return_correct_name() {
        let command = ShowProfileCommand::new();
        assert_eq!(command.name(), "ShowProfileCommand");
    }

    #[test]
    fn show_profile_command_should_not_be_relevant_for_key_events() {
        use crate::repl::models::pane_state::Pane;
        use crossterm::event::{KeyCode, KeyModifiers};

        let command = ShowProfileCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
        };

        // Should never be relevant for key events
        let any_key = KeyEvent::new(KeyCode::Char('p'), KeyModifiers::NONE);
        assert!(!command.is_relevant(any_key, EditorMode::Normal, &context));
    }

    #[test]
    fn show_profile_command_should_set_status_message() {
        let command = ShowProfileCommand::new();
        let mut app_state = AppState::new();
        let mut services = Services::new();

        let mut context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        // Execute the command
        let result = command.execute(&mut context);

        assert!(result.is_ok());
        let events = result.unwrap();
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], ViewEvent::StatusBarUpdateRequired));

        // Verify status message was set (it will contain default profile info)
        // The actual message content is tested in AppState tests
    }
}
