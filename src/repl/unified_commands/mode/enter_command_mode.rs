//! # Enter Command Mode Command
//!
//! Command to handle the ':' (colon) key in Normal and Visual modes which
//! enters Command mode for executing Ex commands.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use crate::register_command;
use crate::repl::models::pane_state::EditorMode;
use crate::repl::unified_commands::{Command, CommandContext, ExecutionContext};
use crate::repl::view_models::post_command_actions::PostCommandAction;

/// Command to handle entering Command mode with the ':' key
///
/// This command handles the ':' key in Normal, Visual, VisualLine, and VisualBlock modes,
/// which transitions the editor to Command mode for executing Ex commands.
pub struct EnterCommandModeCommand;

impl EnterCommandModeCommand {
    /// Create new EnterCommandModeCommand
    pub fn new() -> Self {
        Self
    }
}

impl Default for EnterCommandModeCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl Command for EnterCommandModeCommand {
    fn is_relevant(
        &self,
        key_event: KeyEvent,
        mode: EditorMode,
        _context: &CommandContext,
    ) -> bool {
        // Handle ':' key (colon) in appropriate modes
        matches!(key_event.code, KeyCode::Char(':'))
            && matches!(
                mode,
                EditorMode::Normal
                    | EditorMode::Visual
                    | EditorMode::VisualLine
                    | EditorMode::VisualBlock
            )
            && key_event.modifiers.is_empty()
    }

    fn execute(&self, context: &mut ExecutionContext) -> Result<Vec<PostCommandAction>> {
        tracing::debug!("EnterCommandModeCommand: entering Command mode");

        // Change mode to Command mode
        let _mode_change = context.app_state.set_mode(EditorMode::Command);

        tracing::debug!("EnterCommandModeCommand: mode changed to Command");

        // Return appropriate PostCommandActions
        Ok(vec![PostCommandAction::StatusBarUpdateRequired])
    }

    fn name(&self) -> &'static str {
        "EnterCommandModeCommand"
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
    fn enter_command_mode_command_should_return_correct_name() {
        let command = EnterCommandModeCommand::new();
        assert_eq!(command.name(), "EnterCommandModeCommand");
    }

    #[test]
    fn enter_command_mode_command_should_be_relevant_for_colon_in_normal_mode() {
        let command = EnterCommandModeCommand::new();
        let context = create_test_context();
        let key_event = KeyEvent::new(KeyCode::Char(':'), KeyModifiers::NONE);

        assert!(command.is_relevant(key_event, EditorMode::Normal, &context));
    }

    #[test]
    fn enter_command_mode_command_should_be_relevant_for_colon_in_visual_mode() {
        let command = EnterCommandModeCommand::new();
        let context = create_test_context();
        let key_event = KeyEvent::new(KeyCode::Char(':'), KeyModifiers::NONE);

        assert!(command.is_relevant(key_event, EditorMode::Visual, &context));
    }

    #[test]
    fn enter_command_mode_command_should_be_relevant_for_colon_in_visual_line_mode() {
        let command = EnterCommandModeCommand::new();
        let context = create_test_context();
        let key_event = KeyEvent::new(KeyCode::Char(':'), KeyModifiers::NONE);

        assert!(command.is_relevant(key_event, EditorMode::VisualLine, &context));
    }

    #[test]
    fn enter_command_mode_command_should_be_relevant_for_colon_in_visual_block_mode() {
        let command = EnterCommandModeCommand::new();
        let context = create_test_context();
        let key_event = KeyEvent::new(KeyCode::Char(':'), KeyModifiers::NONE);

        assert!(command.is_relevant(key_event, EditorMode::VisualBlock, &context));
    }

    #[test]
    fn enter_command_mode_command_should_not_be_relevant_for_colon_in_insert_mode() {
        let command = EnterCommandModeCommand::new();
        let context = create_test_context();
        let key_event = KeyEvent::new(KeyCode::Char(':'), KeyModifiers::NONE);

        assert!(!command.is_relevant(key_event, EditorMode::Insert, &context));
    }

    #[test]
    fn enter_command_mode_command_should_not_be_relevant_for_colon_in_command_mode() {
        let command = EnterCommandModeCommand::new();
        let context = create_test_context();
        let key_event = KeyEvent::new(KeyCode::Char(':'), KeyModifiers::NONE);

        assert!(!command.is_relevant(key_event, EditorMode::Command, &context));
    }

    #[test]
    fn enter_command_mode_command_should_not_be_relevant_with_modifiers() {
        let command = EnterCommandModeCommand::new();
        let context = create_test_context();

        let modified_keys = vec![
            KeyEvent::new(KeyCode::Char(':'), KeyModifiers::SHIFT),
            KeyEvent::new(KeyCode::Char(':'), KeyModifiers::CONTROL),
            KeyEvent::new(KeyCode::Char(':'), KeyModifiers::ALT),
            KeyEvent::new(
                KeyCode::Char(':'),
                KeyModifiers::SHIFT | KeyModifiers::CONTROL,
            ),
        ];

        for key_event in modified_keys {
            assert!(
                !command.is_relevant(key_event, EditorMode::Normal, &context),
                "Should not be relevant for modified ':' key: {key_event:?}"
            );
        }
    }

