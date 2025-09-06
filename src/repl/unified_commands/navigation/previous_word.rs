//! # Previous Word Command
//!
//! Command for moving cursor to the previous word boundary (b key).
//! This demonstrates the unified command architecture where the Command
//! owns its business logic and emits appropriate PostCommandActions.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use crate::register_command;
use crate::repl::models::pane_state::EditorMode;
use crate::repl::unified_commands::{Command, CommandContext, ExecutionContext};
use crate::repl::view_models::post_command_actions::PostCommandAction;

/// Command to move cursor to the previous word boundary
///
/// This command demonstrates the new architecture:
/// 1. Checks for 'b' key in Normal/Visual modes
/// 2. Calls the pane manager to move cursor to previous word
/// 3. Emits appropriate PostCommandActions to update the display
///
/// Handles Vim-style 'b' navigation for word backward movement.
pub struct PreviousWordCommand;

impl PreviousWordCommand {
    /// Create new PreviousWordCommand
    pub fn new() -> Self {
        Self
    }

    /// Check if the key event is relevant for moving to previous word
    fn is_previous_word_key(key_event: KeyEvent) -> bool {
        match key_event.code {
            // vim-style 'b' key without modifiers
            KeyCode::Char('b') => key_event.modifiers.is_empty(),
            _ => false,
        }
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

impl Command for PreviousWordCommand {
    fn is_relevant(
        &self,
        key_event: KeyEvent,
        mode: EditorMode,
        _context: &CommandContext,
    ) -> bool {
        // Check if it's a previous word key and we're in a navigation mode
        Self::is_previous_word_key(key_event) && Self::is_navigation_mode(mode)
    }

    fn execute(
        &self,
        _key_event: KeyEvent,
        context: &mut ExecutionContext,
    ) -> Result<Vec<PostCommandAction>> {
        // Get the cursor movement events from pane manager
        let events = context
            .app_state
            .pane_manager
            .move_cursor_to_previous_word();

        tracing::debug!(
            "PreviousWordCommand executed, generated {} events",
            events.len()
        );

        Ok(events)
    }

    fn name(&self) -> &'static str {
        "PreviousWordCommand"
    }
}

impl Default for PreviousWordCommand {
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
    fn previous_word_command_should_return_correct_name() {
        let command = PreviousWordCommand::new();
        assert_eq!(command.name(), "PreviousWordCommand");
    }

    #[test]
    fn previous_word_command_should_be_relevant_for_b_in_normal_mode() {
        let command = PreviousWordCommand::new();

        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let b_key = KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE);
        assert!(command.is_relevant(b_key, EditorMode::Normal, &context));
    }

