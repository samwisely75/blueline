//! # Go to Bottom Command
//!
//! Command for going to the bottom of the buffer (G command).
//! This command responds to uppercase 'G' (Shift+G) in Normal mode.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::register_command;
use crate::repl::models::pane_state::EditorMode;
use crate::repl::view_models::commands::{Command, CommandContext, ExecutionContext};
use crate::repl::view_models::post_command_actions::PostCommandAction;

/// Command to go to the bottom of the buffer (G command)
///
/// This command responds to 'G' key (uppercase G, Shift+G) in Normal mode
/// to move the cursor to the end of the buffer.
///
/// The command is relevant when:
/// - Key is 'G' without modifiers OR 'g' with SHIFT modifier OR 'G' with SHIFT modifier
/// - Current mode is Normal or Visual (navigation modes)
/// - Cursor movement is allowed (not restricted by read-only context)
///
/// Note: Different terminals send different key combinations for Shift+G:
/// - Some send KeyCode::Char('G') with no modifiers
/// - Some send KeyCode::Char('g') with SHIFT modifier
/// - Some send KeyCode::Char('G') with SHIFT modifier
///   This command handles all variations for maximum terminal compatibility.
#[derive(Debug, Default)]
pub struct GoToBottomCommand;

impl GoToBottomCommand {
    /// Create new GoToBottomCommand
    pub fn new() -> Self {
        Self
    }
}

impl Command for GoToBottomCommand {
    fn is_relevant(
        &self,
        key_event: KeyEvent,
        mode: EditorMode,
        _context: &CommandContext,
    ) -> bool {
        // Must be in a navigation mode (Normal or Visual)
        let in_navigation_mode = matches!(mode, EditorMode::Normal | EditorMode::Visual);

        // Must not be in read-only context restrictions (cursor movement allowed everywhere)
        let movement_allowed = true; // Go to bottom is always allowed, even in read-only panes

        // Handle different ways terminals send Shift+G
        let is_g_key =
            // Case 1: Uppercase 'G' without modifiers
            (matches!(key_event.code, KeyCode::Char('G')) && key_event.modifiers.is_empty())
            // Case 2: lowercase 'g' with SHIFT modifier only (no other modifiers)
            || (matches!(key_event.code, KeyCode::Char('g'))
                && key_event.modifiers == KeyModifiers::SHIFT)
            // Case 3: uppercase 'G' with SHIFT modifier only (no other modifiers)
            || (matches!(key_event.code, KeyCode::Char('G'))
                && key_event.modifiers == KeyModifiers::SHIFT);

        in_navigation_mode && movement_allowed && is_g_key
    }

    fn execute(
        &self,
        _key_event: KeyEvent,
        context: &mut ExecutionContext,
    ) -> Result<Vec<PostCommandAction>> {
        let mut actions = Vec::new();

        // Move cursor to document end
        let cursor_actions = context.app_state.pane_manager.move_cursor_to_document_end();
        actions.extend(cursor_actions);

        // No mode change needed - stay in current mode (Normal or Visual)

        actions.push(PostCommandAction::StatusBarUpdateRequired);

        tracing::debug!(
            "GoToBottomCommand executed: moved to document end, generated {} actions",
            actions.len()
        );

        Ok(actions)
    }

    fn name(&self) -> &'static str {
        "GoToBottomCommand"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::models::pane_state::Pane;

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
    fn command_should_return_correct_name() {
        let command = GoToBottomCommand::new();
        assert_eq!(command.name(), "GoToBottomCommand");
    }

    #[test]
    fn should_be_relevant_for_uppercase_g_key_in_normal_mode() {
        let command = GoToBottomCommand::new();
        let context = create_test_context();

        let uppercase_g_key = KeyEvent::new(KeyCode::Char('G'), KeyModifiers::NONE);
        assert!(command.is_relevant(uppercase_g_key, EditorMode::Normal, &context));
    }

