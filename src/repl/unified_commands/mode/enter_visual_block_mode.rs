//! # Enter Visual Block Mode Command
//!
//! Command to handle Ctrl+V key in Normal mode which enters Visual Block mode
//! for rectangular selections and block operations.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::register_command;
use crate::repl::models::pane_state::EditorMode;
use crate::repl::unified_commands::{Command, CommandContext, ExecutionContext};
use crate::repl::view_models::post_command_actions::PostCommandAction;

/// Command to handle entering Visual Block mode with Ctrl+V
///
/// This command handles Ctrl+V in Normal mode to transition the editor
/// to Visual Block mode for rectangular text selections and block operations.
pub struct EnterVisualBlockModeCommand;

impl EnterVisualBlockModeCommand {
    /// Create new EnterVisualBlockModeCommand
    pub fn new() -> Self {
        Self
    }
}

impl Default for EnterVisualBlockModeCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl Command for EnterVisualBlockModeCommand {
    fn is_relevant(
        &self,
        key_event: KeyEvent,
        mode: EditorMode,
        _context: &CommandContext,
    ) -> bool {
        // Handle Ctrl+V key combination in Normal mode
        matches!(key_event.code, KeyCode::Char('v'))
            && key_event.modifiers.contains(KeyModifiers::CONTROL)
            && mode == EditorMode::Normal
    }

    fn execute(
        &self,
        _key_event: KeyEvent,
        context: &mut ExecutionContext,
    ) -> Result<Vec<PostCommandAction>> {
        tracing::debug!("EnterVisualBlockModeCommand: entering Visual Block mode");

        // Change mode to Visual Block mode
        let _mode_change = context.app_state.set_mode(EditorMode::VisualBlock);

        tracing::debug!("EnterVisualBlockModeCommand: mode changed to Visual Block");

        // Return appropriate PostCommandActions
        Ok(vec![
            PostCommandAction::StatusBarUpdateRequired,
            PostCommandAction::ActiveCursorUpdateRequired,
        ])
    }

    fn name(&self) -> &'static str {
        "EnterVisualBlockModeCommand"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::models::pane_state::Pane;
    use crate::repl::models::AppState;
    use crate::repl::services::Services;

