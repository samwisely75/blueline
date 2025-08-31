//! # Move Up Command
//!
//! Command for moving cursor up one line (k key or up arrow).
//! This demonstrates the unified command architecture where the Command
//! owns its business logic and emits appropriate ViewEvents.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::register_command;
use crate::repl::models::events::view_events::ViewEvent;
use crate::repl::models::pane_state::EditorMode;
use crate::repl::unified_commands::{Command, CommandContext, ExecutionContext};

/// Command to move cursor up one line
///
/// This command demonstrates the new architecture:
/// 1. Checks for 'k' key in Normal/Visual modes or Up arrow in any mode
/// 2. Calls the pane manager to move cursor up
/// 3. Emits appropriate ViewEvents to update the display
///
/// Handles both vim-style 'k' navigation and standard up arrow key.
pub struct MoveUpCommand;

impl MoveUpCommand {
    /// Create new MoveUpCommand
    pub fn new() -> Self {
        Self
    }

    /// Check if the key event is relevant for moving up
    fn is_move_up_key(key_event: KeyEvent) -> bool {
        match key_event.code {
            // vim-style 'k' key without modifiers
            KeyCode::Char('k') => key_event.modifiers.is_empty(),
            // Standard up arrow key - allow without Shift/Control (like move_left/right)
            KeyCode::Up => {
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

impl Command for MoveUpCommand {
    fn is_relevant(
        &self,
        key_event: KeyEvent,
        mode: EditorMode,
        _context: &CommandContext,
    ) -> bool {
        // Check if it's a move up key
        if !Self::is_move_up_key(key_event) {
            return false;
        }

        // For 'k' key, only allow in movement modes (Vim behavior)
        // For Up arrow, allow in any mode (standard editor behavior)
        match key_event.code {
            KeyCode::Char('k') => Self::is_movement_mode(mode),
            KeyCode::Up => true,
            _ => false,
        }
    }

    fn execute(&self, context: &mut ExecutionContext) -> Result<Vec<ViewEvent>> {
        // Get the cursor movement events from pane manager
        let events = context.app_state.pane_manager.move_cursor_up();

        tracing::debug!("MoveUpCommand executed, generated {} events", events.len());

        Ok(events)
    }

    fn name(&self) -> &'static str {
        "MoveUpCommand"
    }
}

impl Default for MoveUpCommand {
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
    fn move_up_command_should_return_correct_name() {
        let command = MoveUpCommand::new();
        assert_eq!(command.name(), "MoveUpCommand");
    }

    #[test]
    fn move_up_command_should_be_relevant_for_k_in_normal_mode() {
        let command = MoveUpCommand::new();

        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let k_key = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
        assert!(command.is_relevant(k_key, EditorMode::Normal, &context));
    }

    #[test]
    fn move_up_command_should_be_relevant_for_k_in_visual_mode() {
        let command = MoveUpCommand::new();

        let context = CommandContext {
            current_mode: EditorMode::Visual,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: true,
            ex_command_buffer: String::new(),
        };

        let k_key = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
        assert!(command.is_relevant(k_key, EditorMode::Visual, &context));
    }

    #[test]
    fn move_up_command_should_be_relevant_for_k_in_visual_line_mode() {
        let command = MoveUpCommand::new();

        let context = CommandContext {
            current_mode: EditorMode::VisualLine,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: true,
            ex_command_buffer: String::new(),
        };

        let k_key = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
        assert!(command.is_relevant(k_key, EditorMode::VisualLine, &context));
    }

    #[test]
    fn move_up_command_should_be_relevant_for_k_in_visual_block_mode() {
        let command = MoveUpCommand::new();

        let context = CommandContext {
            current_mode: EditorMode::VisualBlock,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: true,
            ex_command_buffer: String::new(),
        };

        let k_key = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
        assert!(command.is_relevant(k_key, EditorMode::VisualBlock, &context));
    }

    #[test]
    fn move_up_command_should_be_relevant_for_up_arrow_in_any_mode() {
        let command = MoveUpCommand::new();

        let context = CommandContext {
            current_mode: EditorMode::Insert,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let up_key = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
        assert!(command.is_relevant(up_key, EditorMode::Insert, &context));
        assert!(command.is_relevant(up_key, EditorMode::Normal, &context));
        assert!(command.is_relevant(up_key, EditorMode::Visual, &context));
        assert!(command.is_relevant(up_key, EditorMode::Command, &context));
    }

    #[test]
    fn move_up_command_should_allow_up_arrow_with_alt_modifier() {
        let command = MoveUpCommand::new();

        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let up_key = KeyEvent::new(KeyCode::Up, KeyModifiers::ALT);
        assert!(command.is_relevant(up_key, EditorMode::Normal, &context));
    }

    #[test]
    fn move_up_command_should_not_be_relevant_for_up_arrow_with_shift_or_control() {
        let command = MoveUpCommand::new();

        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        // Test Shift+Up (used for selection/scrolling)
        let shift_up = KeyEvent::new(KeyCode::Up, KeyModifiers::SHIFT);
        assert!(!command.is_relevant(shift_up, EditorMode::Normal, &context));

        // Test Ctrl+Up (used for scrolling/other functions)
        let ctrl_up = KeyEvent::new(KeyCode::Up, KeyModifiers::CONTROL);
        assert!(!command.is_relevant(ctrl_up, EditorMode::Normal, &context));
    }

    #[test]
    fn move_up_command_should_not_be_relevant_for_k_in_insert_mode() {
        let command = MoveUpCommand::new();

        let context = CommandContext {
            current_mode: EditorMode::Insert,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let k_key = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
        assert!(!command.is_relevant(k_key, EditorMode::Insert, &context));
    }

    #[test]
    fn move_up_command_should_not_be_relevant_for_k_in_command_mode() {
        let command = MoveUpCommand::new();

        let context = CommandContext {
            current_mode: EditorMode::Command,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let k_key = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
        assert!(!command.is_relevant(k_key, EditorMode::Command, &context));
    }

    #[test]
    fn move_up_command_should_work_in_read_only_panes() {
        let command = MoveUpCommand::new();

        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Response,
            is_read_only: true,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let k_key = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
        assert!(command.is_relevant(k_key, EditorMode::Normal, &context));

        let up_key = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
        assert!(command.is_relevant(up_key, EditorMode::Normal, &context));
    }

    #[test]
    fn move_up_command_should_not_be_relevant_for_k_with_modifiers() {
        let command = MoveUpCommand::new();

        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let k_key_ctrl = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::CONTROL);
        assert!(!command.is_relevant(k_key_ctrl, EditorMode::Normal, &context));

        let k_key_shift = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::SHIFT);
        assert!(!command.is_relevant(k_key_shift, EditorMode::Normal, &context));
    }

    #[test]
    fn move_up_command_should_not_be_relevant_for_wrong_key() {
        let command = MoveUpCommand::new();

        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let j_key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        assert!(!command.is_relevant(j_key, EditorMode::Normal, &context));

        let h_key = KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE);
        assert!(!command.is_relevant(h_key, EditorMode::Normal, &context));
    }

    #[test]
    fn is_move_up_key_should_detect_k_and_up_arrow() {
        let k_key = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
        assert!(MoveUpCommand::is_move_up_key(k_key));

        let up_key = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
        assert!(MoveUpCommand::is_move_up_key(up_key));
    }

    #[test]
    fn is_move_up_key_should_reject_k_with_modifiers() {
        let k_key_ctrl = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::CONTROL);
        assert!(!MoveUpCommand::is_move_up_key(k_key_ctrl));

        let k_key_shift = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::SHIFT);
        assert!(!MoveUpCommand::is_move_up_key(k_key_shift));
    }

    #[test]
    fn is_move_up_key_should_reject_other_keys() {
        let j_key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        assert!(!MoveUpCommand::is_move_up_key(j_key));

        let h_key = KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE);
        assert!(!MoveUpCommand::is_move_up_key(h_key));

        let down_key = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
        assert!(!MoveUpCommand::is_move_up_key(down_key));
    }

