//! # ExCommandMode Command
//!
//! Handles all input in Command mode (after ':' is pressed).
//!
//! This command processes:
//! - Character input → Build ex command buffer  
//! - Enter → Execute the ex command (handled by specific ex commands)
//! - Backspace → Delete from buffer
//! - Escape → Cancel and return to Normal mode
//!
//! This is the core command that enables all ex commands like :q, :w, :set, etc.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::register_command;
use crate::repl::models::pane_state::EditorMode;
use crate::repl::unified_commands::{Command, CommandContext, ExecutionContext};
use crate::repl::view_models::post_command_actions::PostCommandAction;

/// Handles all input in Command mode to build and manage ex command buffer
#[derive(Debug, Default)]
pub struct ExCommandModeCommand;

impl ExCommandModeCommand {
    pub fn new() -> Self {
        Self {}
    }
}

impl Command for ExCommandModeCommand {
    fn is_relevant(
        &self,
        key_event: KeyEvent,
        mode: EditorMode,
        _context: &CommandContext,
    ) -> bool {
        // Only relevant in Command mode for specific keys
        // Do NOT handle Enter - that should be handled by specific ex commands
        mode == EditorMode::Command
            && matches!(
                key_event.code,
                KeyCode::Char(_) | KeyCode::Backspace | KeyCode::Esc
            )
    }

    fn execute(
        &self,
        key_event: KeyEvent,
        context: &mut ExecutionContext,
    ) -> Result<Vec<PostCommandAction>> {
        match key_event.code {
            // Handle character input to build ex command buffer
            KeyCode::Char(ch) if key_event.modifiers == KeyModifiers::NONE => {
                context.app_state.add_ex_command_char(ch)?;
                // Update status bar to show the changing command buffer
                Ok(vec![PostCommandAction::StatusBarUpdateRequired])
            }

            // Handle backspace to delete from buffer
            KeyCode::Backspace => {
                context.app_state.backspace_ex_command()?;
                // Update status bar to reflect the shortened buffer
                Ok(vec![PostCommandAction::StatusBarUpdateRequired])
            }

            // Handle Escape to cancel and return to previous mode
            KeyCode::Esc => {
                // Clear the command buffer and exit command mode
                context.app_state.clear_ex_command_buffer();
                let previous_mode = context.app_state.get_previous_mode();
                context.app_state.change_mode(previous_mode)?;
                Ok(vec![
                    PostCommandAction::StatusBarUpdateRequired,
                    PostCommandAction::ActiveCursorUpdateRequired,
                ])
            }

            // Ignore other key combinations
            _ => Ok(vec![]),
        }
    }

    fn name(&self) -> &'static str {
        "ExCommandMode"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::models::pane_state::EditorMode;
    use crate::repl::models::pane_state::Pane;
    use crate::repl::unified_commands::CommandContext;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn create_test_context() -> CommandContext {
        CommandContext {
            current_mode: EditorMode::Command,
            current_pane: Pane::Request,
            has_selection: false,
            is_read_only: false,
            ex_command_buffer: String::new(),
        }
    }

    #[test]
    fn test_command_name() {
        let command = ExCommandModeCommand::new();
        assert_eq!(command.name(), "ExCommandMode");
    }

    #[test]
    fn test_default_instance() {
        let command = ExCommandModeCommand {};
        assert_eq!(command.name(), "ExCommandMode");
    }

    #[test]
    fn test_is_relevant_in_command_mode() {
        let command = ExCommandModeCommand::new();
        let context = create_test_context();
        let key_event = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);

