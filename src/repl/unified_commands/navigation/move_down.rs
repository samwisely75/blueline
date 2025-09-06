//! # Move Down Command
//!
//! Command for moving cursor down one line (j key or down arrow).
//! This demonstrates the unified command architecture where the Command
//! owns its business logic and emits appropriate PostCommandActions.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::register_command;
use crate::repl::models::pane_state::EditorMode;
use crate::repl::unified_commands::{Command, CommandContext, ExecutionContext};
use crate::repl::view_models::post_command_actions::PostCommandAction;

/// Command to move cursor down one line
///
/// This command demonstrates the new architecture:
/// 1. Checks for 'j' key in Normal/Visual modes or Down arrow in any mode
/// 2. Calls the pane manager to move cursor down
/// 3. Emits appropriate PostCommandActions to update the display
///
/// Handles both vim-style 'j' navigation and standard down arrow key.
pub struct MoveDownCommand;

impl MoveDownCommand {
    /// Create new MoveDownCommand
    pub fn new() -> Self {
        Self
    }

    /// Check if the key event is relevant for moving down
    fn is_move_down_key(key_event: KeyEvent) -> bool {
        match key_event.code {
            // vim-style 'j' key without modifiers
            KeyCode::Char('j') => key_event.modifiers.is_empty(),
            // Standard down arrow key - allow without Shift/Control (like move_left/right)
            KeyCode::Down => {
                !key_event.modifiers.contains(KeyModifiers::SHIFT)
                    && !key_event.modifiers.contains(KeyModifiers::CONTROL)
            }
            _ => false,
        }
    }

    /// Check if current mode allows cursor movement
    fn is_movement_mode(mode: EditorMode) -> bool {
        matches!(
            mode,
            EditorMode::Normal
                | EditorMode::Visual
                | EditorMode::VisualLine
                | EditorMode::VisualBlock
        )
    }
}

impl Command for MoveDownCommand {
    fn is_relevant(
        &self,
        key_event: KeyEvent,
        mode: EditorMode,
        _context: &CommandContext,
    ) -> bool {
        // Check if it's a move down key
        if !Self::is_move_down_key(key_event) {
            return false;
        }

        // For 'j' key, only allow in movement modes (Vim behavior)
        // For Down arrow, allow in any mode (standard editor behavior)
        match key_event.code {
            KeyCode::Char('j') => Self::is_movement_mode(mode),
            KeyCode::Down => true,
            _ => false,
        }
    }

    fn execute(
        &self,
        _key_event: KeyEvent,
        context: &mut ExecutionContext,
    ) -> Result<Vec<PostCommandAction>> {
        // Get the cursor movement events from pane manager
        let events = context.app_state.pane_manager.move_cursor_down();

        tracing::debug!(
            "MoveDownCommand executed, generated {} events",
            events.len()
        );

        Ok(events)
    }

    fn name(&self) -> &'static str {
        "MoveDownCommand"
    }
}

