//! # Enter G Prefix Command
//!
//! Command that handles the first 'g' key press to enter GPrefix mode.
//! This is the first part of multi-key sequences like 'gg' (go to top)
//! and 'gv' (repeat visual selection).

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use crate::register_command;
use crate::repl::models::pane_state::EditorMode;
use crate::repl::unified_commands::{Command, CommandContext, ExecutionContext};
use crate::repl::view_models::post_command_actions::PostCommandAction;

/// Command to enter GPrefix mode on first 'g' key press
///
/// This command handles the first 'g' in Vim g-commands like:
/// - 'gg' (go to top of buffer)
/// - 'gv' (repeat last visual selection)
///
/// The command is relevant when:
/// - Key is 'g' without modifiers
/// - Current mode allows navigation (Normal, Visual modes)
/// - Not already in GPrefix mode
///
/// After execution, the editor enters GPrefix mode, waiting for the second
/// key in the sequence. If an unknown key follows, CancelGPrefixCommand
/// will return the editor to Normal mode.
#[derive(Debug, Default)]
pub struct EnterGPrefixCommand;

impl EnterGPrefixCommand {
    /// Create new EnterGPrefixCommand
    pub fn new() -> Self {
        Self
    }
}

/// Check if current mode allows navigation and g-prefix commands
fn is_navigation_mode(mode: EditorMode) -> bool {
    matches!(
        mode,
        EditorMode::Normal | EditorMode::Visual | EditorMode::VisualLine | EditorMode::VisualBlock
    )
}

impl Command for EnterGPrefixCommand {
    fn is_relevant(
        &self,
        key_event: KeyEvent,
        mode: EditorMode,
        _context: &CommandContext,
    ) -> bool {
        // Must be 'g' key without modifiers
        matches!(key_event.code, KeyCode::Char('g'))
            && key_event.modifiers.is_empty()
            // Must be in navigation mode (excludes Insert, Command, GPrefix, etc.)
            && is_navigation_mode(mode)
            // Must NOT already be in GPrefix mode (that would be handled by GoToTopCommand)
            && mode != EditorMode::GPrefix
    }

    fn execute(
        &self,
        _key_event: KeyEvent,
        context: &mut ExecutionContext,
    ) -> Result<Vec<PostCommandAction>> {
        // Change mode to GPrefix to await the next key in the sequence
        context.app_state.set_mode(EditorMode::GPrefix);

        tracing::debug!("EnterGPrefixCommand executed: entered GPrefix mode, awaiting next key");

        // Return status bar update to reflect mode change
        Ok(vec![PostCommandAction::StatusBarUpdateRequired])
    }

    fn name(&self) -> &'static str {
        "EnterGPrefixCommand"
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
    fn command_should_return_correct_name() {
        let command = EnterGPrefixCommand::new();
        assert_eq!(command.name(), "EnterGPrefixCommand");
    }

