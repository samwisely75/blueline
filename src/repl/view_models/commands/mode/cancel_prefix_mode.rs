//! # Cancel Prefix Mode Command
//!
//! Command to cancel any prefix mode (DPrefix, YPrefix, GPrefix) when Escape key is pressed.
//! This provides a consistent way to exit prefix modes and return to Normal mode.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use crate::register_command;
use crate::repl::models::pane_state::EditorMode;
use crate::repl::view_models::commands::{Command, CommandContext, ExecutionContext};
use crate::repl::view_models::post_command_actions::PostCommandAction;

/// Command to cancel any prefix mode and return to Normal mode
///
/// This command handles the Escape key in any prefix mode (DPrefix, YPrefix, GPrefix)
/// and returns the editor to Normal mode. It provides a consistent escape mechanism
/// for all prefix modes.
///
/// The command is relevant when:
/// - Key is Escape without modifiers
/// - Current mode is any prefix mode (DPrefix, YPrefix, GPrefix)
///
/// After execution, the editor returns to Normal mode with cursor position preserved.
#[derive(Default)]
pub struct CancelPrefixModeCommand;

impl CancelPrefixModeCommand {
    /// Create new CancelPrefixModeCommand
    pub fn new() -> Self {
        Self
    }
}

/// Check if current mode is a prefix mode
fn is_prefix_mode(mode: EditorMode) -> bool {
    matches!(
        mode,
        EditorMode::DPrefix | EditorMode::YPrefix | EditorMode::GPrefix
    )
}

impl Command for CancelPrefixModeCommand {
    fn is_relevant(
        &self,
        key_event: KeyEvent,
        mode: EditorMode,
        _context: &CommandContext,
    ) -> bool {
        // Must be Escape key without modifiers in any prefix mode
        matches!(key_event.code, KeyCode::Esc)
            && key_event.modifiers.is_empty()
            && is_prefix_mode(mode)
    }

    fn execute(
        &self,
        _key_event: KeyEvent,
        context: &mut ExecutionContext,
    ) -> Result<Vec<PostCommandAction>> {
        // Save current cursor position before mode change
        let current_cursor = context.app_state.pane_manager.get_current_display_cursor();

        tracing::debug!(
            "CancelPrefixModeCommand: Saving cursor position {:?} before mode change",
            current_cursor
        );

        // Return to Normal mode from any prefix mode
        context.app_state.change_mode(EditorMode::Normal)?;

        // Restore cursor position if it changed during mode transition
        let new_cursor = context.app_state.pane_manager.get_current_display_cursor();
        if new_cursor != current_cursor {
            tracing::debug!(
                "CancelPrefixModeCommand: Cursor moved from {:?} to {:?}, restoring original position",
                current_cursor,
                new_cursor
            );

            // Use set_current_display_cursor to restore the exact position
            context
                .app_state
                .pane_manager
                .set_current_display_cursor(current_cursor);
        }

        tracing::debug!("CancelPrefixModeCommand: Cancelled prefix mode, returned to Normal mode with cursor preserved");

        // Return appropriate PostCommandActions
        Ok(vec![
            PostCommandAction::StatusBarUpdateRequired,
            PostCommandAction::ActiveCursorUpdateRequired, // Ensure cursor position is updated
        ])
    }

    fn name(&self) -> &'static str {
        "CancelPrefixModeCommand"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::models::pane_state::Pane;
    use crate::repl::models::AppState;
    use crate::repl::services::Services;
    use crossterm::event::KeyModifiers;

    #[test]
    fn cancel_prefix_mode_command_should_return_correct_name() {
        let command = CancelPrefixModeCommand::new();
        assert_eq!(command.name(), "CancelPrefixModeCommand");
    }

    #[test]
    fn cancel_prefix_mode_command_should_be_relevant_for_escape_in_prefix_modes() {
        let command = CancelPrefixModeCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::Normal, // This will be overridden by the mode parameter
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let esc_key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);