impl Default for MoveDownCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::models::pane_state::Pane;
    use crossterm::event::KeyModifiers;

    #[test]
    fn move_down_command_should_return_correct_name() {
        let command = MoveDownCommand::new();
        assert_eq!(command.name(), "MoveDownCommand");
    }

    #[test]
    fn move_down_command_should_be_relevant_for_j_in_normal_mode() {
        let command = MoveDownCommand::new();

        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let j_key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        assert!(command.is_relevant(j_key, EditorMode::Normal, &context));
    }

    #[test]
    fn move_down_command_should_be_relevant_for_j_in_visual_mode() {
        let command = MoveDownCommand::new();

        let context = CommandContext {
            current_mode: EditorMode::Visual,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: true,
            ex_command_buffer: String::new(),
        };

        let j_key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        assert!(command.is_relevant(j_key, EditorMode::Visual, &context));
    }

    #[test]
    fn move_down_command_should_be_relevant_for_j_in_visual_line_mode() {
        let command = MoveDownCommand::new();

        let context = CommandContext {
            current_mode: EditorMode::VisualLine,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: true,
            ex_command_buffer: String::new(),
        };

        let j_key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        assert!(command.is_relevant(j_key, EditorMode::VisualLine, &context));
    }

    #[test]
    fn move_down_command_should_be_relevant_for_j_in_visual_block_mode() {
        let command = MoveDownCommand::new();

        let context = CommandContext {
            current_mode: EditorMode::VisualBlock,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: true,
            ex_command_buffer: String::new(),
        };

        let j_key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        assert!(command.is_relevant(j_key, EditorMode::VisualBlock, &context));
    }

    #[test]
    fn move_down_command_should_be_relevant_for_down_arrow_in_any_mode() {
        let command = MoveDownCommand::new();

        let context = CommandContext {
            current_mode: EditorMode::Insert,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let down_key = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
        assert!(command.is_relevant(down_key, EditorMode::Insert, &context));
        assert!(command.is_relevant(down_key, EditorMode::Normal, &context));
        assert!(command.is_relevant(down_key, EditorMode::Visual, &context));
        assert!(command.is_relevant(down_key, EditorMode::Command, &context));
    }

    #[test]
    fn move_down_command_should_allow_down_arrow_with_alt_modifier() {
        let command = MoveDownCommand::new();

        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let down_key = KeyEvent::new(KeyCode::Down, KeyModifiers::ALT);
        assert!(command.is_relevant(down_key, EditorMode::Normal, &context));
    }

    #[test]
    fn move_down_command_should_not_be_relevant_for_down_arrow_with_shift_or_control() {
        let command = MoveDownCommand::new();

        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        // Test Shift+Down (used for selection/scrolling)
        let shift_down = KeyEvent::new(KeyCode::Down, KeyModifiers::SHIFT);
        assert!(!command.is_relevant(shift_down, EditorMode::Normal, &context));

        // Test Ctrl+Down (used for scrolling/other functions)
        let ctrl_down = KeyEvent::new(KeyCode::Down, KeyModifiers::CONTROL);
        assert!(!command.is_relevant(ctrl_down, EditorMode::Normal, &context));
    }

    #[test]
    fn move_down_command_should_not_be_relevant_for_j_in_insert_mode() {
        let command = MoveDownCommand::new();

        let context = CommandContext {
            current_mode: EditorMode::Insert,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let j_key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        assert!(!command.is_relevant(j_key, EditorMode::Insert, &context));
    }

    #[test]
    fn move_down_command_should_not_be_relevant_for_j_in_command_mode() {
        let command = MoveDownCommand::new();

        let context = CommandContext {
            current_mode: EditorMode::Command,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let j_key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        assert!(!command.is_relevant(j_key, EditorMode::Command, &context));
    }

    #[test]
    fn move_down_command_should_work_in_read_only_panes() {
        let command = MoveDownCommand::new();

        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Response,
            is_read_only: true,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let j_key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        assert!(command.is_relevant(j_key, EditorMode::Normal, &context));

        let down_key = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
        assert!(command.is_relevant(down_key, EditorMode::Normal, &context));
    }

    #[test]
    fn move_down_command_should_not_be_relevant_for_j_with_modifiers() {
        let command = MoveDownCommand::new();

        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let j_key_ctrl = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::CONTROL);
        assert!(!command.is_relevant(j_key_ctrl, EditorMode::Normal, &context));

        let j_key_shift = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::SHIFT);
        assert!(!command.is_relevant(j_key_shift, EditorMode::Normal, &context));
    }

    #[test]
    fn move_down_command_should_not_be_relevant_for_wrong_key() {
        let command = MoveDownCommand::new();

        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let k_key = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
        assert!(!command.is_relevant(k_key, EditorMode::Normal, &context));

        let h_key = KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE);
        assert!(!command.is_relevant(h_key, EditorMode::Normal, &context));
    }

    #[test]
    fn is_move_down_key_should_detect_j_and_down_arrow() {
        let j_key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        assert!(MoveDownCommand::is_move_down_key(j_key));

        let down_key = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
        assert!(MoveDownCommand::is_move_down_key(down_key));
    }

    #[test]
    fn is_move_down_key_should_reject_j_with_modifiers() {
        let j_key_ctrl = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::CONTROL);
        assert!(!MoveDownCommand::is_move_down_key(j_key_ctrl));

        let j_key_shift = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::SHIFT);
        assert!(!MoveDownCommand::is_move_down_key(j_key_shift));
    }

    #[test]
    fn is_move_down_key_should_reject_other_keys() {
        let k_key = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
        assert!(!MoveDownCommand::is_move_down_key(k_key));

        let h_key = KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE);
        assert!(!MoveDownCommand::is_move_down_key(h_key));

        let up_key = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
        assert!(!MoveDownCommand::is_move_down_key(up_key));
    }

    #[test]
    fn move_down_command_should_generate_events_on_execution() {
        use crate::repl::models::AppState;
        use crate::repl::services::Services;

        let command = MoveDownCommand::new();
        let mut app_state = AppState::new();
        let mut services = Services::new();
        let mut context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        // Should succeed and return some events (could be empty if no movement possible)
        let result = command.execute(
            KeyEvent::new(KeyCode::Null, KeyModifiers::empty()),
            &mut context,
        );
        assert!(result.is_ok());

        // We don't check exact events since they depend on internal state
        // but we verify the command can execute without errors
        let _events = result.unwrap();
    }

    #[test]
    fn default_should_create_new_instance() {
        let command = MoveDownCommand;
        assert_eq!(command.name(), "MoveDownCommand");
    }

    #[test]
    fn is_movement_mode_should_identify_correct_modes() {
        // Movement modes
        assert!(MoveDownCommand::is_movement_mode(EditorMode::Normal));
        assert!(MoveDownCommand::is_movement_mode(EditorMode::Visual));
        assert!(MoveDownCommand::is_movement_mode(EditorMode::VisualLine));
        assert!(MoveDownCommand::is_movement_mode(EditorMode::VisualBlock));

        // Non-movement modes
        assert!(!MoveDownCommand::is_movement_mode(EditorMode::Insert));
        assert!(!MoveDownCommand::is_movement_mode(
            EditorMode::VisualBlockInsert
        ));
        assert!(!MoveDownCommand::is_movement_mode(EditorMode::Command));
    }
}

// Auto-register this command using the inventory system
register_command!(MoveDownCommand, "MoveDownCommand");