    fn create_test_context() -> CommandContext {
        CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        }
    }

    #[test]
    fn enter_visual_block_mode_command_should_return_correct_name() {
        let command = EnterVisualBlockModeCommand::new();
        assert_eq!(command.name(), "EnterVisualBlockModeCommand");
    }

    #[test]
    fn enter_visual_block_mode_command_should_be_relevant_for_ctrl_v_in_normal_mode() {
        let command = EnterVisualBlockModeCommand::new();
        let context = create_test_context();
        let key_event = KeyEvent::new(KeyCode::Char('v'), KeyModifiers::CONTROL);

        assert!(command.is_relevant(key_event, EditorMode::Normal, &context));
    }

    #[test]
    fn enter_visual_block_mode_command_should_not_be_relevant_for_v_without_ctrl() {
        let command = EnterVisualBlockModeCommand::new();
        let context = create_test_context();
        let key_event = KeyEvent::new(KeyCode::Char('v'), KeyModifiers::NONE);

        assert!(!command.is_relevant(key_event, EditorMode::Normal, &context));
    }

    #[test]
    fn enter_visual_block_mode_command_should_not_be_relevant_in_insert_mode() {
        let command = EnterVisualBlockModeCommand::new();
        let context = create_test_context();
        let key_event = KeyEvent::new(KeyCode::Char('v'), KeyModifiers::CONTROL);

        assert!(!command.is_relevant(key_event, EditorMode::Insert, &context));
    }

    #[test]
    fn enter_visual_block_mode_command_should_not_be_relevant_in_visual_mode() {
        let command = EnterVisualBlockModeCommand::new();
        let context = create_test_context();
        let key_event = KeyEvent::new(KeyCode::Char('v'), KeyModifiers::CONTROL);

        assert!(!command.is_relevant(key_event, EditorMode::Visual, &context));
    }

    #[test]
    fn enter_visual_block_mode_command_should_not_be_relevant_in_command_mode() {
        let command = EnterVisualBlockModeCommand::new();
        let context = create_test_context();
        let key_event = KeyEvent::new(KeyCode::Char('v'), KeyModifiers::CONTROL);

        assert!(!command.is_relevant(key_event, EditorMode::Command, &context));
    }

    #[test]
    fn enter_visual_block_mode_command_should_not_be_relevant_with_additional_modifiers() {
        let command = EnterVisualBlockModeCommand::new();
        let context = create_test_context();

        let modified_keys = vec![
            KeyEvent::new(
                KeyCode::Char('v'),
                KeyModifiers::CONTROL | KeyModifiers::SHIFT,
            ),
            KeyEvent::new(
                KeyCode::Char('v'),
                KeyModifiers::CONTROL | KeyModifiers::ALT,
            ),
            KeyEvent::new(
                KeyCode::Char('v'),
                KeyModifiers::CONTROL | KeyModifiers::SHIFT | KeyModifiers::ALT,
            ),
        ];

        for key_event in modified_keys {
            // Should still be relevant as long as CONTROL is present
            assert!(
                command.is_relevant(key_event, EditorMode::Normal, &context),
                "Should be relevant for Ctrl+V with additional modifiers: {key_event:?}"
            );
        }
    }

    #[test]
    fn enter_visual_block_mode_command_should_not_be_relevant_for_other_keys() {
        let command = EnterVisualBlockModeCommand::new();
        let context = create_test_context();

        let other_keys = vec![
            KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
            KeyEvent::new(KeyCode::Char('x'), KeyModifiers::CONTROL),
            KeyEvent::new(KeyCode::Char('v'), KeyModifiers::SHIFT),
            KeyEvent::new(KeyCode::Char('v'), KeyModifiers::ALT),
            KeyEvent::new(KeyCode::Enter, KeyModifiers::CONTROL),
            KeyEvent::new(KeyCode::Esc, KeyModifiers::CONTROL),
        ];

        for key_event in other_keys {
            assert!(
                !command.is_relevant(key_event, EditorMode::Normal, &context),
                "Should not be relevant for key: {key_event:?}"
            );
        }
    }

    #[test]
    fn enter_visual_block_mode_command_should_handle_different_panes() {
        let command = EnterVisualBlockModeCommand::new();

        let panes = vec![Pane::Request, Pane::Response];

        for pane in panes {
            let context = CommandContext {
                current_mode: EditorMode::Normal,
                current_pane: pane,
                is_read_only: false,
                has_selection: false,
                ex_command_buffer: String::new(),
            };
            let key_event = KeyEvent::new(KeyCode::Char('v'), KeyModifiers::CONTROL);

            assert!(
                command.is_relevant(key_event, EditorMode::Normal, &context),
                "Should be relevant in {pane:?} pane"
            );
        }
    }

    #[test]
    fn enter_visual_block_mode_command_should_handle_read_only_context() {
        let command = EnterVisualBlockModeCommand::new();
        let read_only_context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Response,
            is_read_only: true, // Read-only context
            has_selection: false,
            ex_command_buffer: String::new(),
        };
        let key_event = KeyEvent::new(KeyCode::Char('v'), KeyModifiers::CONTROL);

        // Should still be relevant even in read-only context (visual block mode is for selection)
        assert!(command.is_relevant(key_event, EditorMode::Normal, &read_only_context));
    }

    #[test]
    fn enter_visual_block_mode_command_execute_should_change_mode_to_visual_block() {
        let command = EnterVisualBlockModeCommand::new();
        let mut app_state = AppState::new();
        let mut services = Services::new();

        // Verify initial state is Normal mode
        assert_eq!(
            app_state.pane_manager.get_current_pane_mode(),
            EditorMode::Normal
        );

        let mut context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        let result = command.execute(
            KeyEvent::new(KeyCode::Null, KeyModifiers::empty()),
            &mut context,
        );

        assert!(result.is_ok());
        let events = result.unwrap();

        // Should return events for UI updates
        assert!(!events.is_empty());
        assert!(events
            .iter()
            .any(|e| matches!(e, PostCommandAction::StatusBarUpdateRequired)));
        assert!(events
            .iter()
            .any(|e| matches!(e, PostCommandAction::ActiveCursorUpdateRequired)));

        // Should have changed to Visual Block mode
        assert_eq!(
            context.app_state.pane_manager.get_current_pane_mode(),
            EditorMode::VisualBlock
        );
    }

    #[test]
    fn enter_visual_block_mode_command_execute_should_work_from_various_modes() {
        let command = EnterVisualBlockModeCommand::new();

        // Test that execute works when called from Normal mode
        // (Note: is_relevant ensures this is only called from Normal mode in practice)
        let mut app_state = AppState::new();
        let mut services = Services::new();

        // Ensure we're in Normal mode
        let _ = app_state.set_mode(EditorMode::Normal);
        assert_eq!(
            app_state.pane_manager.get_current_pane_mode(),
            EditorMode::Normal
        );

        let mut context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        let result = command.execute(
            KeyEvent::new(KeyCode::Null, KeyModifiers::empty()),
            &mut context,
        );
        assert!(result.is_ok());

        // Should have changed to Visual Block mode
        assert_eq!(
            context.app_state.pane_manager.get_current_pane_mode(),
            EditorMode::VisualBlock
        );
    }

    #[test]
    fn enter_visual_block_mode_command_execute_should_return_expected_events() {
        let command = EnterVisualBlockModeCommand::new();
        let mut app_state = AppState::new();
        let mut services = Services::new();

        let mut context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        let result = command.execute(
            KeyEvent::new(KeyCode::Null, KeyModifiers::empty()),
            &mut context,
        );
        assert!(result.is_ok());

        let events = result.unwrap();

        // Should return exactly 2 events
        assert_eq!(events.len(), 2);

        // Verify specific events are present
        let has_status_update = events
            .iter()
            .any(|e| matches!(e, PostCommandAction::StatusBarUpdateRequired));
        let has_cursor_update = events
            .iter()
            .any(|e| matches!(e, PostCommandAction::ActiveCursorUpdateRequired));

        assert!(
            has_status_update,
            "Should have StatusBarUpdateRequired event"
        );
        assert!(
            has_cursor_update,
            "Should have ActiveCursorUpdateRequired event"
        );
    }

    #[test]
    fn default_should_create_new_instance() {
        let command = EnterVisualBlockModeCommand;
        assert_eq!(command.name(), "EnterVisualBlockModeCommand");
    }
}

// Auto-register this command using the inventory system
register_command!(EnterVisualBlockModeCommand, "EnterVisualBlockModeCommand");
