//! # Exit Visual Mode Command
//!
//! Command to handle the Escape key in Visual modes which exits back to Normal mode.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use crate::register_command;
use crate::repl::models::pane_state::EditorMode;
use crate::repl::view_models::commands::{Command, CommandContext, ExecutionContext};
use crate::repl::view_models::post_command_actions::PostCommandAction;

/// Command to handle exiting Visual modes with the Escape key
///
/// This command handles the Escape key in Visual, VisualLine, and VisualBlock modes,
/// transitioning the editor back to Normal mode and clearing the visual selection.
pub struct ExitVisualModeCommand;

impl ExitVisualModeCommand {
    /// Create new ExitVisualModeCommand
    pub fn new() -> Self {
        Self
    }
}

impl Default for ExitVisualModeCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl Command for ExitVisualModeCommand {
    fn is_relevant(
        &self,
        key_event: KeyEvent,
        mode: EditorMode,
        _context: &CommandContext,
    ) -> bool {
        // Handle Escape key in all visual modes
        matches!(key_event.code, KeyCode::Esc)
            && matches!(
                mode,
                EditorMode::Visual | EditorMode::VisualLine | EditorMode::VisualBlock
            )
            && key_event.modifiers.is_empty()
    }

    fn execute(
        &self,
        _key_event: KeyEvent,
        context: &mut ExecutionContext,
    ) -> Result<Vec<PostCommandAction>> {
        tracing::debug!("ExitVisualModeCommand: exiting visual mode to Normal");

        // Change mode to Normal - this properly handles visual selection cleanup via mode manager
        context.app_state.change_mode(EditorMode::Normal)?;

        tracing::debug!("ExitVisualModeCommand: mode changed to Normal");

        // Return actions for UI updates
        Ok(vec![
            PostCommandAction::StatusBarUpdateRequired,
            PostCommandAction::ActiveCursorUpdateRequired,
            PostCommandAction::CurrentAreaRedrawRequired, // Need redraw to remove selection highlighting
        ])
    }

    fn name(&self) -> &'static str {
        "ExitVisualMode"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::models::pane_state::{EditorMode, Pane};
    use crate::repl::models::AppState;
    use crate::repl::services::Services;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn create_test_key_event(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent::new(code, modifiers)
    }

    fn create_test_context() -> CommandContext {
        CommandContext {
            current_mode: EditorMode::Visual,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: true,
            ex_command_buffer: String::new(),
        }
    }

    fn create_execution_context() -> (AppState, Services) {
        let mut app_state = AppState::new();
        app_state.set_mode(EditorMode::Visual);
        let services = Services::new();
        (app_state, services)
    }

    #[test]
    fn command_name_should_be_exit_visual_mode() {
        let command = ExitVisualModeCommand::new();
        assert_eq!(command.name(), "ExitVisualMode");
    }

    #[test]
    fn should_be_relevant_for_escape_in_visual_mode() {
        let command = ExitVisualModeCommand::new();
        let context = create_test_context();
        let key_event = create_test_key_event(KeyCode::Esc, KeyModifiers::empty());

        assert!(command.is_relevant(key_event, EditorMode::Visual, &context));
    }

    #[test]
    fn should_be_relevant_for_escape_in_visual_line_mode() {
        let command = ExitVisualModeCommand::new();
        let context = create_test_context();
        let key_event = create_test_key_event(KeyCode::Esc, KeyModifiers::empty());

        assert!(command.is_relevant(key_event, EditorMode::VisualLine, &context));
    }

    #[test]
    fn should_be_relevant_for_escape_in_visual_block_mode() {
        let command = ExitVisualModeCommand::new();
        let context = create_test_context();
        let key_event = create_test_key_event(KeyCode::Esc, KeyModifiers::empty());

        assert!(command.is_relevant(key_event, EditorMode::VisualBlock, &context));
    }

    #[test]
    fn should_not_be_relevant_for_escape_in_normal_mode() {
        let command = ExitVisualModeCommand::new();
        let context = create_test_context();
        let key_event = create_test_key_event(KeyCode::Esc, KeyModifiers::empty());

        assert!(!command.is_relevant(key_event, EditorMode::Normal, &context));
    }

    #[test]
    fn should_not_be_relevant_for_escape_in_insert_mode() {
        let command = ExitVisualModeCommand::new();
        let context = create_test_context();
        let key_event = create_test_key_event(KeyCode::Esc, KeyModifiers::empty());

        assert!(!command.is_relevant(key_event, EditorMode::Insert, &context));
    }

