//! # Cancel G Prefix Command
//!
//! Command to cancel GPrefix mode when an unknown key is pressed.
//! This prevents the editor from getting stuck in GPrefix mode when
//! the user presses 'g' followed by an unrecognized key.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use crate::register_command;
use crate::repl::models::pane_state::EditorMode;
use crate::repl::view_models::commands::{Command, CommandContext, ExecutionContext};
use crate::repl::view_models::post_command_actions::PostCommandAction;

/// Command to cancel GPrefix mode on any non-matching key
///
/// This command serves as a fallback when in GPrefix mode and an unknown
/// key is pressed. It ensures the editor doesn't get stuck in GPrefix mode
/// by returning to Normal mode.
///
/// The command is relevant when:
/// - Current mode is GPrefix
/// - Key is NOT one of the known GPrefix commands ('g', 'v')
/// - Not in a read-only pane (though cancellation is allowed everywhere)
///
/// Known GPrefix commands that this should NOT cancel:
/// - 'g' (handled by GoToTopCommand for gg)
/// - 'v' (handled by RepeatVisualSelectionCommand for gv)
#[derive(Default)]
pub struct CancelGPrefixCommand;

impl CancelGPrefixCommand {
    /// Create new CancelGPrefixCommand
    pub fn new() -> Self {
        Self
    }
}

impl Command for CancelGPrefixCommand {
    fn is_relevant(
        &self,
        key_event: KeyEvent,
        mode: EditorMode,
        _context: &CommandContext,
    ) -> bool {
        // Only relevant in GPrefix mode
        if mode != EditorMode::GPrefix {
            return false;
        }

        // Cancel on any key except the known GPrefix commands
        match key_event.code {
            KeyCode::Char('g') if key_event.modifiers.is_empty() => false, // GoToTopCommand handles this
            KeyCode::Char('v') if key_event.modifiers.is_empty() => false, // RepeatVisualSelectionCommand handles this
            _ => true, // All other keys should cancel GPrefix mode
        }
    }

    fn execute(
        &self,
        _key_event: KeyEvent,
        context: &mut ExecutionContext,
    ) -> Result<Vec<PostCommandAction>> {
        // Save current cursor position before mode change
        let current_cursor = context.app_state.pane_manager.get_current_display_cursor();

        tracing::debug!(
            "CancelGPrefixCommand: Saving cursor position {:?} before mode change",
            current_cursor
        );

        // Return to Normal mode from GPrefix mode
        context.app_state.change_mode(EditorMode::Normal)?;

        // Restore cursor position if it changed during mode transition
        let new_cursor = context.app_state.pane_manager.get_current_display_cursor();
        if new_cursor != current_cursor {
            tracing::debug!(
                "CancelGPrefixCommand: Cursor moved from {:?} to {:?}, restoring original position",
                current_cursor,
                new_cursor
            );

            // Use set_current_display_cursor to restore the exact position
            context
                .app_state
                .pane_manager
                .set_current_display_cursor(current_cursor);
        }

        tracing::debug!("CancelGPrefixCommand: Cancelled GPrefix mode, returned to Normal mode with cursor preserved");

        // Return appropriate PostCommandActions
        Ok(vec![
            PostCommandAction::StatusBarUpdateRequired,
            PostCommandAction::ActiveCursorUpdateRequired, // Ensure cursor position is updated
        ])
    }

    fn name(&self) -> &'static str {
        "CancelGPrefixCommand"
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
    fn cancel_g_prefix_command_should_return_correct_name() {
        let command = CancelGPrefixCommand::new();
        assert_eq!(command.name(), "CancelGPrefixCommand");
    }

    #[test]
    fn cancel_g_prefix_command_should_be_relevant_for_unknown_keys_in_g_prefix_mode() {
        let command = CancelGPrefixCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::GPrefix,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        // Unknown keys should be relevant
        let unknown_keys = vec![
            KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('w'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE),
        ];

        for key_event in unknown_keys {
            assert!(
                command.is_relevant(key_event, EditorMode::GPrefix, &context),
                "Key {key_event:?} should be relevant for canceling GPrefix mode"
            );
        }
    }

    #[test]
    fn cancel_g_prefix_command_should_not_be_relevant_for_known_g_prefix_keys() {
        let command = CancelGPrefixCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::GPrefix,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        // Known GPrefix keys should NOT be relevant (handled by other commands)
        let known_keys = vec![
            KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE), // GoToTopCommand
            KeyEvent::new(KeyCode::Char('v'), KeyModifiers::NONE), // RepeatVisualSelectionCommand
        ];

        for key_event in known_keys {
            assert!(
                !command.is_relevant(key_event, EditorMode::GPrefix, &context),
                "Key {key_event:?} should NOT be relevant (handled by specific command)"
            );
        }
    }

    #[test]
    fn cancel_g_prefix_command_should_not_be_relevant_in_non_g_prefix_modes() {
        let command = CancelGPrefixCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let key_event = KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE);

        // Should not be relevant in other modes
        assert!(!command.is_relevant(key_event, EditorMode::Normal, &context));
        assert!(!command.is_relevant(key_event, EditorMode::Insert, &context));
        assert!(!command.is_relevant(key_event, EditorMode::Visual, &context));
        assert!(!command.is_relevant(key_event, EditorMode::VisualLine, &context));
        assert!(!command.is_relevant(key_event, EditorMode::VisualBlock, &context));
        assert!(!command.is_relevant(key_event, EditorMode::Command, &context));
        assert!(!command.is_relevant(key_event, EditorMode::DPrefix, &context));
        assert!(!command.is_relevant(key_event, EditorMode::YPrefix, &context));
    }

    #[test]
    fn cancel_g_prefix_command_should_work_in_read_only_panes() {
        let command = CancelGPrefixCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::GPrefix,
            current_pane: Pane::Response,
            is_read_only: true,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let key_event = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);

        // Should be relevant even in read-only panes for cancellation
        assert!(command.is_relevant(key_event, EditorMode::GPrefix, &context));
    }

    #[test]
    fn cancel_g_prefix_execute_should_change_mode_to_normal() {
        let command = CancelGPrefixCommand::new();
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

        // Should return status bar update event
        assert!(!events.is_empty());
        assert!(events
            .iter()
            .any(|e| matches!(e, PostCommandAction::StatusBarUpdateRequired)));

        // Should have changed mode to Normal
        assert_eq!(context.app_state.get_mode(), EditorMode::Normal);
    }

    #[test]
    fn cancel_g_prefix_command_should_handle_modified_keys() {
        let command = CancelGPrefixCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::GPrefix,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        // Modified versions of known keys should cancel GPrefix mode
        let modified_keys = vec![
            KeyEvent::new(KeyCode::Char('g'), KeyModifiers::CONTROL),
            KeyEvent::new(KeyCode::Char('g'), KeyModifiers::SHIFT),
            KeyEvent::new(KeyCode::Char('v'), KeyModifiers::CONTROL),
            KeyEvent::new(KeyCode::Char('v'), KeyModifiers::SHIFT),
        ];

        for key_event in modified_keys {
            assert!(
                command.is_relevant(key_event, EditorMode::GPrefix, &context),
                "Modified key {key_event:?} should be relevant for canceling GPrefix mode"
            );
        }
    }

    #[test]
    fn default_should_create_new_instance() {
        let command = CancelGPrefixCommand;
        assert_eq!(command.name(), "CancelGPrefixCommand");
    }
}

// Auto-register this command using the inventory system
register_command!(CancelGPrefixCommand, "CancelGPrefixCommand");
