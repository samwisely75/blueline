//! # Repeat Visual Selection Command
//!
//! Command to repeat (restore) the last visual selection.
//! This command handles the 'gv' key sequence in vim to restore
//! the previous visual selection including its mode and range.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use crate::register_command;
use crate::repl::models::pane_state::EditorMode;
use crate::repl::unified_commands::{Command, CommandContext, ExecutionContext};
use crate::repl::view_models::post_command_actions::PostCommandAction;

/// Command to repeat (restore) the last visual selection
///
/// This command:
/// 1. Only works when 'v' key is pressed in GPrefix mode (completing "gv" sequence)
/// 2. Exits GPrefix mode and returns to Normal mode first
/// 3. Attempts to restore the last visual selection state
/// 4. Changes to the restored visual mode if successful
/// 5. Stays in Normal mode if no previous selection exists
/// 6. Provides status feedback about the operation
pub struct RepeatVisualSelectionCommand;

impl RepeatVisualSelectionCommand {
    /// Create new RepeatVisualSelectionCommand
    pub fn new() -> Self {
        Self
    }
}

impl Command for RepeatVisualSelectionCommand {
    fn is_relevant(
        &self,
        key_event: KeyEvent,
        mode: EditorMode,
        _context: &CommandContext,
    ) -> bool {
        // Only relevant for 'v' key in GPrefix mode without modifiers
        // This completes the "gv" command sequence
        matches!(key_event.code, KeyCode::Char('v'))
            && key_event.modifiers.is_empty()
            && matches!(mode, EditorMode::GPrefix)
    }

    fn execute(&self, context: &mut ExecutionContext) -> Result<Vec<PostCommandAction>> {
        tracing::info!("Handling repeat visual selection (gv command)");

        // First, return to Normal mode to exit GPrefix mode
        context.app_state.change_mode(EditorMode::Normal)?;

        // Try to restore the last visual selection
        match context.app_state.restore_last_visual_selection()? {
            Some(mode) => {
                tracing::info!("Restored last visual selection with mode {mode:?}");

                // Change to the restored visual mode
                context.app_state.change_mode(mode)?;

                // Set status message for successful restoration
                let mode_name = match mode {
                    EditorMode::Visual => "character-wise",
                    EditorMode::VisualLine => "line-wise",
                    EditorMode::VisualBlock => "block-wise",
                    _ => "visual",
                };
                context
                    .app_state
                    .set_status_message(format!("Restored {mode_name} visual selection"));

                // Return view events for UI updates
                Ok(vec![
                    PostCommandAction::CurrentAreaRedrawRequired,
                    PostCommandAction::StatusBarUpdateRequired,
                ])
            }
            None => {
                tracing::info!("No previous visual selection to restore");

                // Set status message indicating no selection to restore
                context
                    .app_state
                    .set_status_message("No previous visual selection to restore".to_string());

                // Stay in Normal mode, only update status bar
                Ok(vec![PostCommandAction::StatusBarUpdateRequired])
            }
        }
    }

    fn name(&self) -> &'static str {
        "RepeatVisualSelectionCommand"
    }
}