    #[test]
    fn should_be_relevant_for_lowercase_g_with_shift_in_normal_mode() {
        let command = GoToBottomCommand::new();
        let context = create_test_context();

        let g_key_shift = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::SHIFT);
        assert!(command.is_relevant(g_key_shift, EditorMode::Normal, &context));
    }

    #[test]
    fn should_be_relevant_for_uppercase_g_with_shift_in_normal_mode() {
        let command = GoToBottomCommand::new();
        let context = create_test_context();

        let uppercase_g_key_shift = KeyEvent::new(KeyCode::Char('G'), KeyModifiers::SHIFT);
        assert!(command.is_relevant(uppercase_g_key_shift, EditorMode::Normal, &context));
    }

    #[test]
    fn should_be_relevant_for_uppercase_g_key_in_visual_mode() {
        let command = GoToBottomCommand::new();
        let context = create_test_context();

        let uppercase_g_key = KeyEvent::new(KeyCode::Char('G'), KeyModifiers::NONE);
        assert!(command.is_relevant(uppercase_g_key, EditorMode::Visual, &context));
    }

    #[test]
    fn should_not_be_relevant_for_lowercase_g_without_shift_in_normal_mode() {
        let command = GoToBottomCommand::new();
        let context = create_test_context();

        let lowercase_g_key = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE);
        assert!(!command.is_relevant(lowercase_g_key, EditorMode::Normal, &context));
    }

    #[test]
    fn should_not_be_relevant_for_g_key_in_insert_mode() {
        let command = GoToBottomCommand::new();
        let context = create_test_context();

        let uppercase_g_key = KeyEvent::new(KeyCode::Char('G'), KeyModifiers::NONE);
        assert!(!command.is_relevant(uppercase_g_key, EditorMode::Insert, &context));
    }

    #[test]
    fn should_not_be_relevant_for_g_key_in_command_mode() {
        let command = GoToBottomCommand::new();
        let context = create_test_context();

        let uppercase_g_key = KeyEvent::new(KeyCode::Char('G'), KeyModifiers::NONE);
        assert!(!command.is_relevant(uppercase_g_key, EditorMode::Command, &context));
    }

    #[test]
    fn should_not_be_relevant_for_g_key_in_g_prefix_mode() {
        let command = GoToBottomCommand::new();
        let context = create_test_context();

        let uppercase_g_key = KeyEvent::new(KeyCode::Char('G'), KeyModifiers::NONE);
        assert!(!command.is_relevant(uppercase_g_key, EditorMode::GPrefix, &context));
    }

    #[test]
    fn should_not_be_relevant_for_g_key_with_ctrl_modifier() {
        let command = GoToBottomCommand::new();
        let context = create_test_context();

        let g_key_ctrl = KeyEvent::new(KeyCode::Char('G'), KeyModifiers::CONTROL);
        assert!(!command.is_relevant(g_key_ctrl, EditorMode::Normal, &context));
    }

    #[test]
    fn should_not_be_relevant_for_g_key_with_alt_modifier() {
        let command = GoToBottomCommand::new();
        let context = create_test_context();

        let g_key_alt = KeyEvent::new(KeyCode::Char('G'), KeyModifiers::ALT);
        assert!(!command.is_relevant(g_key_alt, EditorMode::Normal, &context));
    }

    #[test]
    fn should_not_be_relevant_for_other_keys() {
        let command = GoToBottomCommand::new();
        let context = create_test_context();

        // Test other letter keys
        let h_key = KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE);
        assert!(!command.is_relevant(h_key, EditorMode::Normal, &context));

        let f_key = KeyEvent::new(KeyCode::Char('F'), KeyModifiers::NONE);
        assert!(!command.is_relevant(f_key, EditorMode::Normal, &context));

        // Test special keys
        let down_key = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
        assert!(!command.is_relevant(down_key, EditorMode::Normal, &context));

        let end_key = KeyEvent::new(KeyCode::End, KeyModifiers::NONE);
        assert!(!command.is_relevant(end_key, EditorMode::Normal, &context));
    }

    #[test]
    fn should_work_in_read_only_pane() {
        let command = GoToBottomCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Response,
            is_read_only: true, // Response pane is read-only
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let uppercase_g_key = KeyEvent::new(KeyCode::Char('G'), KeyModifiers::NONE);
        assert!(command.is_relevant(uppercase_g_key, EditorMode::Normal, &context));
    }

    #[test]
    fn execute_should_generate_actions() {
        use crate::repl::models::AppState;
        use crate::repl::services::Services;

        let command = GoToBottomCommand::new();
        let mut app_state = AppState::new();

        // Should work in Normal mode
        app_state.set_mode(EditorMode::Normal);
        assert_eq!(app_state.get_mode(), EditorMode::Normal);

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

        let actions = result.unwrap();

        // Should generate at least one action (StatusBarUpdateRequired)
        assert!(!actions.is_empty());

        // Should stay in Normal mode (no mode change)
        assert_eq!(context.app_state.get_mode(), EditorMode::Normal);

        // Should include status bar update action
        assert!(actions.contains(&PostCommandAction::StatusBarUpdateRequired));
    }

    #[test]
    fn execute_should_not_change_mode() {
        use crate::repl::models::AppState;
        use crate::repl::services::Services;

        let command = GoToBottomCommand::new();

        // Test Normal mode
        let mut app_state = AppState::new();
        app_state.set_mode(EditorMode::Normal);

        let mut services = Services::new();
        let mut context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        let _result = command
            .execute(
                KeyEvent::new(KeyCode::Null, KeyModifiers::empty()),
                &mut context,
            )
            .unwrap();

        // Mode should remain Normal
        assert_eq!(context.app_state.get_mode(), EditorMode::Normal);

        // Test Visual mode
        let mut app_state2 = AppState::new();
        app_state2.set_mode(EditorMode::Visual);

        let mut services2 = Services::new();
        let mut context2 = ExecutionContext {
            app_state: &mut app_state2,
            services: &mut services2,
        };

        let _result2 = command
            .execute(
                KeyEvent::new(KeyCode::Null, KeyModifiers::empty()),
                &mut context2,
            )
            .unwrap();

        // Mode should remain Visual
        assert_eq!(context2.app_state.get_mode(), EditorMode::Visual);
    }

    #[test]
    fn default_should_create_new_instance() {
        let command = GoToBottomCommand;
        assert_eq!(command.name(), "GoToBottomCommand");
    }

    #[test]
    fn execute_should_move_cursor_to_document_end() {
        use crate::repl::models::AppState;
        use crate::repl::services::Services;

        let command = GoToBottomCommand::new();
        let mut app_state = AppState::new();

        // Set up Normal mode
        app_state.set_mode(EditorMode::Normal);

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

        // The exact cursor position and actions depend on pane_manager implementation,
        // but we verify that the command executed successfully
        let actions = result.unwrap();
        assert!(!actions.is_empty());

        // Mode should remain Normal
        assert_eq!(context.app_state.get_mode(), EditorMode::Normal);
    }

    #[test]
    fn should_handle_all_terminal_g_variations() {
        let command = GoToBottomCommand::new();
        let context = create_test_context();

        // Variation 1: Uppercase 'G' without modifiers
        let var1 = KeyEvent::new(KeyCode::Char('G'), KeyModifiers::NONE);
        assert!(command.is_relevant(var1, EditorMode::Normal, &context));

        // Variation 2: Lowercase 'g' with SHIFT modifier
        let var2 = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::SHIFT);
        assert!(command.is_relevant(var2, EditorMode::Normal, &context));

        // Variation 3: Uppercase 'G' with SHIFT modifier
        let var3 = KeyEvent::new(KeyCode::Char('G'), KeyModifiers::SHIFT);
        assert!(command.is_relevant(var3, EditorMode::Normal, &context));
    }

    #[test]
    fn should_not_be_relevant_for_combined_modifiers() {
        let command = GoToBottomCommand::new();
        let context = create_test_context();

        // Should not be relevant with multiple modifiers
        let ctrl_shift_g = KeyEvent::new(
            KeyCode::Char('G'),
            KeyModifiers::CONTROL | KeyModifiers::SHIFT,
        );
        assert!(!command.is_relevant(ctrl_shift_g, EditorMode::Normal, &context));

        let alt_shift_g =
            KeyEvent::new(KeyCode::Char('G'), KeyModifiers::ALT | KeyModifiers::SHIFT);
        assert!(!command.is_relevant(alt_shift_g, EditorMode::Normal, &context));

        let ctrl_alt_g = KeyEvent::new(
            KeyCode::Char('G'),
            KeyModifiers::CONTROL | KeyModifiers::ALT,
        );
        assert!(!command.is_relevant(ctrl_alt_g, EditorMode::Normal, &context));
    }
}

// Auto-register this command using the inventory system
register_command!(GoToBottomCommand, "GoToBottomCommand");
