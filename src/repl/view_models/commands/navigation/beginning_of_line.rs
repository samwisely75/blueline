//! # Beginning of Line Command
//!
//! Command for moving cursor to the beginning of the current line (0 key).
//! This follows the unified command architecture where the Command owns its
//! business logic and emits appropriate PostCommandActions.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use crate::register_command;
use crate::repl::models::pane_state::EditorMode;
use crate::repl::view_models::commands::{Command, CommandContext, ExecutionContext};
use crate::repl::view_models::post_command_actions::PostCommandAction;

/// Command to move cursor to the beginning of the current line
///
/// This command demonstrates the new architecture:
/// 1. Checks for '0' key in Normal/Visual modes
/// 2. Calls the pane manager to move cursor to start of line
/// 3. Emits appropriate PostCommandActions to update the display
///
/// Handles Vim-style '0' navigation for moving to column 0.
pub struct BeginningOfLineCommand;

impl BeginningOfLineCommand {
    /// Create new BeginningOfLineCommand
    pub fn new() -> Self {
        Self
    }

    /// Check if the key event is relevant for moving to beginning of line
    fn is_beginning_of_line_key(key_event: KeyEvent) -> bool {
        match key_event.code {
            // vim-style '0' key without modifiers
            KeyCode::Char('0') => key_event.modifiers.is_empty(),
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

impl Command for BeginningOfLineCommand {
    fn is_relevant(
        &self,
        key_event: KeyEvent,
        mode: EditorMode,
        _context: &CommandContext,
    ) -> bool {
        // Check if it's a beginning of line key and we're in a navigation mode
        Self::is_beginning_of_line_key(key_event) && Self::is_navigation_mode(mode)
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

        tracing::debug!("BeginningOfLineCommand executed");

        // Generate view update events based on what this command did
        Ok(vec![
            PostCommandAction::ActiveCursorUpdateRequired,
            PostCommandAction::PositionIndicatorUpdateRequired,
            PostCommandAction::CurrentAreaRedrawRequired,
        ])
    }

    fn name(&self) -> &'static str {
        "BeginningOfLineCommand"
    }
}

impl Default for BeginningOfLineCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyModifiers;

    #[test]
    fn beginning_of_line_command_should_return_correct_name() {
        let command = BeginningOfLineCommand::new();
        assert_eq!(command.name(), "BeginningOfLineCommand");
    }

    #[test]
    fn beginning_of_line_command_should_be_relevant_for_zero_key_in_normal_mode() {
        let command = BeginningOfLineCommand::new();
        let key_event = KeyEvent::new(KeyCode::Char('0'), KeyModifiers::empty());
        let mode = EditorMode::Normal;
        let context = CommandContext::test_default();

        assert!(command.is_relevant(key_event, mode, &context));
    }

    #[test]
    fn beginning_of_line_command_should_be_relevant_for_zero_key_in_visual_mode() {
        let command = BeginningOfLineCommand::new();
        let key_event = KeyEvent::new(KeyCode::Char('0'), KeyModifiers::empty());
        let mode = EditorMode::Visual;
        let context = CommandContext::test_default();

        assert!(command.is_relevant(key_event, mode, &context));
    }

    #[test]
    fn beginning_of_line_command_should_be_relevant_for_zero_key_in_visual_line_mode() {
        let command = BeginningOfLineCommand::new();
        let key_event = KeyEvent::new(KeyCode::Char('0'), KeyModifiers::empty());
        let mode = EditorMode::VisualLine;
        let context = CommandContext::test_default();

        assert!(command.is_relevant(key_event, mode, &context));
    }

    #[test]
    fn beginning_of_line_command_should_be_relevant_for_zero_key_in_visual_block_mode() {
        let command = BeginningOfLineCommand::new();
        let key_event = KeyEvent::new(KeyCode::Char('0'), KeyModifiers::empty());
        let mode = EditorMode::VisualBlock;
        let context = CommandContext::test_default();

        assert!(command.is_relevant(key_event, mode, &context));
    }

    #[test]
    fn beginning_of_line_command_should_not_be_relevant_for_zero_key_in_insert_mode() {
        let command = BeginningOfLineCommand::new();
        let key_event = KeyEvent::new(KeyCode::Char('0'), KeyModifiers::empty());
        let mode = EditorMode::Insert;
        let context = CommandContext::test_default();

        assert!(!command.is_relevant(key_event, mode, &context));
    }

    #[test]
    fn beginning_of_line_command_should_not_be_relevant_for_zero_key_in_command_mode() {
        let command = BeginningOfLineCommand::new();
        let key_event = KeyEvent::new(KeyCode::Char('0'), KeyModifiers::empty());
        let mode = EditorMode::Command;
        let context = CommandContext::test_default();

        assert!(!command.is_relevant(key_event, mode, &context));
    }

    #[test]
    fn beginning_of_line_command_should_not_be_relevant_for_zero_key_with_modifiers() {
        let command = BeginningOfLineCommand::new();
        let key_event = KeyEvent::new(KeyCode::Char('0'), KeyModifiers::CONTROL);
        let mode = EditorMode::Normal;
        let context = CommandContext::test_default();

        assert!(!command.is_relevant(key_event, mode, &context));
    }

    #[test]
    fn beginning_of_line_command_should_not_be_relevant_for_other_keys() {
        let command = BeginningOfLineCommand::new();
        let key_event = KeyEvent::new(KeyCode::Char('1'), KeyModifiers::empty());
        let mode = EditorMode::Normal;
        let context = CommandContext::test_default();

        assert!(!command.is_relevant(key_event, mode, &context));
    }

    #[test]
    fn beginning_of_line_command_should_not_be_relevant_for_zero_key_in_g_prefix_mode() {
        let command = BeginningOfLineCommand::new();
        let key_event = KeyEvent::new(KeyCode::Char('0'), KeyModifiers::empty());
        let mode = EditorMode::GPrefix;
        let context = CommandContext::test_default();

        assert!(!command.is_relevant(key_event, mode, &context));
    }

    #[test]
    fn beginning_of_line_command_should_not_be_relevant_for_zero_key_in_visual_block_insert_mode() {
        let command = BeginningOfLineCommand::new();
        let key_event = KeyEvent::new(KeyCode::Char('0'), KeyModifiers::empty());
        let mode = EditorMode::VisualBlockInsert;
        let context = CommandContext::test_default();

        assert!(!command.is_relevant(key_event, mode, &context));
    }

    #[test]
    fn is_beginning_of_line_key_should_detect_zero_key() {
        let key_event = KeyEvent::new(KeyCode::Char('0'), KeyModifiers::empty());
        assert!(BeginningOfLineCommand::is_beginning_of_line_key(key_event));
    }

    #[test]
    fn is_beginning_of_line_key_should_not_detect_zero_key_with_modifiers() {
        let key_event = KeyEvent::new(KeyCode::Char('0'), KeyModifiers::SHIFT);
        assert!(!BeginningOfLineCommand::is_beginning_of_line_key(key_event));
    }

    #[test]
    fn is_beginning_of_line_key_should_not_detect_other_keys() {
        let key_event = KeyEvent::new(KeyCode::Char('o'), KeyModifiers::empty());
        assert!(!BeginningOfLineCommand::is_beginning_of_line_key(key_event));
    }

    #[test]
    fn is_navigation_mode_should_detect_normal_mode() {
        assert!(BeginningOfLineCommand::is_navigation_mode(
            EditorMode::Normal
        ));
    }

    #[test]
    fn is_navigation_mode_should_detect_visual_mode() {
        assert!(BeginningOfLineCommand::is_navigation_mode(
            EditorMode::Visual
        ));
    }

    #[test]
    fn is_navigation_mode_should_detect_visual_line_mode() {
        assert!(BeginningOfLineCommand::is_navigation_mode(
            EditorMode::VisualLine
        ));
    }

    #[test]
    fn is_navigation_mode_should_detect_visual_block_mode() {
        assert!(BeginningOfLineCommand::is_navigation_mode(
            EditorMode::VisualBlock
        ));
    }

    #[test]
    fn is_navigation_mode_should_not_detect_insert_mode() {
        assert!(!BeginningOfLineCommand::is_navigation_mode(
            EditorMode::Insert
        ));
    }

    #[test]
    fn is_navigation_mode_should_not_detect_command_mode() {
        assert!(!BeginningOfLineCommand::is_navigation_mode(
            EditorMode::Command
        ));
    }

    #[test]
    fn default_should_create_new_command() {
        let command = BeginningOfLineCommand;
        assert_eq!(command.name(), "BeginningOfLineCommand");
    }

    // Integration test placeholder - will be tested via integration tests
    #[test]
    fn beginning_of_line_command_integration_test_placeholder() {
        // This test serves as a placeholder for integration testing
        // Integration tests will verify the actual cursor movement behavior
        let command = BeginningOfLineCommand::new();
        assert_eq!(command.name(), "BeginningOfLineCommand");
    }
}

// Register the command for dynamic discovery
register_command!(BeginningOfLineCommand, "BeginningOfLineCommand");