    #[test]
    fn enter_command_mode_command_should_not_be_relevant_for_other_keys() {
        let command = EnterCommandModeCommand::new();
        let context = create_test_context();

        let other_keys = vec![
            KeyEvent::new(KeyCode::Char(';'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE),
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
    fn enter_command_mode_command_should_handle_different_panes() {
        let command = EnterCommandModeCommand::new();

        let panes = vec![Pane::Request, Pane::Response];

        for pane in panes {
            let context = CommandContext {
                current_mode: EditorMode::Normal,
                current_pane: pane,
                is_read_only: false,
                has_selection: false,
                ex_command_buffer: String::new(),
            };
            let key_event = KeyEvent::new(KeyCode::Char(':'), KeyModifiers::NONE);

            assert!(
                command.is_relevant(key_event, EditorMode::Normal, &context),
                "Should be relevant in {pane:?} pane"
            );
        }
    }

    #[test]
    fn enter_command_mode_command_should_handle_read_only_context() {
        let command = EnterCommandModeCommand::new();
        let read_only_context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Response,
            is_read_only: true, // Read-only context
            has_selection: false,
            ex_command_buffer: String::new(),
        };
        let key_event = KeyEvent::new(KeyCode::Char(':'), KeyModifiers::NONE);

        // Should still be relevant even in read-only context
        assert!(command.is_relevant(key_event, EditorMode::Normal, &read_only_context));
    }

    #[test]
    fn enter_command_mode_command_should_work_with_selection() {
        let command = EnterCommandModeCommand::new();
        let context_with_selection = CommandContext {
            current_mode: EditorMode::Visual,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: true, // Has active selection
            ex_command_buffer: String::new(),
        };
        let key_event = KeyEvent::new(KeyCode::Char(':'), KeyModifiers::NONE);

        // Should be relevant with selection in Visual mode
        assert!(command.is_relevant(key_event, EditorMode::Visual, &context_with_selection));
    }

    #[test]
    fn enter_command_mode_execute_should_change_mode_to_command() {
        let command = EnterCommandModeCommand::new();
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

        let result = command.execute(&mut context);

        assert!(result.is_ok());
        let events = result.unwrap();

        // Should return events for status update
        assert!(!events.is_empty());
        assert!(events
            .iter()
            .any(|e| matches!(e, PostCommandAction::StatusBarUpdateRequired)));

        // Should have changed to Command mode
        assert_eq!(
            context.app_state.pane_manager.get_current_pane_mode(),
            EditorMode::Command
        );
    }

    #[test]
    fn enter_command_mode_execute_should_work_from_visual_mode() {
        let command = EnterCommandModeCommand::new();
        let mut app_state = AppState::new();
        let mut services = Services::new();

        // Set initial mode to Visual
        let _ = app_state.set_mode(EditorMode::Visual);
        assert_eq!(
            app_state.pane_manager.get_current_pane_mode(),
            EditorMode::Visual
        );

        let mut context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        let result = command.execute(&mut context);

        assert!(result.is_ok());
        let events = result.unwrap();

        // Should return appropriate events
        assert!(!events.is_empty());
        assert!(events
            .iter()
            .any(|e| matches!(e, PostCommandAction::StatusBarUpdateRequired)));

        // Should have changed from Visual to Command mode
        assert_eq!(
            context.app_state.pane_manager.get_current_pane_mode(),
            EditorMode::Command
        );
    }

    #[test]
    fn enter_command_mode_execute_should_handle_different_initial_modes() {
        let command = EnterCommandModeCommand::new();

        let initial_modes = vec![
            EditorMode::Normal,
            EditorMode::Visual,
            EditorMode::VisualLine,
            EditorMode::VisualBlock,
        ];

        for initial_mode in initial_modes {
            let mut app_state = AppState::new();
            let mut services = Services::new();

            // Set the initial mode
            let _ = app_state.set_mode(initial_mode);
            assert_eq!(app_state.pane_manager.get_current_pane_mode(), initial_mode);

            let mut context = ExecutionContext {
                app_state: &mut app_state,
                services: &mut services,
            };

            let result = command.execute(&mut context);
            assert!(
                result.is_ok(),
                "Execute should succeed from {initial_mode:?} mode"
            );

            // Should have changed to Command mode
            assert_eq!(
                context.app_state.pane_manager.get_current_pane_mode(),
                EditorMode::Command,
                "Should transition from {initial_mode:?} to Command mode"
            );
        }
    }

    #[test]
    fn enter_command_mode_execute_should_return_expected_events() {
        let command = EnterCommandModeCommand::new();
        let mut app_state = AppState::new();
        let mut services = Services::new();

        let mut context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        let result = command.execute(&mut context);
        assert!(result.is_ok());

        let events = result.unwrap();

        // Should return exactly 1 event
        assert_eq!(events.len(), 1);

        // Verify specific events are present
        let has_status_update = events
            .iter()
            .any(|e| matches!(e, PostCommandAction::StatusBarUpdateRequired));

        assert!(
            has_status_update,
            "Should have StatusBarUpdateRequired event"
        );
    }

    #[test]
    fn default_should_create_new_instance() {
        let command = EnterCommandModeCommand;
        assert_eq!(command.name(), "EnterCommandModeCommand");
    }
}

// Auto-register this command using the inventory system
register_command!(EnterCommandModeCommand, "EnterCommandModeCommand");