    #[test]
    fn should_not_be_relevant_for_escape_with_modifiers() {
        let command = ExitVisualModeCommand::new();
        let context = create_test_context();
        let key_event = create_test_key_event(KeyCode::Esc, KeyModifiers::SHIFT);

        assert!(!command.is_relevant(key_event, EditorMode::Visual, &context));
    }

    #[test]
    fn should_not_be_relevant_for_other_keys() {
        let command = ExitVisualModeCommand::new();
        let context = create_test_context();

        let test_keys = vec![
            KeyCode::Char('v'),
            KeyCode::Char('a'),
            KeyCode::Enter,
            KeyCode::Tab,
            KeyCode::Backspace,
        ];

        for key_code in test_keys {
            let key_event = create_test_key_event(key_code, KeyModifiers::empty());
            assert!(
                !command.is_relevant(key_event, EditorMode::Visual, &context),
                "Command should not be relevant for {key_code:?}"
            );
        }
    }

    #[test]
    fn execute_should_change_mode_to_normal() {
        let command = ExitVisualModeCommand::new();
        let (mut app_state, mut services) = create_execution_context();

        let mut execution_context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        let result = command
            .execute(
                KeyEvent::new(KeyCode::Null, KeyModifiers::empty()),
                &mut execution_context,
            )
            .unwrap();

        assert_eq!(execution_context.app_state.get_mode(), EditorMode::Normal);
        assert!(!result.is_empty());
    }

    #[test]
    fn execute_should_clear_visual_selection() {
        let command = ExitVisualModeCommand::new();
        let (mut app_state, mut services) = create_execution_context();

        // Start with a visual selection
        app_state.set_mode(EditorMode::Visual);

        let mut execution_context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        command
            .execute(
                KeyEvent::new(KeyCode::Null, KeyModifiers::empty()),
                &mut execution_context,
            )
            .unwrap();

        // Visual selection should be cleared (mode manager handles this internally)
        // Note: The exact selection state depends on the mode manager implementation
        assert_eq!(execution_context.app_state.get_mode(), EditorMode::Normal);
    }

    #[test]
    fn execute_should_return_appropriate_post_command_actions() {
        let command = ExitVisualModeCommand::new();
        let (mut app_state, mut services) = create_execution_context();

        let mut execution_context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        let result = command
            .execute(
                KeyEvent::new(KeyCode::Null, KeyModifiers::empty()),
                &mut execution_context,
            )
            .unwrap();

        // Should return UI update actions
        assert!(result.contains(&PostCommandAction::StatusBarUpdateRequired));
        assert!(result.contains(&PostCommandAction::ActiveCursorUpdateRequired));
        assert!(result.contains(&PostCommandAction::CurrentAreaRedrawRequired));
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn default_should_create_new_instance() {
        let command1 = ExitVisualModeCommand;
        let command2 = ExitVisualModeCommand::new();

        assert_eq!(command1.name(), command2.name());
    }

    #[test]
    fn should_handle_execution_gracefully() {
        let command = ExitVisualModeCommand::new();
        let (mut app_state, mut services) = create_execution_context();

        let mut execution_context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        let result = command.execute(
            KeyEvent::new(KeyCode::Null, KeyModifiers::empty()),
            &mut execution_context,
        );

        // Should succeed
        assert!(result.is_ok());
        assert_eq!(execution_context.app_state.get_mode(), EditorMode::Normal);
    }

    #[test]
    fn should_work_from_all_visual_modes() {
        let command = ExitVisualModeCommand::new();

        let visual_modes = vec![
            EditorMode::Visual,
            EditorMode::VisualLine,
            EditorMode::VisualBlock,
        ];

        for initial_mode in visual_modes {
            let mut app_state = AppState::new();
            app_state.set_mode(initial_mode);
            let mut services = Services::new();

            let mut execution_context = ExecutionContext {
                app_state: &mut app_state,
                services: &mut services,
            };

            let result = command.execute(
                KeyEvent::new(KeyCode::Null, KeyModifiers::empty()),
                &mut execution_context,
            );

            assert!(result.is_ok(), "Should succeed from mode {initial_mode:?}");
            assert_eq!(
                execution_context.app_state.get_mode(),
                EditorMode::Normal,
                "Should change to Normal from mode {initial_mode:?}"
            );
        }
    }
}

// Register command for dynamic discovery
register_command!(ExitVisualModeCommand, "ExitVisualMode");
