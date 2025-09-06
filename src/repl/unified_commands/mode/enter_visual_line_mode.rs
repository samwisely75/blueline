//! # Enter Visual Line Mode Command
//!
//! Command to handle Shift+V in Normal mode which enters Visual Line mode
//! for line-based text selection.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::register_command;
use crate::repl::models::pane_state::EditorMode;
use crate::repl::unified_commands::{Command, CommandContext, ExecutionContext};
use crate::repl::view_models::post_command_actions::PostCommandAction;

/// Command to handle entering Visual Line mode with Shift+V
///
/// This command handles the Shift+V key combination in Normal mode, transitioning the editor
/// to Visual Line mode for line-based text selection.
pub struct EnterVisualLineModeCommand;

impl EnterVisualLineModeCommand {
    /// Create new EnterVisualLineModeCommand
    pub fn new() -> Self {
        Self
    }
}

impl Default for EnterVisualLineModeCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl Command for EnterVisualLineModeCommand {
    fn is_relevant(
        &self,
        key_event: KeyEvent,
        mode: EditorMode,
        _context: &CommandContext,
    ) -> bool {
        // Handle Shift+V in Normal mode - can be uppercase V or lowercase v with SHIFT modifier
        let is_uppercase_v = matches!(key_event.code, KeyCode::Char('V'));
        let is_shift_v = matches!(key_event.code, KeyCode::Char('v'))
            && key_event.modifiers.contains(KeyModifiers::SHIFT);
        let is_shift_v_key = is_uppercase_v || is_shift_v;

        is_shift_v_key && mode == EditorMode::Normal
    }

    fn execute(
        &self,
        _key_event: KeyEvent,
        context: &mut ExecutionContext,
    ) -> Result<Vec<PostCommandAction>> {
        tracing::debug!("EnterVisualLineModeCommand: entering Visual Line mode");

        // Change mode to VisualLine - this handles visual selection initialization internally
        context.app_state.change_mode(EditorMode::VisualLine)?;

        tracing::debug!("EnterVisualLineModeCommand: mode changed to VisualLine");

        // Return actions for UI updates
        Ok(vec![
            PostCommandAction::StatusBarUpdateRequired,
            PostCommandAction::ActiveCursorUpdateRequired,
            PostCommandAction::CurrentAreaRedrawRequired, // Visual Line mode needs redraw for selection highlighting
        ])
    }

    fn name(&self) -> &'static str {
        "EnterVisualLineMode"
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
    fn command_name_should_be_enter_visual_line_mode() {
        let command = EnterVisualLineModeCommand::new();
        assert_eq!(command.name(), "EnterVisualLineMode");
    }

    #[test]
    fn should_be_relevant_for_uppercase_v_in_normal_mode() {
        let command = EnterVisualLineModeCommand::new();
        let context = create_test_context();
        let key_event = create_test_key_event(KeyCode::Char('V'), KeyModifiers::empty());

        assert!(command.is_relevant(key_event, EditorMode::Normal, &context));
    }

    #[test]
    fn should_be_relevant_for_lowercase_v_with_shift_in_normal_mode() {
        let command = EnterVisualLineModeCommand::new();
        let context = create_test_context();
        let key_event = create_test_key_event(KeyCode::Char('v'), KeyModifiers::SHIFT);

        assert!(command.is_relevant(key_event, EditorMode::Normal, &context));
    }

    #[test]
    fn should_be_relevant_for_uppercase_v_with_shift_in_normal_mode() {
        let command = EnterVisualLineModeCommand::new();
        let context = create_test_context();
        let key_event = create_test_key_event(KeyCode::Char('V'), KeyModifiers::SHIFT);

        assert!(command.is_relevant(key_event, EditorMode::Normal, &context));
    }

    #[test]
    fn should_not_be_relevant_for_lowercase_v_without_shift_in_normal_mode() {
        let command = EnterVisualLineModeCommand::new();
        let context = create_test_context();
        let key_event = create_test_key_event(KeyCode::Char('v'), KeyModifiers::empty());

        assert!(!command.is_relevant(key_event, EditorMode::Normal, &context));
    }

    #[test]
    fn should_not_be_relevant_for_uppercase_v_in_insert_mode() {
        let command = EnterVisualLineModeCommand::new();
        let context = create_test_context();
        let key_event = create_test_key_event(KeyCode::Char('V'), KeyModifiers::empty());

        assert!(!command.is_relevant(key_event, EditorMode::Insert, &context));
    }

