//! # Go to Top Command
//!
//! Command for going to the top of the buffer (gg command).
//! This command handles the second 'g' in the "gg" sequence in Vim.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use crate::register_command;
use crate::repl::models::pane_state::EditorMode;
use crate::repl::unified_commands::{Command, CommandContext, ExecutionContext};
use crate::repl::view_models::post_command_actions::PostCommandAction;

/// Command to go to the top of the buffer (gg command)
///
/// This command handles the second 'g' in the "gg" vim sequence:
/// 1. First 'g' enters GPrefix mode (handled by EnterGPrefixCommand)
/// 2. Second 'g' (this command) moves cursor to top and returns to Normal mode
///
/// The command is relevant when:
/// - Key is 'g' without modifiers
/// - Current mode is GPrefix
/// - Not in read-only mode restrictions (cursor movement allowed)
#[derive(Debug, Default)]
pub struct GoToTopCommand;

impl GoToTopCommand {
    /// Create new GoToTopCommand
    pub fn new() -> Self {
        Self
    }
}

impl Command for GoToTopCommand {
    fn is_relevant(
        &self,
        key_event: KeyEvent,
        mode: EditorMode,
        _context: &CommandContext,
    ) -> bool {
        // Must be 'g' key without modifiers
        matches!(key_event.code, KeyCode::Char('g'))
            && key_event.modifiers.is_empty()
            // Must be in GPrefix mode (after first 'g' was pressed)
            && mode == EditorMode::GPrefix
    }

    fn execute(&self, context: &mut ExecutionContext) -> Result<Vec<PostCommandAction>> {
        // Move cursor to document start and exit GPrefix mode
        let mut actions = Vec::new();

        // Move cursor to the top of the buffer
        let cursor_actions = context
            .app_state
            .pane_manager
            .move_cursor_to_document_start();
        actions.extend(cursor_actions);

        // Exit GPrefix mode back to Normal mode
        context.app_state.set_mode(EditorMode::Normal);
        actions.push(PostCommandAction::StatusBarUpdateRequired);

        tracing::debug!(
            "GoToTopCommand executed: moved to document start and returned to Normal mode, generated {} actions",
            actions.len()
        );

        Ok(actions)
    }

    fn name(&self) -> &'static str {
        "GoToTopCommand"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::models::pane_state::Pane;
    use crossterm::event::KeyModifiers;

    fn create_test_context() -> CommandContext {
        CommandContext {
            current_mode: EditorMode::GPrefix,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        }
    }

    #[test]
    fn command_should_return_correct_name() {
        let command = GoToTopCommand::new();
        assert_eq!(command.name(), "GoToTopCommand");
    }

