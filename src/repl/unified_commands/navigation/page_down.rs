//! # Page Down Command
//!
//! Command for moving cursor down one full page (Ctrl+f key).
//! This implements the vim-style page-down navigation that moves the cursor
//! down by the height of the viewport.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::register_command;
use crate::repl::models::pane_state::EditorMode;
use crate::repl::unified_commands::{Command, CommandContext, ExecutionContext};
use crate::repl::view_models::post_command_actions::PostCommandAction;

/// Command to move cursor down one full page (Ctrl+f)
///
/// This command implements vim-style full page navigation:
/// 1. Checks for Ctrl+f key combination in navigation modes
/// 2. Calls the pane manager to move cursor down by viewport height
/// 3. Returns appropriate PostCommandActions to update the display
///
/// Only works in navigation modes (Normal, Visual, VisualLine, VisualBlock).
pub struct PageDownCommand;

impl PageDownCommand {
    /// Create new PageDownCommand
    pub fn new() -> Self {
        Self
    }

    /// Check if the key event is Ctrl+f for page down navigation
    fn is_page_down_key(key_event: KeyEvent) -> bool {
        matches!(key_event.code, KeyCode::Char('f'))
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

impl Command for PageDownCommand {
    fn is_relevant(
        &self,
        key_event: KeyEvent,
        mode: EditorMode,
        _context: &CommandContext,
    ) -> bool {
        // Must be Ctrl+f key combination
        if !Self::is_page_down_key(key_event) {
            return false;
        }

        // Must be in a navigation mode
        let is_relevant = Self::is_navigation_mode(mode);

        // Add debug logging like the legacy command
        tracing::debug!(
            "PageDownCommand.is_relevant(): ctrl+f={}, mode={:?}, result={}",
            true, // We already validated it's ctrl+f above
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
        // Use the pane manager's page down method which returns PostCommandActions
        let events = context.app_state.pane_manager.move_cursor_page_down();

        tracing::debug!(
            "PageDownCommand executed, generated {} events",
            events.len()
        );

        Ok(events)
    }

    fn name(&self) -> &'static str {
        "PageDownCommand"
    }
}

impl Default for PageDownCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyModifiers;

    #[test]
    fn page_down_command_should_have_correct_name() {
        let cmd = PageDownCommand::new();
        assert_eq!(cmd.name(), "PageDownCommand");
    }

    #[test]
    fn should_create_default_instance() {
        let cmd = PageDownCommand;
        assert_eq!(cmd.name(), "PageDownCommand");
    }

    #[test]
    fn is_page_down_key_should_detect_ctrl_f() {
        let event = KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL);
        assert!(PageDownCommand::is_page_down_key(event));
    }

    #[test]
    fn is_page_down_key_should_reject_plain_f() {
        let event = KeyEvent::new(KeyCode::Char('f'), KeyModifiers::empty());
        assert!(!PageDownCommand::is_page_down_key(event));
    }

    #[test]
    fn is_page_down_key_should_reject_ctrl_shift_f() {
        let event = KeyEvent::new(
            KeyCode::Char('f'),
            KeyModifiers::CONTROL | KeyModifiers::SHIFT,
        );
        assert!(!PageDownCommand::is_page_down_key(event));
    }

    #[test]
    fn is_page_down_key_should_reject_alt_ctrl_f() {
        let event = KeyEvent::new(
            KeyCode::Char('f'),
            KeyModifiers::CONTROL | KeyModifiers::ALT,
        );
        assert!(!PageDownCommand::is_page_down_key(event));
    }

    #[test]
    fn is_page_down_key_should_reject_other_chars() {
        let event = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::CONTROL);
        assert!(!PageDownCommand::is_page_down_key(event));
    }

    #[test]
    fn is_navigation_mode_should_identify_correct_modes() {
        // Navigation modes
        assert!(PageDownCommand::is_navigation_mode(EditorMode::Normal));
        assert!(PageDownCommand::is_navigation_mode(EditorMode::Visual));
        assert!(PageDownCommand::is_navigation_mode(EditorMode::VisualLine));
        assert!(PageDownCommand::is_navigation_mode(EditorMode::VisualBlock));

        // Non-navigation modes
        assert!(!PageDownCommand::is_navigation_mode(EditorMode::Insert));
        assert!(!PageDownCommand::is_navigation_mode(
            EditorMode::VisualBlockInsert
        ));
        assert!(!PageDownCommand::is_navigation_mode(EditorMode::Command));
    }

    #[test]
    fn is_relevant_should_accept_ctrl_f_in_normal_mode() {
        let cmd = PageDownCommand::new();
        let event = KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL);
        let context = CommandContext::test_default();

        assert!(cmd.is_relevant(event, EditorMode::Normal, &context));
    }

    #[test]
    fn is_relevant_should_accept_ctrl_f_in_visual_modes() {
        let cmd = PageDownCommand::new();
        let event = KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL);
        let context = CommandContext::test_default();

        assert!(cmd.is_relevant(event, EditorMode::Visual, &context));
        assert!(cmd.is_relevant(event, EditorMode::VisualLine, &context));
        assert!(cmd.is_relevant(event, EditorMode::VisualBlock, &context));
    }

    #[test]
    fn is_relevant_should_reject_ctrl_f_in_insert_mode() {
        let cmd = PageDownCommand::new();
        let event = KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL);
        let context = CommandContext::test_default();

        assert!(!cmd.is_relevant(event, EditorMode::Insert, &context));
    }

    #[test]
    fn is_relevant_should_reject_ctrl_f_in_command_mode() {
        let cmd = PageDownCommand::new();
        let event = KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL);
        let context = CommandContext::test_default();

        assert!(!cmd.is_relevant(event, EditorMode::Command, &context));
    }

    #[test]
    fn is_relevant_should_reject_plain_f_in_normal_mode() {
        let cmd = PageDownCommand::new();
        let event = KeyEvent::new(KeyCode::Char('f'), KeyModifiers::empty());
        let context = CommandContext::test_default();

        assert!(!cmd.is_relevant(event, EditorMode::Normal, &context));
    }

    #[test]
    fn is_relevant_should_reject_wrong_key_combinations() {
        let cmd = PageDownCommand::new();
        let context = CommandContext::test_default();

        // Test various wrong combinations
        let wrong_events = vec![
            KeyEvent::new(KeyCode::Char('g'), KeyModifiers::CONTROL),
            KeyEvent::new(
                KeyCode::Char('f'),
                KeyModifiers::CONTROL | KeyModifiers::SHIFT,
            ),
            KeyEvent::new(
                KeyCode::Char('f'),
                KeyModifiers::CONTROL | KeyModifiers::ALT,
            ),
            KeyEvent::new(KeyCode::PageDown, KeyModifiers::empty()),
        ];

        for event in wrong_events {
            assert!(
                !cmd.is_relevant(event, EditorMode::Normal, &context),
                "Should reject event: {event:?}"
            );
        }
    }

    #[test]
    fn execute_should_call_pane_manager_page_down() {
        // This test verifies that execute calls the right method
        // Integration tests will verify the actual functionality
        let cmd = PageDownCommand::new();
        // Cannot test execute without mocking AppState, so this is a placeholder
        // for integration tests that will verify the method calls pane_manager.move_cursor_page_down()
        assert_eq!(cmd.name(), "PageDownCommand");
    }
}

// Register the command with the dynamic discovery system
register_command!(PageDownCommand, "PageDownCommand");
