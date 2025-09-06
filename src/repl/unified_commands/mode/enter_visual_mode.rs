//! # Enter Visual Mode Command
//!
//! Command to handle the 'v' key in Normal mode which enters Visual mode
//! for character-based text selection.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use crate::register_command;
use crate::repl::models::pane_state::EditorMode;
use crate::repl::unified_commands::{Command, CommandContext, ExecutionContext};
use crate::repl::view_models::post_command_actions::PostCommandAction;

/// Command to handle entering Visual mode with the 'v' key
///
/// This command handles the 'v' key in Normal mode, transitioning the editor
/// to Visual mode for character-based text selection.
pub struct EnterVisualModeCommand;

impl EnterVisualModeCommand {
    /// Create new EnterVisualModeCommand
    pub fn new() -> Self {
        Self
    }
}

impl Default for EnterVisualModeCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl Command for EnterVisualModeCommand {
    fn is_relevant(
        &self,
        key_event: KeyEvent,
        mode: EditorMode,
        _context: &CommandContext,
    ) -> bool {
        // Handle 'v' key in Normal mode only
        matches!(key_event.code, KeyCode::Char('v'))
            && mode == EditorMode::Normal
            && key_event.modifiers.is_empty()
    }

    fn execute(
        &self,
        _key_event: KeyEvent,
        context: &mut ExecutionContext,
    ) -> Result<Vec<PostCommandAction>> {
        tracing::debug!("EnterVisualModeCommand: entering Visual mode");

        // Change mode to Visual - this properly handles visual selection initialization via mode manager
        context.app_state.change_mode(EditorMode::Visual)?;

        tracing::debug!("EnterVisualModeCommand: mode changed to Visual");

        // Return actions for UI updates
        Ok(vec![
            PostCommandAction::StatusBarUpdateRequired,
            PostCommandAction::ActiveCursorUpdateRequired,
            PostCommandAction::CurrentAreaRedrawRequired, // Visual mode needs redraw for selection highlighting
        ])
    }

    fn name(&self) -> &'static str {
        "EnterVisualMode"
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
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        }
    }

    fn create_execution_context() -> (AppState, Services) {
        let app_state = AppState::new();
        let services = Services::new();
        (app_state, services)
    }

    #[test]
    fn command_name_should_be_enter_visual_mode() {
        let command = EnterVisualModeCommand::new();
        assert_eq!(command.name(), "EnterVisualMode");
    }

    #[test]
    fn should_be_relevant_for_v_key_in_normal_mode() {
        let command = EnterVisualModeCommand::new();
        let context = create_test_context();
        let key_event = create_test_key_event(KeyCode::Char('v'), KeyModifiers::empty());

        assert!(command.is_relevant(key_event, EditorMode::Normal, &context));
    }

    #[test]
    fn should_not_be_relevant_for_v_key_in_insert_mode() {
        let command = EnterVisualModeCommand::new();
        let context = create_test_context();
        let key_event = create_test_key_event(KeyCode::Char('v'), KeyModifiers::empty());

        assert!(!command.is_relevant(key_event, EditorMode::Insert, &context));
    }

    #[test]
    fn should_not_be_relevant_for_v_key_in_visual_mode() {
        let command = EnterVisualModeCommand::new();
        let context = create_test_context();
        let key_event = create_test_key_event(KeyCode::Char('v'), KeyModifiers::empty());

        assert!(!command.is_relevant(key_event, EditorMode::Visual, &context));
    }

    #[test]
    fn should_not_be_relevant_for_v_key_with_modifiers() {
        let command = EnterVisualModeCommand::new();
        let context = create_test_context();
        let key_event = create_test_key_event(KeyCode::Char('v'), KeyModifiers::SHIFT);

        assert!(!command.is_relevant(key_event, EditorMode::Normal, &context));
    }

    #[test]
    fn should_not_be_relevant_for_uppercase_v() {
        let command = EnterVisualModeCommand::new();
        let context = create_test_context();
        let key_event = create_test_key_event(KeyCode::Char('V'), KeyModifiers::empty());

        assert!(!command.is_relevant(key_event, EditorMode::Normal, &context));
    }

    #[test]
    fn should_not_be_relevant_for_other_keys() {
        let command = EnterVisualModeCommand::new();
        let context = create_test_context();

        let test_keys = vec![
            KeyCode::Char('a'),
            KeyCode::Char('i'),
            KeyCode::Enter,
            KeyCode::Esc,
            KeyCode::Tab,
        ];

        for key_code in test_keys {
            let key_event = create_test_key_event(key_code, KeyModifiers::empty());
            assert!(
                !command.is_relevant(key_event, EditorMode::Normal, &context),
                "Command should not be relevant for {key_code:?}"
            );
        }
    }

    #[test]
    fn execute_should_change_mode_to_visual() {
        let command = EnterVisualModeCommand::new();
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

        assert_eq!(execution_context.app_state.get_mode(), EditorMode::Visual);
        assert!(!result.is_empty());
    }

    #[test]
    fn execute_should_initialize_visual_selection() {
        let command = EnterVisualModeCommand::new();
        let (mut app_state, mut services) = create_execution_context();

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

        // Visual selection should be initialized (the mode manager handles this internally)
        assert!(execution_context.app_state.has_visual_selection());
    }

    #[test]
    fn execute_should_return_appropriate_post_command_actions() {
        let command = EnterVisualModeCommand::new();
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
        let command1 = EnterVisualModeCommand;
        let command2 = EnterVisualModeCommand::new();

        assert_eq!(command1.name(), command2.name());
    }

    #[test]
    fn should_handle_execution_gracefully() {
        let command = EnterVisualModeCommand::new();
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
        assert_eq!(execution_context.app_state.get_mode(), EditorMode::Visual);
    }
}

// Register command for dynamic discovery
register_command!(EnterVisualModeCommand, "EnterVisualMode");