impl Default for RepeatVisualSelectionCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::models::pane_state::Pane;
    use crate::repl::models::AppState;
    use crate::repl::services::Services;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn create_execution_context() -> (AppState, Services) {
        let app_state = AppState::new();
        let services = Services::new();
        (app_state, services)
    }

    fn create_key_event(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent::new(code, modifiers)
    }

    #[test]
    fn repeat_visual_selection_should_be_relevant_for_v_key_in_gprefix_mode() {
        let command = RepeatVisualSelectionCommand::new();
        let key_event = create_key_event(KeyCode::Char('v'), KeyModifiers::NONE);
        let context = CommandContext {
            current_mode: EditorMode::GPrefix,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        assert!(command.is_relevant(key_event, EditorMode::GPrefix, &context));
    }

    #[test]
    fn repeat_visual_selection_should_not_be_relevant_for_other_keys() {
        let command = RepeatVisualSelectionCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::GPrefix,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        // Test other keys
        assert!(!command.is_relevant(
            create_key_event(KeyCode::Char('g'), KeyModifiers::NONE),
            EditorMode::GPrefix,
            &context
        ));
        assert!(!command.is_relevant(
            create_key_event(KeyCode::Char('c'), KeyModifiers::NONE),
            EditorMode::GPrefix,
            &context
        ));
        assert!(!command.is_relevant(
            create_key_event(KeyCode::Enter, KeyModifiers::NONE),
            EditorMode::GPrefix,
            &context
        ));
    }

    #[test]
    fn repeat_visual_selection_should_not_be_relevant_in_other_modes() {
        let command = RepeatVisualSelectionCommand::new();
        let key_event = create_key_event(KeyCode::Char('v'), KeyModifiers::NONE);
        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        // Test different modes
        assert!(!command.is_relevant(key_event, EditorMode::Normal, &context));
        assert!(!command.is_relevant(key_event, EditorMode::Insert, &context));
        assert!(!command.is_relevant(key_event, EditorMode::Visual, &context));
        assert!(!command.is_relevant(key_event, EditorMode::VisualLine, &context));
        assert!(!command.is_relevant(key_event, EditorMode::VisualBlock, &context));
        assert!(!command.is_relevant(key_event, EditorMode::VisualBlockInsert, &context));
    }

    #[test]
    fn repeat_visual_selection_should_not_be_relevant_with_modifiers() {
        let command = RepeatVisualSelectionCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::GPrefix,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        // Test with various modifiers
        assert!(!command.is_relevant(
            create_key_event(KeyCode::Char('v'), KeyModifiers::CONTROL),
            EditorMode::GPrefix,
            &context
        ));
        assert!(!command.is_relevant(
            create_key_event(KeyCode::Char('v'), KeyModifiers::SHIFT),
            EditorMode::GPrefix,
            &context
        ));
        assert!(!command.is_relevant(
            create_key_event(KeyCode::Char('v'), KeyModifiers::ALT),
            EditorMode::GPrefix,
            &context
        ));
    }

    #[test]
    fn repeat_visual_selection_execute_should_handle_no_selection_gracefully() {
        let command = RepeatVisualSelectionCommand::new();
        let (mut app_state, mut services) = create_execution_context();

        // Set up GPrefix mode
        app_state.change_mode(EditorMode::GPrefix).unwrap();

        let mut context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        let result = command.execute(&mut context);
        assert!(result.is_ok());

        let events = result.unwrap();
        assert_eq!(events.len(), 1);
        assert!(matches!(
            events[0],
            PostCommandAction::StatusBarUpdateRequired
        ));

        // Verify we returned to Normal mode
        assert_eq!(context.app_state.get_mode(), EditorMode::Normal);

        // Verify status message was set
        assert_eq!(
            context.app_state.get_status_message(),
            Some("No previous visual selection to restore")
        );
    }

    #[test]
    fn repeat_visual_selection_execute_should_exit_gprefix_mode() {
        let command = RepeatVisualSelectionCommand::new();
        let (mut app_state, mut services) = create_execution_context();

        // Start in GPrefix mode
        app_state.change_mode(EditorMode::GPrefix).unwrap();

        let mut context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        let result = command.execute(&mut context);
        assert!(result.is_ok());

        // Should always return to Normal mode first (exiting GPrefix)
        assert_eq!(context.app_state.get_mode(), EditorMode::Normal);
    }

    #[test]
    fn command_name_should_return_correct_name() {
        let command = RepeatVisualSelectionCommand::new();
        assert_eq!(command.name(), "RepeatVisualSelectionCommand");
    }

    #[test]
    fn default_should_create_new_instance() {
        let command = RepeatVisualSelectionCommand::new();
        assert_eq!(command.name(), "RepeatVisualSelectionCommand");
    }
}

// Auto-register this command using the inventory system
register_command!(RepeatVisualSelectionCommand, "RepeatVisualSelectionCommand");