    #[test]
    fn previous_word_command_should_be_relevant_for_b_in_visual_mode() {
        let command = PreviousWordCommand::new();

        let context = CommandContext {
            current_mode: EditorMode::Visual,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: true,
            ex_command_buffer: String::new(),
        };

        let b_key = KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE);
        assert!(command.is_relevant(b_key, EditorMode::Visual, &context));
    }

    #[test]
    fn previous_word_command_should_be_relevant_for_b_in_visual_line_mode() {
        let command = PreviousWordCommand::new();

        let context = CommandContext {
            current_mode: EditorMode::VisualLine,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: true,
            ex_command_buffer: String::new(),
        };

        let b_key = KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE);
        assert!(command.is_relevant(b_key, EditorMode::VisualLine, &context));
    }

    #[test]
    fn previous_word_command_should_be_relevant_for_b_in_visual_block_mode() {
        let command = PreviousWordCommand::new();

        let context = CommandContext {
            current_mode: EditorMode::VisualBlock,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: true,
            ex_command_buffer: String::new(),
        };

        let b_key = KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE);
        assert!(command.is_relevant(b_key, EditorMode::VisualBlock, &context));
    }

    #[test]
    fn previous_word_command_should_not_be_relevant_for_b_in_insert_mode() {
        let command = PreviousWordCommand::new();

        let context = CommandContext {
            current_mode: EditorMode::Insert,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let b_key = KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE);
        assert!(!command.is_relevant(b_key, EditorMode::Insert, &context));
    }

    #[test]
    fn previous_word_command_should_not_be_relevant_for_b_in_command_mode() {
        let command = PreviousWordCommand::new();

        let context = CommandContext {
            current_mode: EditorMode::Command,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let b_key = KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE);
        assert!(!command.is_relevant(b_key, EditorMode::Command, &context));
    }

    #[test]
    fn previous_word_command_should_not_be_relevant_for_b_in_visual_block_insert_mode() {
        let command = PreviousWordCommand::new();

        let context = CommandContext {
            current_mode: EditorMode::VisualBlockInsert,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let b_key = KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE);
        assert!(!command.is_relevant(b_key, EditorMode::VisualBlockInsert, &context));
    }

    #[test]
    fn previous_word_command_should_work_in_read_only_panes() {
        let command = PreviousWordCommand::new();

        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Response,
            is_read_only: true,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let b_key = KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE);
        assert!(command.is_relevant(b_key, EditorMode::Normal, &context));
    }

    #[test]
    fn previous_word_command_should_not_be_relevant_for_b_with_modifiers() {
        let command = PreviousWordCommand::new();

        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let b_key_ctrl = KeyEvent::new(KeyCode::Char('b'), KeyModifiers::CONTROL);
        assert!(!command.is_relevant(b_key_ctrl, EditorMode::Normal, &context));

        let b_key_shift = KeyEvent::new(KeyCode::Char('b'), KeyModifiers::SHIFT);
        assert!(!command.is_relevant(b_key_shift, EditorMode::Normal, &context));

        let b_key_alt = KeyEvent::new(KeyCode::Char('b'), KeyModifiers::ALT);
        assert!(!command.is_relevant(b_key_alt, EditorMode::Normal, &context));
    }

    #[test]
    fn previous_word_command_should_not_be_relevant_for_wrong_key() {
        let command = PreviousWordCommand::new();

        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let w_key = KeyEvent::new(KeyCode::Char('w'), KeyModifiers::NONE);
        assert!(!command.is_relevant(w_key, EditorMode::Normal, &context));

        let e_key = KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE);
        assert!(!command.is_relevant(e_key, EditorMode::Normal, &context));

        let j_key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        assert!(!command.is_relevant(j_key, EditorMode::Normal, &context));
    }

    #[test]
    fn is_previous_word_key_should_detect_b_without_modifiers() {
        let b_key = KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE);
        assert!(PreviousWordCommand::is_previous_word_key(b_key));
    }

    #[test]
    fn is_previous_word_key_should_reject_b_with_modifiers() {
        let b_key_ctrl = KeyEvent::new(KeyCode::Char('b'), KeyModifiers::CONTROL);
        assert!(!PreviousWordCommand::is_previous_word_key(b_key_ctrl));

        let b_key_shift = KeyEvent::new(KeyCode::Char('b'), KeyModifiers::SHIFT);
        assert!(!PreviousWordCommand::is_previous_word_key(b_key_shift));

        let b_key_alt = KeyEvent::new(KeyCode::Char('b'), KeyModifiers::ALT);
        assert!(!PreviousWordCommand::is_previous_word_key(b_key_alt));
    }

    #[test]
    fn is_previous_word_key_should_reject_other_keys() {
        let w_key = KeyEvent::new(KeyCode::Char('w'), KeyModifiers::NONE);
        assert!(!PreviousWordCommand::is_previous_word_key(w_key));

        let e_key = KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE);
        assert!(!PreviousWordCommand::is_previous_word_key(e_key));

        let left_key = KeyEvent::new(KeyCode::Left, KeyModifiers::NONE);
        assert!(!PreviousWordCommand::is_previous_word_key(left_key));
    }

    #[test]
    fn previous_word_command_should_generate_events_on_execution() {
        use crate::repl::models::AppState;
        use crate::repl::services::Services;

        let command = PreviousWordCommand::new();
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
        let command = PreviousWordCommand;
        assert_eq!(command.name(), "PreviousWordCommand");
    }

    #[test]
    fn is_navigation_mode_should_identify_correct_modes() {
        // Navigation modes
        assert!(PreviousWordCommand::is_navigation_mode(EditorMode::Normal));
        assert!(PreviousWordCommand::is_navigation_mode(EditorMode::Visual));
        assert!(PreviousWordCommand::is_navigation_mode(
            EditorMode::VisualLine
        ));
        assert!(PreviousWordCommand::is_navigation_mode(
            EditorMode::VisualBlock
        ));

        // Non-navigation modes
        assert!(!PreviousWordCommand::is_navigation_mode(EditorMode::Insert));
        assert!(!PreviousWordCommand::is_navigation_mode(
            EditorMode::VisualBlockInsert
        ));
        assert!(!PreviousWordCommand::is_navigation_mode(
            EditorMode::Command
        ));
        assert!(!PreviousWordCommand::is_navigation_mode(
            EditorMode::GPrefix
        ));
    }
}

// Auto-register this command using the inventory system
register_command!(PreviousWordCommand, "PreviousWordCommand");
