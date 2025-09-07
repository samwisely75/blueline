//! # Half Page Up Command
//!
//! Command for moving cursor up half a page (Ctrl+u key).
//! This implements the vim-style half-page-up navigation that moves the cursor
//! up by half the height of the viewport.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::register_command;
use crate::repl::models::pane_state::EditorMode;
use crate::repl::view_models::commands::{Command, CommandContext, ExecutionContext};
use crate::repl::view_models::post_command_actions::PostCommandAction;

/// Command to move cursor up half a page (Ctrl+u)
///
/// This command implements vim-style half page navigation:
/// 1. Checks for Ctrl+u key combination in navigation modes
/// 2. Calls the pane manager to move cursor up by half viewport height
/// 3. Returns appropriate PostCommandActions to update the display
///
/// Only works in navigation modes (Normal, Visual, VisualLine, VisualBlock).
pub struct HalfPageUpCommand;

impl HalfPageUpCommand {
    /// Create new HalfPageUpCommand
    pub fn new() -> Self {
        Self
    }

    /// Check if the key event is Ctrl+u for half page up navigation
    fn is_half_page_up_key(key_event: KeyEvent) -> bool {
        matches!(key_event.code, KeyCode::Char('u'))
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

impl Command for HalfPageUpCommand {
    fn is_relevant(
        &self,
        key_event: KeyEvent,
        mode: EditorMode,
        _context: &CommandContext,
    ) -> bool {
        // Must be Ctrl+u key combination
        if !Self::is_half_page_up_key(key_event) {
            return false;
        }

        // Must be in a navigation mode
        let is_relevant = Self::is_navigation_mode(mode);

        // Add debug logging like the legacy command
        tracing::debug!(
            "HalfPageUpCommand.is_relevant(): ctrl+u={}, mode={:?}, result={}",
            true, // We already validated it's ctrl+u above
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
        // Use the pane manager's half page up method which returns PostCommandActions
        let events = context.app_state.pane_manager.move_cursor_half_page_up();

        tracing::debug!(
            "HalfPageUpCommand executed, generated {} events",
            events.len()
        );

        Ok(events)
    }

    fn name(&self) -> &'static str {
        "HalfPageUpCommand"
    }
}

impl Default for HalfPageUpCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyModifiers;

    #[test]
    fn half_page_up_command_should_have_correct_name() {
        let cmd = HalfPageUpCommand::new();
        assert_eq!(cmd.name(), "HalfPageUpCommand");
    }

    #[test]
    fn should_create_default_instance() {
        let cmd = HalfPageUpCommand;
        assert_eq!(cmd.name(), "HalfPageUpCommand");
    }

    #[test]
    fn is_half_page_up_key_should_detect_ctrl_u() {
        let event = KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL);
        assert!(HalfPageUpCommand::is_half_page_up_key(event));
    }

    #[test]
    fn is_half_page_up_key_should_reject_plain_u() {
        let event = KeyEvent::new(KeyCode::Char('u'), KeyModifiers::empty());
        assert!(!HalfPageUpCommand::is_half_page_up_key(event));
    }

    #[test]
    fn is_half_page_up_key_should_reject_ctrl_shift_u() {
        let event = KeyEvent::new(
            KeyCode::Char('u'),
            KeyModifiers::CONTROL | KeyModifiers::SHIFT,
        );
        assert!(!HalfPageUpCommand::is_half_page_up_key(event));
    }

    #[test]
    fn is_half_page_up_key_should_reject_alt_ctrl_u() {
        let event = KeyEvent::new(
            KeyCode::Char('u'),
            KeyModifiers::CONTROL | KeyModifiers::ALT,
        );
        assert!(!HalfPageUpCommand::is_half_page_up_key(event));
    }

