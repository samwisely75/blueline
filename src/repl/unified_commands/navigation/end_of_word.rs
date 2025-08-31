//! # End Of Word Command
//!
//! Command for moving cursor to end of current or next word (e key).
//! This demonstrates the unified command architecture where the Command
//! owns its business logic and returns appropriate PostCommandActions.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use crate::register_command;
use crate::repl::models::pane_state::EditorMode;
use crate::repl::unified_commands::{Command, CommandContext, ExecutionContext};
use crate::repl::view_models::post_command_actions::PostCommandAction;

/// Command to move cursor to end of current or next word
///
/// This command demonstrates the unified architecture:
/// 1. Checks for 'e' key in Normal and Visual modes (vim behavior)
/// 2. Calls the pane manager to move cursor to end of word
/// 3. Returns appropriate PostCommandActions to update the display
///
/// Handles vim-style 'e' navigation for word end movement.
pub struct EndOfWordCommand;

impl EndOfWordCommand {
    /// Create new EndOfWordCommand
    pub fn new() -> Self {
        Self
    }

    /// Check if the key event is relevant for end of word movement
    fn is_end_of_word_key(key_event: KeyEvent) -> bool {
        matches!(key_event.code, KeyCode::Char('e')) && key_event.modifiers.is_empty()
    }

    /// Check if current mode allows word navigation
    fn is_navigation_mode(mode: EditorMode) -> bool {
        matches!(
            mode,
            EditorMode::Normal
                | EditorMode::Visual
                | EditorMode::VisualLine
                | EditorMode::VisualBlock
        )
    }
}

impl Command for EndOfWordCommand {
    fn is_relevant(
        &self,
        key_event: KeyEvent,
        mode: EditorMode,
        _context: &CommandContext,
    ) -> bool {
        // Check if it's the end of word key ('e')
        if !Self::is_end_of_word_key(key_event) {
            return false;
        }

        // Only allow in navigation modes (vim behavior)
        Self::is_navigation_mode(mode)
    }

    fn execute(&self, context: &mut ExecutionContext) -> Result<Vec<PostCommandAction>> {
        // Get the cursor movement events from pane manager
        let events = context.app_state.pane_manager.move_cursor_to_end_of_word();

        tracing::debug!(
            "EndOfWordCommand executed, generated {} events",
            events.len()
        );

        Ok(events)
    }

    fn name(&self) -> &'static str {
        "EndOfWordCommand"
    }
}

impl Default for EndOfWordCommand {
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
    fn end_of_word_command_should_return_correct_name() {
        let command = EndOfWordCommand::new();
        assert_eq!(command.name(), "EndOfWordCommand");
    }

    #[test]
    fn end_of_word_command_should_be_relevant_for_e_in_normal_mode() {
        let command = EndOfWordCommand::new();

        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let e_key = KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE);
        assert!(command.is_relevant(e_key, EditorMode::Normal, &context));
    }

    #[test]
    fn end_of_word_command_should_be_relevant_for_e_in_visual_mode() {
        let command = EndOfWordCommand::new();

        let context = CommandContext {
            current_mode: EditorMode::Visual,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: true,
            ex_command_buffer: String::new(),
        };

        let e_key = KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE);
        assert!(command.is_relevant(e_key, EditorMode::Visual, &context));
    }

    #[test]
    fn end_of_word_command_should_be_relevant_for_e_in_visual_line_mode() {
        let command = EndOfWordCommand::new();

        let context = CommandContext {
            current_mode: EditorMode::VisualLine,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: true,
            ex_command_buffer: String::new(),
        };

        let e_key = KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE);
        assert!(command.is_relevant(e_key, EditorMode::VisualLine, &context));
    }

    #[test]
    fn end_of_word_command_should_be_relevant_for_e_in_visual_block_mode() {
        let command = EndOfWordCommand::new();

        let context = CommandContext {
            current_mode: EditorMode::VisualBlock,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: true,
            ex_command_buffer: String::new(),
        };

        let e_key = KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE);
        assert!(command.is_relevant(e_key, EditorMode::VisualBlock, &context));
    }

    #[test]
    fn end_of_word_command_should_not_be_relevant_for_e_in_insert_mode() {
        let command = EndOfWordCommand::new();

        let context = CommandContext {
            current_mode: EditorMode::Insert,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let e_key = KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE);
        assert!(!command.is_relevant(e_key, EditorMode::Insert, &context));
    }

    #[test]
    fn end_of_word_command_should_not_be_relevant_for_e_with_modifiers() {
        let command = EndOfWordCommand::new();

        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let ctrl_e_key = KeyEvent::new(KeyCode::Char('e'), KeyModifiers::CONTROL);
        assert!(!command.is_relevant(ctrl_e_key, EditorMode::Normal, &context));

        let shift_e_key = KeyEvent::new(KeyCode::Char('e'), KeyModifiers::SHIFT);
        assert!(!command.is_relevant(shift_e_key, EditorMode::Normal, &context));
    }

    #[test]
    fn end_of_word_command_should_not_be_relevant_for_other_keys() {
        let command = EndOfWordCommand::new();

        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let w_key = KeyEvent::new(KeyCode::Char('w'), KeyModifiers::NONE);
        assert!(!command.is_relevant(w_key, EditorMode::Normal, &context));

        let b_key = KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE);
        assert!(!command.is_relevant(b_key, EditorMode::Normal, &context));
    }

    #[test]
    fn end_of_word_command_should_detect_navigation_modes() {
        assert!(EndOfWordCommand::is_navigation_mode(EditorMode::Normal));
        assert!(EndOfWordCommand::is_navigation_mode(EditorMode::Visual));
        assert!(EndOfWordCommand::is_navigation_mode(EditorMode::VisualLine));
        assert!(EndOfWordCommand::is_navigation_mode(
            EditorMode::VisualBlock
        ));

        assert!(!EndOfWordCommand::is_navigation_mode(EditorMode::Insert));
        assert!(!EndOfWordCommand::is_navigation_mode(EditorMode::Command));
    }

    #[test]
    fn end_of_word_command_should_detect_end_of_word_key() {
        let e_key = KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE);
        assert!(EndOfWordCommand::is_end_of_word_key(e_key));

        let ctrl_e_key = KeyEvent::new(KeyCode::Char('e'), KeyModifiers::CONTROL);
        assert!(!EndOfWordCommand::is_end_of_word_key(ctrl_e_key));

        let w_key = KeyEvent::new(KeyCode::Char('w'), KeyModifiers::NONE);
        assert!(!EndOfWordCommand::is_end_of_word_key(w_key));
    }

    #[test]
    fn end_of_word_command_should_create_default_instance() {
        let command = EndOfWordCommand;
        assert_eq!(command.name(), "EndOfWordCommand");
    }

    // TODO: Add integration tests once the testing framework is set up for unified commands
    // Integration tests would verify that:
    // - The command actually moves the cursor to the end of the word
    // - The command works correctly with multi-byte characters
    // - The command integrates properly with the pane manager
}

// Register this command for automatic discovery
register_command!(EndOfWordCommand, "EndOfWordCommand");
