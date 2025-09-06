//! # End of Line Command
//!
//! Command for moving cursor to the end of the current line ($ key).
//! This follows the unified command architecture where the Command owns its
//! business logic and emits appropriate PostCommandActions.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use crate::register_command;
use crate::repl::models::pane_state::EditorMode;
use crate::repl::unified_commands::{Command, CommandContext, ExecutionContext};
use crate::repl::view_models::post_command_actions::PostCommandAction;

/// Command to move cursor to the end of the current line
///
/// This command demonstrates the new architecture:
/// 1. Checks for '$' key in Normal/Visual modes
/// 2. Calls the pane manager to move cursor to end of line
/// 3. Emits appropriate PostCommandActions to update the display
///
/// Handles Vim-style '$' navigation for moving to the end of line.
pub struct EndOfLineCommand;

impl EndOfLineCommand {
    /// Create new EndOfLineCommand
    pub fn new() -> Self {
        Self
    }

    /// Check if the key event is relevant for moving to end of line
    fn is_end_of_line_key(key_event: KeyEvent) -> bool {
        match key_event.code {
            // vim-style '$' key without modifiers
            KeyCode::Char('$') => key_event.modifiers.is_empty(),
            _ => false,
        }
    }

    /// Check if current mode allows line navigation
    fn is_navigation_mode(mode: EditorMode) -> bool {
        matches!(
            mode,
            EditorMode::Normal
                | EditorMode::Visual
                | EditorMode::VisualLine
                | EditorMode::VisualBlock
        )
    }
}

impl Command for EndOfLineCommand {
    fn is_relevant(
        &self,
        key_event: KeyEvent,
        mode: EditorMode,
        _context: &CommandContext,
    ) -> bool {
        // Check if it's an end of line key and we're in a navigation mode
        Self::is_end_of_line_key(key_event) && Self::is_navigation_mode(mode)
    }

    fn execute(
        &self,
        _key_event: KeyEvent,
        context: &mut ExecutionContext,
    ) -> Result<Vec<PostCommandAction>> {
        // Get the cursor movement events from pane manager
        let events = context.app_state.pane_manager.move_cursor_to_end_of_line();

        tracing::debug!(
            "EndOfLineCommand executed, generated {} events",
            events.len()
        );

        Ok(events)
    }

    fn name(&self) -> &'static str {
        "EndOfLineCommand"
    }
}