    #[test]
    fn should_be_relevant_for_g_key_in_normal_mode() {
        let command = EnterGPrefixCommand::new();
        let context = create_test_context();

        let g_key = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE);
        assert!(command.is_relevant(g_key, EditorMode::Normal, &context));
    }

    #[test]
    fn should_be_relevant_for_g_key_in_visual_mode() {
        let command = EnterGPrefixCommand::new();
        let context = create_test_context();

        let g_key = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE);
        assert!(command.is_relevant(g_key, EditorMode::Visual, &context));
    }

    #[test]
    fn should_be_relevant_for_g_key_in_visual_line_mode() {
        let command = EnterGPrefixCommand::new();
        let context = create_test_context();

        let g_key = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE);
        assert!(command.is_relevant(g_key, EditorMode::VisualLine, &context));
    }

    #[test]
    fn should_be_relevant_for_g_key_in_visual_block_mode() {
        let command = EnterGPrefixCommand::new();
        let context = create_test_context();

        let g_key = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE);
        assert!(command.is_relevant(g_key, EditorMode::VisualBlock, &context));
    }

    #[test]
    fn should_not_be_relevant_for_g_key_in_insert_mode() {
        let command = EnterGPrefixCommand::new();
        let context = create_test_context();

        let g_key = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE);
        assert!(!command.is_relevant(g_key, EditorMode::Insert, &context));
    }

    #[test]
    fn should_not_be_relevant_for_g_key_in_command_mode() {
        let command = EnterGPrefixCommand::new();
        let context = create_test_context();

        let g_key = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE);
        assert!(!command.is_relevant(g_key, EditorMode::Command, &context));
    }

    #[test]
    fn should_not_be_relevant_for_g_key_in_g_prefix_mode() {
        let command = EnterGPrefixCommand::new();
        let context = create_test_context();

        let g_key = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE);
        // Should not be relevant in GPrefix mode (that's handled by GoToTopCommand)
        assert!(!command.is_relevant(g_key, EditorMode::GPrefix, &context));
    }

    #[test]
    fn should_not_be_relevant_for_g_key_in_d_prefix_mode() {
        let command = EnterGPrefixCommand::new();
        let context = create_test_context();

        let g_key = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE);
        assert!(!command.is_relevant(g_key, EditorMode::DPrefix, &context));
    }

    #[test]
    fn should_not_be_relevant_for_g_key_in_y_prefix_mode() {
        let command = EnterGPrefixCommand::new();
        let context = create_test_context();

        let g_key = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE);
        assert!(!command.is_relevant(g_key, EditorMode::YPrefix, &context));
    }

    #[test]
    fn should_not_be_relevant_for_g_key_with_modifiers() {
        let command = EnterGPrefixCommand::new();
        let context = create_test_context();

        // Test with Ctrl modifier
        let g_key_ctrl = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::CONTROL);
        assert!(!command.is_relevant(g_key_ctrl, EditorMode::Normal, &context));

        // Test with Shift modifier
        let g_key_shift = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::SHIFT);
        assert!(!command.is_relevant(g_key_shift, EditorMode::Normal, &context));

        // Test with Alt modifier
        let g_key_alt = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::ALT);
        assert!(!command.is_relevant(g_key_alt, EditorMode::Normal, &context));
    }

    #[test]
    fn should_not_be_relevant_for_other_keys() {
        let command = EnterGPrefixCommand::new();
        let context = create_test_context();

        // Test other letter keys
        let h_key = KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE);
        assert!(!command.is_relevant(h_key, EditorMode::Normal, &context));

        let upper_g_key = KeyEvent::new(KeyCode::Char('G'), KeyModifiers::NONE);
        assert!(!command.is_relevant(upper_g_key, EditorMode::Normal, &context));

        // Test special keys
        let up_key = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
        assert!(!command.is_relevant(up_key, EditorMode::Normal, &context));

        let home_key = KeyEvent::new(KeyCode::Home, KeyModifiers::NONE);
        assert!(!command.is_relevant(home_key, EditorMode::Normal, &context));
    }

    #[test]
    fn should_work_in_read_only_pane() {
        let command = EnterGPrefixCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Response,
            is_read_only: true, // Response pane is read-only
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let g_key = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE);
        // Should be relevant even in read-only panes (navigation is allowed)
        assert!(command.is_relevant(g_key, EditorMode::Normal, &context));
    }

    #[test]
    fn execute_should_change_mode_to_g_prefix() {
        let command = EnterGPrefixCommand::new();
        let mut app_state = AppState::new();
        let mut services = Services::new();

        // Start in Normal mode
        assert_eq!(app_state.get_mode(), EditorMode::Normal);

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
        assert!(actions.contains(&PostCommandAction::StatusBarUpdateRequired));

        // Should have changed mode to GPrefix
        assert_eq!(context.app_state.get_mode(), EditorMode::GPrefix);
    }

    #[test]
    fn execute_should_work_from_visual_mode() {
        let command = EnterGPrefixCommand::new();
        let mut app_state = AppState::new();
        let mut services = Services::new();

        // Start in Visual mode
        app_state.set_mode(EditorMode::Visual);
        assert_eq!(app_state.get_mode(), EditorMode::Visual);

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

        // Should generate actions
        assert!(!actions.is_empty());
        assert!(actions.contains(&PostCommandAction::StatusBarUpdateRequired));

        // Should have changed mode to GPrefix
        assert_eq!(context.app_state.get_mode(), EditorMode::GPrefix);
    }

    #[test]
    fn default_should_create_new_instance() {
        let command = EnterGPrefixCommand;
        assert_eq!(command.name(), "EnterGPrefixCommand");
    }

    #[test]
    fn is_navigation_mode_should_identify_correct_modes() {
        // Navigation modes should return true
        assert!(is_navigation_mode(EditorMode::Normal));
        assert!(is_navigation_mode(EditorMode::Visual));
        assert!(is_navigation_mode(EditorMode::VisualLine));
        assert!(is_navigation_mode(EditorMode::VisualBlock));

        // Non-navigation modes should return false
        assert!(!is_navigation_mode(EditorMode::Insert));
        assert!(!is_navigation_mode(EditorMode::Command));
        assert!(!is_navigation_mode(EditorMode::GPrefix));
        assert!(!is_navigation_mode(EditorMode::DPrefix));
        assert!(!is_navigation_mode(EditorMode::YPrefix));
    }

    #[test]
    fn should_be_relevant_when_has_selection() {
        let command = EnterGPrefixCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::Visual,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: true, // Has active selection
            ex_command_buffer: String::new(),
        };

        let g_key = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE);
        // Should be relevant even when there's a selection (for gv command)
        assert!(command.is_relevant(g_key, EditorMode::Visual, &context));
    }
}

// Auto-register this command using the inventory system
register_command!(EnterGPrefixCommand, "EnterGPrefixCommand");