    #[test]
    fn move_up_command_should_generate_events_on_execution() {
        use crate::repl::models::AppState;
        use crate::repl::services::Services;

        let command = MoveUpCommand::new();
        let mut app_state = AppState::new();
        let mut services = Services::new();
        let mut context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        // Should succeed and return some events (could be empty if no movement possible)
        let result = command.execute(&mut context);
        assert!(result.is_ok());

        // We don't check exact events since they depend on internal state
        // but we verify the command can execute without errors
        let _events = result.unwrap();
    }

    #[test]
    fn default_should_create_new_instance() {
        let command = MoveUpCommand;
        assert_eq!(command.name(), "MoveUpCommand");
    }

    #[test]
    fn is_movement_mode_should_identify_correct_modes() {
        // Movement modes
        assert!(MoveUpCommand::is_movement_mode(EditorMode::Normal));
        assert!(MoveUpCommand::is_movement_mode(EditorMode::Visual));
        assert!(MoveUpCommand::is_movement_mode(EditorMode::VisualLine));
        assert!(MoveUpCommand::is_movement_mode(EditorMode::VisualBlock));

        // Non-movement modes
        assert!(!MoveUpCommand::is_movement_mode(EditorMode::Insert));
        assert!(!MoveUpCommand::is_movement_mode(
            EditorMode::VisualBlockInsert
        ));
        assert!(!MoveUpCommand::is_movement_mode(EditorMode::Command));
    }
}

// Auto-register this command using the inventory system
register_command!(MoveUpCommand, "MoveUpCommand");