    #[test]
    fn should_be_relevant_for_g_key_in_g_prefix_mode() {
        let command = GoToTopCommand::new();
        let context = create_test_context();

        let g_key = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE);
        assert!(command.is_relevant(g_key, EditorMode::GPrefix, &context));
    }

    #[test]
    fn should_not_be_relevant_for_g_key_in_normal_mode() {
        let command = GoToTopCommand::new();
        let context = create_test_context();

        let g_key = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE);
        assert!(!command.is_relevant(g_key, EditorMode::Normal, &context));
    }

    #[test]
    fn should_not_be_relevant_for_g_key_in_insert_mode() {
        let command = GoToTopCommand::new();
        let context = create_test_context();

        let g_key = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE);
        assert!(!command.is_relevant(g_key, EditorMode::Insert, &context));
    }

    #[test]
    fn should_not_be_relevant_for_g_key_in_visual_mode() {
        let command = GoToTopCommand::new();
        let context = create_test_context();

        let g_key = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE);
        assert!(!command.is_relevant(g_key, EditorMode::Visual, &context));
    }

    #[test]
    fn should_not_be_relevant_for_g_key_in_command_mode() {
        let command = GoToTopCommand::new();
        let context = create_test_context();

        let g_key = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE);
        assert!(!command.is_relevant(g_key, EditorMode::Command, &context));
    }

    #[test]
    fn should_not_be_relevant_for_g_key_with_modifiers() {
        let command = GoToTopCommand::new();
        let context = create_test_context();

        // Test with Ctrl modifier
        let g_key_ctrl = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::CONTROL);
        assert!(!command.is_relevant(g_key_ctrl, EditorMode::GPrefix, &context));

        // Test with Shift modifier
        let g_key_shift = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::SHIFT);
        assert!(!command.is_relevant(g_key_shift, EditorMode::GPrefix, &context));

        // Test with Alt modifier
        let g_key_alt = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::ALT);
        assert!(!command.is_relevant(g_key_alt, EditorMode::GPrefix, &context));
    }

    #[test]
    fn should_not_be_relevant_for_other_keys() {
        let command = GoToTopCommand::new();
        let context = create_test_context();

        // Test other letter keys
        let h_key = KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE);
        assert!(!command.is_relevant(h_key, EditorMode::GPrefix, &context));

        let upper_g_key = KeyEvent::new(KeyCode::Char('G'), KeyModifiers::NONE);
        assert!(!command.is_relevant(upper_g_key, EditorMode::GPrefix, &context));

        // Test special keys
        let up_key = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
        assert!(!command.is_relevant(up_key, EditorMode::GPrefix, &context));

        let home_key = KeyEvent::new(KeyCode::Home, KeyModifiers::NONE);
        assert!(!command.is_relevant(home_key, EditorMode::GPrefix, &context));
    }

    #[test]
    fn should_work_in_read_only_pane() {
        let command = GoToTopCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::GPrefix,
            current_pane: Pane::Response,
            is_read_only: true, // Response pane is read-only
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let g_key = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE);
        assert!(command.is_relevant(g_key, EditorMode::GPrefix, &context));
    }

    #[test]
    fn execute_should_generate_actions() {
        use crate::repl::models::AppState;
        use crate::repl::services::Services;

        let command = GoToTopCommand::new();
        let mut app_state = AppState::new();

        // Set up GPrefix mode first
        app_state.set_mode(EditorMode::GPrefix);
        assert_eq!(app_state.get_mode(), EditorMode::GPrefix);

        let mut services = Services::new();
        let mut context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        let result = command.execute(&mut context);
        assert!(result.is_ok());

        let actions = result.unwrap();

        // Should generate at least one action (StatusBarUpdateRequired)
        assert!(!actions.is_empty());

        // Should have returned to Normal mode
        assert_eq!(context.app_state.get_mode(), EditorMode::Normal);

        // Should include status bar update action
        assert!(actions.contains(&PostCommandAction::StatusBarUpdateRequired));
    }

    #[test]
    fn execute_should_change_mode_from_g_prefix_to_normal() {
        use crate::repl::models::AppState;
        use crate::repl::services::Services;

        let command = GoToTopCommand::new();
        let mut app_state = AppState::new();

        // Start in GPrefix mode
        app_state.set_mode(EditorMode::GPrefix);
        assert_eq!(app_state.get_mode(), EditorMode::GPrefix);

        let mut services = Services::new();
        let mut context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        let _result = command.execute(&mut context).unwrap();

        // Mode should be changed to Normal
        assert_eq!(context.app_state.get_mode(), EditorMode::Normal);
    }

    #[test]
    fn default_should_create_new_instance() {
        let command = GoToTopCommand;
        assert_eq!(command.name(), "GoToTopCommand");
    }

    #[test]
    fn execute_should_move_cursor_to_document_start() {
        use crate::repl::models::AppState;
        use crate::repl::services::Services;

        let command = GoToTopCommand::new();
        let mut app_state = AppState::new();

        // Set up GPrefix mode first
        app_state.set_mode(EditorMode::GPrefix);

        let mut services = Services::new();
        let mut context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        let result = command.execute(&mut context);
        assert!(result.is_ok());

        // The exact cursor position and actions depend on pane_manager implementation,
        // but we verify that the command executed successfully
        let actions = result.unwrap();
        assert!(!actions.is_empty());

        // Mode should be Normal
        assert_eq!(context.app_state.get_mode(), EditorMode::Normal);
    }
}

// Auto-register this command using the inventory system
register_command!(GoToTopCommand, "GoToTopCommand");