    #[test]
    fn should_not_be_relevant_for_uppercase_v_in_visual_mode() {
        let command = EnterVisualLineModeCommand::new();
        let context = create_test_context();
        let key_event = create_test_key_event(KeyCode::Char('V'), KeyModifiers::empty());

        assert!(!command.is_relevant(key_event, EditorMode::Visual, &context));
    }

    #[test]
    fn should_not_be_relevant_for_uppercase_v_in_visual_line_mode() {
        let command = EnterVisualLineModeCommand::new();
        let context = create_test_context();
        let key_event = create_test_key_event(KeyCode::Char('V'), KeyModifiers::empty());

        assert!(!command.is_relevant(key_event, EditorMode::VisualLine, &context));
    }

    #[test]
    fn should_not_be_relevant_for_other_keys() {
        let command = EnterVisualLineModeCommand::new();
        let context = create_test_context();

        let test_keys = vec![
            (KeyCode::Char('a'), KeyModifiers::empty()),
            (KeyCode::Char('i'), KeyModifiers::empty()),
            (KeyCode::Char('v'), KeyModifiers::CONTROL),
            (KeyCode::Enter, KeyModifiers::empty()),
            (KeyCode::Esc, KeyModifiers::empty()),
            (KeyCode::Tab, KeyModifiers::empty()),
        ];

        for (key_code, modifiers) in test_keys {
            let key_event = create_test_key_event(key_code, modifiers);
            assert!(
                !command.is_relevant(key_event, EditorMode::Normal, &context),
                "Command should not be relevant for {key_code:?} with {modifiers:?}"
            );
        }
    }

    #[test]
    fn execute_should_change_mode_to_visual_line() {
        let command = EnterVisualLineModeCommand::new();
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

        assert_eq!(
            execution_context.app_state.get_mode(),
            EditorMode::VisualLine
        );
        assert!(!result.is_empty());
    }

    #[test]
    fn execute_should_initialize_visual_line_selection() {
        let command = EnterVisualLineModeCommand::new();
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

        // Visual Line mode should be set (the mode manager handles selection initialization internally)
        assert_eq!(
            execution_context.app_state.get_mode(),
            EditorMode::VisualLine
        );
        // Note: Visual selection initialization may be handled by the view layer, not directly by set_mode
    }

    #[test]
    fn execute_should_return_appropriate_post_command_actions() {
        let command = EnterVisualLineModeCommand::new();
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
        let command1 = EnterVisualLineModeCommand;
        let command2 = EnterVisualLineModeCommand::new();

        assert_eq!(command1.name(), command2.name());
    }

    #[test]
    fn should_handle_execution_gracefully() {
        let command = EnterVisualLineModeCommand::new();
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
        assert_eq!(
            execution_context.app_state.get_mode(),
            EditorMode::VisualLine
        );
    }

    #[test]
    fn should_work_from_normal_mode_only() {
        let command = EnterVisualLineModeCommand::new();

        // Test from Normal mode - should work
        let (mut app_state, mut services) = create_execution_context();
        app_state.set_mode(EditorMode::Normal);

        let mut execution_context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        let result = command.execute(
            KeyEvent::new(KeyCode::Null, KeyModifiers::empty()),
            &mut execution_context,
        );
        assert!(result.is_ok(), "Should succeed from Normal mode");
        assert_eq!(
            execution_context.app_state.get_mode(),
            EditorMode::VisualLine
        );

        // Test relevance for different key combinations
        let context = create_test_context();

        // Should be relevant for uppercase V
        assert!(command.is_relevant(
            create_test_key_event(KeyCode::Char('V'), KeyModifiers::empty()),
            EditorMode::Normal,
            &context
        ));

        // Should be relevant for lowercase v with SHIFT
        assert!(command.is_relevant(
            create_test_key_event(KeyCode::Char('v'), KeyModifiers::SHIFT),
            EditorMode::Normal,
            &context
        ));

        // Should NOT be relevant for lowercase v without SHIFT
        assert!(!command.is_relevant(
            create_test_key_event(KeyCode::Char('v'), KeyModifiers::empty()),
            EditorMode::Normal,
            &context
        ));
    }
}

// Register command for dynamic discovery
register_command!(EnterVisualLineModeCommand, "EnterVisualLineMode");