        // Should be relevant in all prefix modes
        assert!(command.is_relevant(esc_key, EditorMode::DPrefix, &context));
        assert!(command.is_relevant(esc_key, EditorMode::YPrefix, &context));
        assert!(command.is_relevant(esc_key, EditorMode::GPrefix, &context));
    }

    #[test]
    fn cancel_prefix_mode_command_should_not_be_relevant_in_non_prefix_modes() {
        let command = CancelPrefixModeCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let esc_key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);

        // Should not be relevant in non-prefix modes
        assert!(!command.is_relevant(esc_key, EditorMode::Normal, &context));
        assert!(!command.is_relevant(esc_key, EditorMode::Insert, &context));
        assert!(!command.is_relevant(esc_key, EditorMode::Visual, &context));
        assert!(!command.is_relevant(esc_key, EditorMode::VisualLine, &context));
        assert!(!command.is_relevant(esc_key, EditorMode::VisualBlock, &context));
        assert!(!command.is_relevant(esc_key, EditorMode::Command, &context));
    }

    #[test]
    fn cancel_prefix_mode_command_should_not_be_relevant_for_non_escape_keys() {
        let command = CancelPrefixModeCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::DPrefix,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        // Test other keys - should not be relevant
        let other_keys = vec![
            KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE),
        ];

        for key_event in other_keys {
            assert!(
                !command.is_relevant(key_event, EditorMode::DPrefix, &context),
                "Key {key_event:?} should not be relevant"
            );
        }
    }

    #[test]
    fn cancel_prefix_mode_command_should_not_be_relevant_with_modifiers() {
        let command = CancelPrefixModeCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::DPrefix,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        // Test with modifiers - should not be relevant
        let modified_keys = vec![
            KeyEvent::new(KeyCode::Esc, KeyModifiers::CONTROL),
            KeyEvent::new(KeyCode::Esc, KeyModifiers::SHIFT),
            KeyEvent::new(KeyCode::Esc, KeyModifiers::ALT),
        ];

        for key_event in modified_keys {
            assert!(
                !command.is_relevant(key_event, EditorMode::DPrefix, &context),
                "Modified key {key_event:?} should not be relevant"
            );
        }
    }

    #[test]
    fn cancel_prefix_mode_command_should_work_in_read_only_panes() {
        let command = CancelPrefixModeCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::GPrefix,
            current_pane: Pane::Response,
            is_read_only: true,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let esc_key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);

        // Should be relevant even in read-only panes for cancellation
        assert!(command.is_relevant(esc_key, EditorMode::GPrefix, &context));
    }

    #[test]
    fn cancel_prefix_mode_execute_should_change_mode_to_normal_from_d_prefix() {
        let command = CancelPrefixModeCommand::new();
        let mut app_state = AppState::new();
        let mut services = Services::new();

        // Set to DPrefix mode first
        app_state.change_mode(EditorMode::DPrefix).unwrap();
        assert_eq!(app_state.get_mode(), EditorMode::DPrefix);

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
        let events = result.unwrap();

        // Should return status bar update event
        assert!(!events.is_empty());
        assert!(events
            .iter()
            .any(|e| matches!(e, PostCommandAction::StatusBarUpdateRequired)));

        // Should have changed mode to Normal
        assert_eq!(context.app_state.get_mode(), EditorMode::Normal);
    }

    #[test]
    fn cancel_prefix_mode_execute_should_change_mode_to_normal_from_y_prefix() {
        let command = CancelPrefixModeCommand::new();
        let mut app_state = AppState::new();
        let mut services = Services::new();

        // Set to YPrefix mode first
        app_state.change_mode(EditorMode::YPrefix).unwrap();
        assert_eq!(app_state.get_mode(), EditorMode::YPrefix);

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
        let events = result.unwrap();

        // Should return appropriate events
        assert!(!events.is_empty());
        assert!(events
            .iter()
            .any(|e| matches!(e, PostCommandAction::StatusBarUpdateRequired)));

        // Should have changed mode to Normal
        assert_eq!(context.app_state.get_mode(), EditorMode::Normal);
    }

    #[test]
    fn cancel_prefix_mode_execute_should_change_mode_to_normal_from_g_prefix() {
        let command = CancelPrefixModeCommand::new();
        let mut app_state = AppState::new();
        let mut services = Services::new();

        // Set to GPrefix mode first
        app_state.change_mode(EditorMode::GPrefix).unwrap();
        assert_eq!(app_state.get_mode(), EditorMode::GPrefix);

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
        let events = result.unwrap();

        // Should return appropriate events
        assert!(!events.is_empty());
        assert!(events
            .iter()
            .any(|e| matches!(e, PostCommandAction::StatusBarUpdateRequired)));

        // Should have changed mode to Normal
        assert_eq!(context.app_state.get_mode(), EditorMode::Normal);
    }

    #[test]
    fn default_should_create_new_instance() {
        let command = CancelPrefixModeCommand;
        assert_eq!(command.name(), "CancelPrefixModeCommand");
    }

    #[test]
    fn is_prefix_mode_should_identify_correct_modes() {
        // Prefix modes should return true
        assert!(is_prefix_mode(EditorMode::DPrefix));
        assert!(is_prefix_mode(EditorMode::YPrefix));
        assert!(is_prefix_mode(EditorMode::GPrefix));

        // Non-prefix modes should return false
        assert!(!is_prefix_mode(EditorMode::Normal));
        assert!(!is_prefix_mode(EditorMode::Insert));
        assert!(!is_prefix_mode(EditorMode::Visual));
        assert!(!is_prefix_mode(EditorMode::VisualLine));
        assert!(!is_prefix_mode(EditorMode::VisualBlock));
        assert!(!is_prefix_mode(EditorMode::Command));
    }
}

// Auto-register this command using the inventory system
register_command!(CancelPrefixModeCommand, "CancelPrefixModeCommand");
