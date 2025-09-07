//! # Switch Pane Command
//!
//! Command to handle switching between request and response panes using Tab key.
//! This provides easy navigation between different areas of the application.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use crate::register_command;
use crate::repl::models::pane_state::EditorMode;
use crate::repl::view_models::commands::{Command, CommandContext, ExecutionContext};
use crate::repl::view_models::post_command_actions::PostCommandAction;

/// Command to switch between request and response panes
///
/// This command handles Tab key in Normal mode to toggle
/// between panes, providing easy navigation between editor areas.
pub struct SwitchPaneCommand;

impl SwitchPaneCommand {
    /// Create new SwitchPaneCommand
    pub fn new() -> Self {
        Self
    }
}

impl Default for SwitchPaneCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl Command for SwitchPaneCommand {
    fn is_relevant(
        &self,
        key_event: KeyEvent,
        mode: EditorMode,
        _context: &CommandContext,
    ) -> bool {
        // Handle Tab key in Normal mode for pane switching
        matches!(key_event.code, KeyCode::Tab)
            && mode == EditorMode::Normal
            && key_event.modifiers.is_empty()
    }

    fn execute(
        &self,
        _key_event: KeyEvent,
        context: &mut ExecutionContext,
    ) -> Result<Vec<PostCommandAction>> {
        // Get current pane for logging
        let current_pane = context.app_state.get_current_pane();

        tracing::debug!("SwitchPaneCommand: switching from {:?} pane", current_pane);

        // Switch to the other pane
        context.app_state.switch_to_other_pane();

        let new_pane = context.app_state.get_current_pane();
        tracing::debug!("SwitchPaneCommand: switched to {:?} pane", new_pane);

        // Command determines what view updates are needed for pane switching
        Ok(vec![
            PostCommandAction::FocusSwitched,
            PostCommandAction::StatusBarUpdateRequired,
            PostCommandAction::ActiveCursorUpdateRequired,
        ])
    }

    fn name(&self) -> &'static str {
        "SwitchPaneCommand"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::models::pane_state::Pane;
    use crate::repl::models::AppState;
    use crate::repl::services::Services;
    use crossterm::event::KeyModifiers;

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
    fn switch_pane_command_should_return_correct_name() {
        let command = SwitchPaneCommand::new();
        assert_eq!(command.name(), "SwitchPaneCommand");
    }

    #[test]
    fn switch_pane_command_should_be_relevant_for_tab_in_normal_mode() {
        let command = SwitchPaneCommand::new();
        let context = create_test_context();
        let key_event = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);

        assert!(command.is_relevant(key_event, EditorMode::Normal, &context));
    }

    #[test]
    fn switch_pane_command_should_not_be_relevant_in_insert_mode() {
        let command = SwitchPaneCommand::new();
        let context = create_test_context();
        let key_event = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);

        assert!(!command.is_relevant(key_event, EditorMode::Insert, &context));
    }

    #[test]
    fn switch_pane_command_should_not_be_relevant_in_visual_modes() {
        let command = SwitchPaneCommand::new();
        let context = create_test_context();
        let key_event = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);

        let visual_modes = vec![
            EditorMode::Visual,
            EditorMode::VisualLine,
            EditorMode::VisualBlock,
        ];

        for mode in visual_modes {
            assert!(
                !command.is_relevant(key_event, mode, &context),
                "Should not be relevant in {mode:?} mode"
            );
        }
    }

    #[test]
    fn switch_pane_command_should_not_be_relevant_in_command_mode() {
        let command = SwitchPaneCommand::new();
        let context = create_test_context();
        let key_event = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);

        assert!(!command.is_relevant(key_event, EditorMode::Command, &context));
    }

    #[test]
    fn switch_pane_command_should_not_be_relevant_for_other_keys() {
        let command = SwitchPaneCommand::new();
        let context = create_test_context();

        let other_keys = vec![
            KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE),
        ];

        for key_event in other_keys {
            assert!(
                !command.is_relevant(key_event, EditorMode::Normal, &context),
                "Should not be relevant for key: {key_event:?}"
            );
        }
    }

    #[test]
    fn switch_pane_command_should_not_be_relevant_for_tab_with_modifiers() {
        let command = SwitchPaneCommand::new();
        let context = create_test_context();

        let modified_tab_keys = vec![
            KeyEvent::new(KeyCode::Tab, KeyModifiers::SHIFT),
            KeyEvent::new(KeyCode::Tab, KeyModifiers::CONTROL),
            KeyEvent::new(KeyCode::Tab, KeyModifiers::ALT),
            KeyEvent::new(KeyCode::Tab, KeyModifiers::CONTROL | KeyModifiers::SHIFT),
        ];

        for key_event in modified_tab_keys {
            assert!(
                !command.is_relevant(key_event, EditorMode::Normal, &context),
                "Should not be relevant for modified Tab key: {key_event:?}"
            );
        }
    }

    #[test]
    fn switch_pane_execute_should_switch_from_request_to_response() {
        let command = SwitchPaneCommand::new();
        let mut app_state = AppState::new();
        let mut services = Services::new();

        // Verify initial state is Request pane
        assert_eq!(app_state.get_current_pane(), Pane::Request);

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

        // Should return appropriate events for pane switching
        assert!(!events.is_empty());
        assert!(events
            .iter()
            .any(|e| matches!(e, PostCommandAction::FocusSwitched)));
        assert!(events
            .iter()
            .any(|e| matches!(e, PostCommandAction::StatusBarUpdateRequired)));
        assert!(events
            .iter()
            .any(|e| matches!(e, PostCommandAction::ActiveCursorUpdateRequired)));

        // Should have switched to Response pane
        assert_eq!(context.app_state.get_current_pane(), Pane::Response);
    }

    #[test]
    fn switch_pane_execute_should_switch_from_response_to_request() {
        let command = SwitchPaneCommand::new();
        let mut app_state = AppState::new();
        let mut services = Services::new();

        // Set initial state to Response pane
        app_state.switch_to_response_pane();
        assert_eq!(app_state.get_current_pane(), Pane::Response);

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

        // Should return appropriate events for pane switching
        assert!(!events.is_empty());
        assert!(events
            .iter()
            .any(|e| matches!(e, PostCommandAction::FocusSwitched)));

        // Should have switched to Request pane
        assert_eq!(context.app_state.get_current_pane(), Pane::Request);
    }

    #[test]
    fn switch_pane_command_should_work_regardless_of_context_read_only_state() {
        let command = SwitchPaneCommand::new();
        let read_only_context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Response,
            is_read_only: true,
            has_selection: false,
            ex_command_buffer: String::new(),
        };
        let key_event = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);

        // Should still be relevant even in read-only context for pane switching
        assert!(command.is_relevant(key_event, EditorMode::Normal, &read_only_context));
    }

    #[test]
    fn switch_pane_command_should_work_with_selection() {
        let command = SwitchPaneCommand::new();
        let context_with_selection = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: true,
            ex_command_buffer: String::new(),
        };
        let key_event = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);

        // Should still be relevant even with selection for pane switching
        assert!(command.is_relevant(key_event, EditorMode::Normal, &context_with_selection));
    }

    #[test]
    fn default_should_create_new_instance() {
        let command = SwitchPaneCommand;
        assert_eq!(command.name(), "SwitchPaneCommand");
    }
}

// Auto-register this command using the inventory system
register_command!(SwitchPaneCommand, "SwitchPaneCommand");