        assert!(command.is_relevant(key_event, EditorMode::Command, &context));
    }

    #[test]
    fn test_is_not_relevant_in_normal_mode() {
        let command = ExCommandModeCommand::new();
        let context = create_test_context();
        let key_event = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);

        assert!(!command.is_relevant(key_event, EditorMode::Normal, &context));
    }

    #[test]
    fn test_is_not_relevant_in_insert_mode() {
        let command = ExCommandModeCommand::new();
        let context = create_test_context();
        let key_event = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);

        assert!(!command.is_relevant(key_event, EditorMode::Insert, &context));
    }

    #[test]
    fn test_is_not_relevant_in_visual_mode() {
        let command = ExCommandModeCommand::new();
        let context = create_test_context();
        let key_event = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);

        assert!(!command.is_relevant(key_event, EditorMode::Visual, &context));
    }

    #[test]
    fn test_is_relevant_for_character_keys() {
        let command = ExCommandModeCommand::new();
        let context = create_test_context();

        // Test different character keys
        let chars = ['q', 'w', 's', 'e', 't', '1', '!'];
        for ch in chars {
            let key_event = KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE);
            assert!(
                command.is_relevant(key_event, EditorMode::Command, &context),
                "Should be relevant for character '{ch}'"
            );
        }
    }

    #[test]
    fn test_is_relevant_for_special_keys() {
        let command = ExCommandModeCommand::new();
        let context = create_test_context();

        // Test special keys that should be handled (Enter is handled by specific ex commands)
        let special_keys = [KeyCode::Backspace, KeyCode::Esc];
        for key_code in special_keys {
            let key_event = KeyEvent::new(key_code, KeyModifiers::NONE);
            assert!(
                command.is_relevant(key_event, EditorMode::Command, &context),
                "Should be relevant for key {key_code:?}"
            );
        }

        // Enter should NOT be handled by this command
        let enter_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert!(
            !command.is_relevant(enter_event, EditorMode::Command, &context),
            "Should NOT be relevant for Enter key (handled by specific ex commands)"
        );
    }

    #[test]
    fn test_is_relevant_for_keys_with_modifiers() {
        let command = ExCommandModeCommand::new();
        let context = create_test_context();

        // Test keys with modifiers (should still be relevant in Command mode)
        let key_event = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL);
        assert!(command.is_relevant(key_event, EditorMode::Command, &context));

        let key_event = KeyEvent::new(KeyCode::Char('Q'), KeyModifiers::SHIFT);
        assert!(command.is_relevant(key_event, EditorMode::Command, &context));
    }

    // Note: Execute method tests would require setting up ExecutionContext with a mock app_state
    // These tests focus on the is_relevant logic which is the core of the command pattern

    #[test]
    fn test_character_input_relevance() {
        let command = ExCommandModeCommand::new();
        let context = create_test_context();

        // Test that character input is relevant in command mode
        let key_event = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        assert!(command.is_relevant(key_event, EditorMode::Command, &context));
    }

    #[test]
    fn test_backspace_relevance() {
        let command = ExCommandModeCommand::new();
        let context = create_test_context();

        // Test that backspace is relevant in command mode
        let key_event = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);
        assert!(command.is_relevant(key_event, EditorMode::Command, &context));
    }

    #[test]
    fn test_enter_relevance() {
        let command = ExCommandModeCommand::new();
        let context = create_test_context();

        // Test that enter is NOT relevant in command mode (handled by specific ex commands)
        let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert!(!command.is_relevant(key_event, EditorMode::Command, &context));
    }

    #[test]
    fn test_escape_relevance() {
        let command = ExCommandModeCommand::new();
        let context = create_test_context();

        // Test that escape is relevant in command mode
        let key_event = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        assert!(command.is_relevant(key_event, EditorMode::Command, &context));
    }

    #[test]
    fn test_command_mode_detection_helper() {
        let command = ExCommandModeCommand::new();
        let context = create_test_context();
        let key_event = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);

        // Test all modes
        assert!(command.is_relevant(key_event, EditorMode::Command, &context));
        assert!(!command.is_relevant(key_event, EditorMode::Normal, &context));
        assert!(!command.is_relevant(key_event, EditorMode::Insert, &context));
        assert!(!command.is_relevant(key_event, EditorMode::Visual, &context));
        assert!(!command.is_relevant(key_event, EditorMode::VisualLine, &context));
        assert!(!command.is_relevant(key_event, EditorMode::VisualBlock, &context));
    }

    // Integration test placeholder - would need full ExecutionContext setup
    #[test]
    fn test_integration_placeholder() {
        // TODO: Add integration tests that verify:
        // - Character input adds to ex command buffer
        // - Backspace removes from buffer
        // - Enter triggers ex command execution
        // - Escape clears buffer and exits command mode
        // These tests would need a full ExecutionContext with app_state
    }
}

// Register the command with the dynamic discovery system
register_command!(ExCommandModeCommand, "ExCommandMode");