impl Default for EndOfLineCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyModifiers;

    #[test]
    fn end_of_line_command_should_return_correct_name() {
        let command = EndOfLineCommand::new();
        assert_eq!(command.name(), "EndOfLineCommand");
    }

    #[test]
    fn end_of_line_command_should_be_relevant_for_dollar_key_in_normal_mode() {
        let command = EndOfLineCommand::new();
        let key_event = KeyEvent::new(KeyCode::Char('$'), KeyModifiers::empty());
        let mode = EditorMode::Normal;
        let context = CommandContext::test_default();

        assert!(command.is_relevant(key_event, mode, &context));
    }

    #[test]
    fn end_of_line_command_should_be_relevant_for_dollar_key_in_visual_mode() {
        let command = EndOfLineCommand::new();
        let key_event = KeyEvent::new(KeyCode::Char('$'), KeyModifiers::empty());
        let mode = EditorMode::Visual;
        let context = CommandContext::test_default();

        assert!(command.is_relevant(key_event, mode, &context));
    }

    #[test]
    fn end_of_line_command_should_be_relevant_for_dollar_key_in_visual_line_mode() {
        let command = EndOfLineCommand::new();
        let key_event = KeyEvent::new(KeyCode::Char('$'), KeyModifiers::empty());
        let mode = EditorMode::VisualLine;
        let context = CommandContext::test_default();

        assert!(command.is_relevant(key_event, mode, &context));
    }

    #[test]
    fn end_of_line_command_should_be_relevant_for_dollar_key_in_visual_block_mode() {
        let command = EndOfLineCommand::new();
        let key_event = KeyEvent::new(KeyCode::Char('$'), KeyModifiers::empty());
        let mode = EditorMode::VisualBlock;
        let context = CommandContext::test_default();

        assert!(command.is_relevant(key_event, mode, &context));
    }

    #[test]
    fn end_of_line_command_should_not_be_relevant_for_dollar_key_in_insert_mode() {
        let command = EndOfLineCommand::new();
        let key_event = KeyEvent::new(KeyCode::Char('$'), KeyModifiers::empty());
        let mode = EditorMode::Insert;
        let context = CommandContext::test_default();

        assert!(!command.is_relevant(key_event, mode, &context));
    }

    #[test]
    fn end_of_line_command_should_not_be_relevant_for_dollar_key_in_command_mode() {
        let command = EndOfLineCommand::new();
        let key_event = KeyEvent::new(KeyCode::Char('$'), KeyModifiers::empty());
        let mode = EditorMode::Command;
        let context = CommandContext::test_default();

        assert!(!command.is_relevant(key_event, mode, &context));
    }

    #[test]
    fn end_of_line_command_should_not_be_relevant_for_dollar_key_with_modifiers() {
        let command = EndOfLineCommand::new();
        let key_event = KeyEvent::new(KeyCode::Char('$'), KeyModifiers::SHIFT);
        let mode = EditorMode::Normal;
        let context = CommandContext::test_default();

        assert!(!command.is_relevant(key_event, mode, &context));
    }

    #[test]
    fn end_of_line_command_should_not_be_relevant_for_other_keys() {
        let command = EndOfLineCommand::new();
        let key_event = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::empty());
        let mode = EditorMode::Normal;
        let context = CommandContext::test_default();

        assert!(!command.is_relevant(key_event, mode, &context));
    }

    #[test]
    fn end_of_line_command_should_not_be_relevant_for_dollar_key_in_g_prefix_mode() {
        let command = EndOfLineCommand::new();
        let key_event = KeyEvent::new(KeyCode::Char('$'), KeyModifiers::empty());
        let mode = EditorMode::GPrefix;
        let context = CommandContext::test_default();

        assert!(!command.is_relevant(key_event, mode, &context));
    }

    #[test]
    fn end_of_line_command_should_not_be_relevant_for_dollar_key_in_visual_block_insert_mode() {
        let command = EndOfLineCommand::new();
        let key_event = KeyEvent::new(KeyCode::Char('$'), KeyModifiers::empty());
        let mode = EditorMode::VisualBlockInsert;
        let context = CommandContext::test_default();

        assert!(!command.is_relevant(key_event, mode, &context));
    }

    #[test]
    fn is_end_of_line_key_should_detect_dollar_key() {
        let key_event = KeyEvent::new(KeyCode::Char('$'), KeyModifiers::empty());
        assert!(EndOfLineCommand::is_end_of_line_key(key_event));
    }

    #[test]
    fn is_end_of_line_key_should_not_detect_dollar_key_with_modifiers() {
        let key_event = KeyEvent::new(KeyCode::Char('$'), KeyModifiers::CONTROL);
        assert!(!EndOfLineCommand::is_end_of_line_key(key_event));
    }

    #[test]
    fn is_end_of_line_key_should_not_detect_other_keys() {
        let key_event = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::empty());
        assert!(!EndOfLineCommand::is_end_of_line_key(key_event));
    }

    #[test]
    fn is_navigation_mode_should_detect_normal_mode() {
        assert!(EndOfLineCommand::is_navigation_mode(EditorMode::Normal));
    }

    #[test]
    fn is_navigation_mode_should_detect_visual_mode() {
        assert!(EndOfLineCommand::is_navigation_mode(EditorMode::Visual));
    }

    #[test]
    fn is_navigation_mode_should_detect_visual_line_mode() {
        assert!(EndOfLineCommand::is_navigation_mode(EditorMode::VisualLine));
    }

    #[test]
    fn is_navigation_mode_should_detect_visual_block_mode() {
        assert!(EndOfLineCommand::is_navigation_mode(
            EditorMode::VisualBlock
        ));
    }

    #[test]
    fn is_navigation_mode_should_not_detect_insert_mode() {
        assert!(!EndOfLineCommand::is_navigation_mode(EditorMode::Insert));
    }

    #[test]
    fn is_navigation_mode_should_not_detect_command_mode() {
        assert!(!EndOfLineCommand::is_navigation_mode(EditorMode::Command));
    }

    #[test]
    fn default_should_create_new_command() {
        let command = EndOfLineCommand;
        assert_eq!(command.name(), "EndOfLineCommand");
    }

    // Integration test placeholder - will be tested via integration tests
    #[test]
    fn end_of_line_command_integration_test_placeholder() {
        // This test serves as a placeholder for integration testing
        // Integration tests will verify the actual cursor movement behavior
        let command = EndOfLineCommand::new();
        assert_eq!(command.name(), "EndOfLineCommand");
    }
}

// Register the command for dynamic discovery
register_command!(EndOfLineCommand, "EndOfLineCommand");