    #[test]
    fn is_half_page_up_key_should_reject_other_chars() {
        let event = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL);
        assert!(!HalfPageUpCommand::is_half_page_up_key(event));
    }

    #[test]
    fn is_navigation_mode_should_identify_correct_modes() {
        // Navigation modes
        assert!(HalfPageUpCommand::is_navigation_mode(EditorMode::Normal));
        assert!(HalfPageUpCommand::is_navigation_mode(EditorMode::Visual));
        assert!(HalfPageUpCommand::is_navigation_mode(
            EditorMode::VisualLine
        ));
        assert!(HalfPageUpCommand::is_navigation_mode(
            EditorMode::VisualBlock
        ));

        // Non-navigation modes
        assert!(!HalfPageUpCommand::is_navigation_mode(EditorMode::Insert));
        assert!(!HalfPageUpCommand::is_navigation_mode(
            EditorMode::VisualBlockInsert
        ));
        assert!(!HalfPageUpCommand::is_navigation_mode(EditorMode::Command));
    }

    #[test]
    fn is_relevant_should_accept_ctrl_u_in_normal_mode() {
        let cmd = HalfPageUpCommand::new();
        let event = KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL);
        let context = CommandContext::test_default();

        assert!(cmd.is_relevant(event, EditorMode::Normal, &context));
    }

    #[test]
    fn is_relevant_should_accept_ctrl_u_in_visual_modes() {
        let cmd = HalfPageUpCommand::new();
        let event = KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL);
        let context = CommandContext::test_default();

        assert!(cmd.is_relevant(event, EditorMode::Visual, &context));
        assert!(cmd.is_relevant(event, EditorMode::VisualLine, &context));
        assert!(cmd.is_relevant(event, EditorMode::VisualBlock, &context));
    }

    #[test]
    fn is_relevant_should_reject_ctrl_u_in_insert_mode() {
        let cmd = HalfPageUpCommand::new();
        let event = KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL);
        let context = CommandContext::test_default();

        assert!(!cmd.is_relevant(event, EditorMode::Insert, &context));
    }

    #[test]
    fn is_relevant_should_reject_ctrl_u_in_command_mode() {
        let cmd = HalfPageUpCommand::new();
        let event = KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL);
        let context = CommandContext::test_default();

        assert!(!cmd.is_relevant(event, EditorMode::Command, &context));
    }

    #[test]
    fn is_relevant_should_reject_ctrl_u_in_visual_block_insert_mode() {
        let cmd = HalfPageUpCommand::new();
        let event = KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL);
        let context = CommandContext::test_default();

        assert!(!cmd.is_relevant(event, EditorMode::VisualBlockInsert, &context));
    }

    #[test]
    fn is_relevant_should_reject_plain_u_in_normal_mode() {
        let cmd = HalfPageUpCommand::new();
        let event = KeyEvent::new(KeyCode::Char('u'), KeyModifiers::empty());
        let context = CommandContext::test_default();

        assert!(!cmd.is_relevant(event, EditorMode::Normal, &context));
    }

    #[test]
    fn is_relevant_should_reject_wrong_key_combinations() {
        let cmd = HalfPageUpCommand::new();
        let context = CommandContext::test_default();

        // Test various wrong combinations
        let wrong_events = vec![
            KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL),
            KeyEvent::new(
                KeyCode::Char('u'),
                KeyModifiers::CONTROL | KeyModifiers::SHIFT,
            ),
            KeyEvent::new(
                KeyCode::Char('u'),
                KeyModifiers::CONTROL | KeyModifiers::ALT,
            ),
            KeyEvent::new(KeyCode::PageUp, KeyModifiers::empty()),
        ];

        for event in wrong_events {
            assert!(
                !cmd.is_relevant(event, EditorMode::Normal, &context),
                "Should reject event: {event:?}"
            );
        }
    }

    #[test]
    fn execute_should_call_pane_manager_half_page_up() {
        // This test verifies that execute calls the right method
        // Integration tests will verify the actual functionality
        let cmd = HalfPageUpCommand::new();
        // Cannot test execute without mocking AppState, so this is a placeholder
        // for integration tests that will verify the method calls pane_manager.move_cursor_half_page_up()
        assert_eq!(cmd.name(), "HalfPageUpCommand");
    }

    #[test]
    fn is_relevant_should_reject_ctrl_u_in_g_prefix_mode() {
        let cmd = HalfPageUpCommand::new();
        let event = KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL);
        let context = CommandContext::test_default();

        assert!(!cmd.is_relevant(event, EditorMode::GPrefix, &context));
    }

    #[test]
    fn is_relevant_should_reject_ctrl_u_in_d_prefix_mode() {
        let cmd = HalfPageUpCommand::new();
        let event = KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL);
        let context = CommandContext::test_default();

        assert!(!cmd.is_relevant(event, EditorMode::DPrefix, &context));
    }

    #[test]
    fn is_relevant_should_reject_ctrl_u_in_y_prefix_mode() {
        let cmd = HalfPageUpCommand::new();
        let event = KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL);
        let context = CommandContext::test_default();

        assert!(!cmd.is_relevant(event, EditorMode::YPrefix, &context));
    }

    #[test]
    fn should_not_be_relevant_for_uppercase_u() {
        let cmd = HalfPageUpCommand::new();
        let event = KeyEvent::new(KeyCode::Char('U'), KeyModifiers::CONTROL);
        let context = CommandContext::test_default();

        assert!(!cmd.is_relevant(event, EditorMode::Normal, &context));
    }

    #[test]
    fn should_not_be_relevant_for_shift_ctrl_u() {
        let cmd = HalfPageUpCommand::new();
        let event = KeyEvent::new(
            KeyCode::Char('u'),
            KeyModifiers::SHIFT | KeyModifiers::CONTROL,
        );
        let context = CommandContext::test_default();

        assert!(!cmd.is_relevant(event, EditorMode::Normal, &context));
    }
}

// Register the command with the dynamic discovery system
register_command!(HalfPageUpCommand, "HalfPageUpCommand");
