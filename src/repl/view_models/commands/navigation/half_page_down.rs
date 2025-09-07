//! # Half Page Down Command
//!
//! Command for moving cursor down half a page (Ctrl+D key).
//! This implements the vim-style half-page-down navigation that moves the cursor
//! down by half the height of the viewport.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::register_command;
use crate::repl::models::pane_state::EditorMode;
use crate::repl::view_models::commands::{Command, CommandContext, ExecutionContext};
use crate::repl::view_models::post_command_actions::PostCommandAction;

/// Command to move cursor down half a page (Ctrl+D)
///
/// This command implements vim-style half page navigation:
/// 1. Checks for Ctrl+D key combination in navigation modes
/// 2. Calls the pane manager to move cursor down by half viewport height
/// 3. Returns appropriate PostCommandActions to update the display
///
/// Only works in navigation modes (Normal, Visual, VisualLine, VisualBlock).
pub struct HalfPageDownCommand;

impl HalfPageDownCommand {
    /// Create new HalfPageDownCommand
    pub fn new() -> Self {
        Self
    }

    /// Check if the key event is Ctrl+D for half page down navigation
    fn is_half_page_down_key(key_event: KeyEvent) -> bool {
        matches!(key_event.code, KeyCode::Char('d'))
            && key_event.modifiers.contains(KeyModifiers::CONTROL)
            && !key_event.modifiers.contains(KeyModifiers::SHIFT)
            && !key_event.modifiers.contains(KeyModifiers::ALT)
    }

    /// Check if current mode allows navigation
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

impl Command for HalfPageDownCommand {
    fn is_relevant(
        &self,
        key_event: KeyEvent,
        mode: EditorMode,
        _context: &CommandContext,
    ) -> bool {
        // Must be Ctrl+D key combination
        if !Self::is_half_page_down_key(key_event) {
            return false;
        }

        // Must be in a navigation mode
        let is_relevant = Self::is_navigation_mode(mode);

        // Add debug logging like the legacy command
        tracing::debug!(
            "HalfPageDownCommand.is_relevant(): ctrl+d={}, mode={:?}, result={}",
            true, // We already validated it's ctrl+d above
            mode,
            is_relevant
        );

        is_relevant
    }

    fn execute(
        &self,
        _key_event: KeyEvent,
        context: &mut ExecutionContext,
    ) -> Result<Vec<PostCommandAction>> {
        // Use the pane manager's half page down method which returns PostCommandActions
        context.app_state.pane_manager.move_cursor_half_page_down();

        tracing::debug!("HalfPageDownCommand executed, generated {} events", 0);

        Ok(vec![
            PostCommandAction::ActiveCursorUpdateRequired,
            PostCommandAction::PositionIndicatorUpdateRequired,
            PostCommandAction::CurrentAreaRedrawRequired,
        ])
    }

    fn name(&self) -> &'static str {
        "HalfPageDownCommand"
    }
}

impl Default for HalfPageDownCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn test_name() {
        let command = HalfPageDownCommand::new();
        assert_eq!(command.name(), "HalfPageDownCommand");
    }

    #[test]
    fn test_is_relevant_with_ctrl_d_in_normal_mode() {
        let command = HalfPageDownCommand::new();
        let key_event = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL);
        let context = CommandContext::test_default();

        assert!(command.is_relevant(key_event, EditorMode::Normal, &context));
    }

    #[test]
    fn test_is_relevant_with_ctrl_d_in_visual_mode() {
        let command = HalfPageDownCommand::new();
        let key_event = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL);
        let context = CommandContext::test_default();

        assert!(command.is_relevant(key_event, EditorMode::Visual, &context));
    }

    #[test]
    fn test_is_relevant_with_ctrl_d_in_visual_line_mode() {
        let command = HalfPageDownCommand::new();
        let key_event = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL);
        let context = CommandContext::test_default();

        assert!(command.is_relevant(key_event, EditorMode::VisualLine, &context));
    }

    #[test]
    fn test_is_relevant_with_ctrl_d_in_visual_block_mode() {
        let command = HalfPageDownCommand::new();
        let key_event = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL);
        let context = CommandContext::test_default();

        assert!(command.is_relevant(key_event, EditorMode::VisualBlock, &context));
    }

    #[test]
    fn test_is_not_relevant_with_ctrl_d_in_insert_mode() {
        let command = HalfPageDownCommand::new();
        let key_event = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL);
        let context = CommandContext::test_default();

        assert!(!command.is_relevant(key_event, EditorMode::Insert, &context));
    }

    #[test]
    fn test_is_not_relevant_with_ctrl_d_in_command_mode() {
        let command = HalfPageDownCommand::new();
        let key_event = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL);
        let context = CommandContext::test_default();

        assert!(!command.is_relevant(key_event, EditorMode::Command, &context));
    }

    #[test]
    fn test_is_not_relevant_with_plain_d_in_normal_mode() {
        let command = HalfPageDownCommand::new();
        let key_event = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::empty());
        let context = CommandContext::test_default();

        assert!(!command.is_relevant(key_event, EditorMode::Normal, &context));
    }

    #[test]
    fn test_is_not_relevant_with_ctrl_shift_d_in_normal_mode() {
        let command = HalfPageDownCommand::new();
        let key_event = KeyEvent::new(
            KeyCode::Char('d'),
            KeyModifiers::CONTROL | KeyModifiers::SHIFT,
        );
        let context = CommandContext::test_default();

        assert!(!command.is_relevant(key_event, EditorMode::Normal, &context));
    }

    #[test]
    fn test_is_not_relevant_with_ctrl_alt_d_in_normal_mode() {
        let command = HalfPageDownCommand::new();
        let key_event = KeyEvent::new(
            KeyCode::Char('d'),
            KeyModifiers::CONTROL | KeyModifiers::ALT,
        );
        let context = CommandContext::test_default();

        assert!(!command.is_relevant(key_event, EditorMode::Normal, &context));
    }

    #[test]
    fn test_is_not_relevant_with_different_key_in_normal_mode() {
        let command = HalfPageDownCommand::new();
        let key_event = KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL);
        let context = CommandContext::test_default();

        assert!(!command.is_relevant(key_event, EditorMode::Normal, &context));
    }

    #[test]
    fn test_is_half_page_down_key_with_ctrl_d() {
        let key_event = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL);
        assert!(HalfPageDownCommand::is_half_page_down_key(key_event));
    }

    #[test]
    fn test_is_half_page_down_key_with_plain_d() {
        let key_event = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::empty());
        assert!(!HalfPageDownCommand::is_half_page_down_key(key_event));
    }

    #[test]
    fn test_is_half_page_down_key_with_ctrl_shift_d() {
        let key_event = KeyEvent::new(
            KeyCode::Char('d'),
            KeyModifiers::CONTROL | KeyModifiers::SHIFT,
        );
        assert!(!HalfPageDownCommand::is_half_page_down_key(key_event));
    }

    #[test]
    fn test_is_half_page_down_key_with_ctrl_alt_d() {
        let key_event = KeyEvent::new(
            KeyCode::Char('d'),
            KeyModifiers::CONTROL | KeyModifiers::ALT,
        );
        assert!(!HalfPageDownCommand::is_half_page_down_key(key_event));
    }

    #[test]
    fn test_is_navigation_mode_normal() {
        assert!(HalfPageDownCommand::is_navigation_mode(EditorMode::Normal));
    }

    #[test]
    fn test_is_navigation_mode_visual() {
        assert!(HalfPageDownCommand::is_navigation_mode(EditorMode::Visual));
    }

    #[test]
    fn test_is_navigation_mode_visual_line() {
        assert!(HalfPageDownCommand::is_navigation_mode(
            EditorMode::VisualLine
        ));
    }

    #[test]
    fn test_is_navigation_mode_visual_block() {
        assert!(HalfPageDownCommand::is_navigation_mode(
            EditorMode::VisualBlock
        ));
    }

    #[test]
    fn test_is_not_navigation_mode_insert() {
        assert!(!HalfPageDownCommand::is_navigation_mode(EditorMode::Insert));
    }

    #[test]
    fn test_is_not_navigation_mode_command() {
        assert!(!HalfPageDownCommand::is_navigation_mode(
            EditorMode::Command
        ));
    }

    #[test]
    fn test_default_instance() {
        let command = HalfPageDownCommand;
        assert_eq!(command.name(), "HalfPageDownCommand");
    }

    // Integration test placeholder - actual execution would require full context setup
    // This test would verify that the command properly calls pane_manager.move_cursor_half_page_down()
    // and returns the correct PostCommandActions
    #[test]
    fn test_execute_integration() {
        // Note: This is a placeholder for integration testing
        // Full integration test would require setting up ExecutionContext with proper app_state
        let command = HalfPageDownCommand::new();
        assert_eq!(command.name(), "HalfPageDownCommand");
    }
}

// Register command using the dynamic discovery system for zero-conflict migration
register_command!(HalfPageDownCommand, "HalfPageDownCommand");
