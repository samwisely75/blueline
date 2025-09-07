//! # Next Word Command
//!
//! Command for moving cursor to the next word boundary (w key).
//! This demonstrates the unified command architecture where the Command
//! owns its business logic and emits appropriate PostCommandActions.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use crate::register_command;
use crate::repl::models::pane_state::EditorMode;
use crate::repl::view_models::commands::{Command, CommandContext, ExecutionContext};
use crate::repl::view_models::post_command_actions::PostCommandAction;

/// Command to move cursor to the next word boundary
///
/// This command demonstrates the new architecture:
/// 1. Checks for 'w' key in Normal/Visual modes
/// 2. Calls the pane manager to move cursor to next word
/// 3. Emits appropriate PostCommandActions to update the display
///
/// Handles Vim-style 'w' navigation for word forward movement.
pub struct NextWordCommand;

impl NextWordCommand {
    /// Create new NextWordCommand
    pub fn new() -> Self {
        Self
    }

    /// Check if the key event is relevant for moving to next word
    fn is_next_word_key(key_event: KeyEvent) -> bool {
        match key_event.code {
            // vim-style 'w' key without modifiers
            KeyCode::Char('w') => key_event.modifiers.is_empty(),
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

impl Command for NextWordCommand {
    fn is_relevant(
        &self,
        key_event: KeyEvent,
        mode: EditorMode,
        _context: &CommandContext,
    ) -> bool {
        // Check if it's a next word key and we're in a navigation mode
        Self::is_next_word_key(key_event) && Self::is_navigation_mode(mode)
    }

    fn execute(
        &self,
        _key_event: KeyEvent,
        context: &mut ExecutionContext,
    ) -> Result<Vec<PostCommandAction>> {
        // Get the cursor movement events from pane manager
        let events = context.app_state.pane_manager.move_cursor_to_next_word();

        tracing::debug!(
            "NextWordCommand executed, generated {} events",
            events.len()
        );

        Ok(events)
    }

    fn name(&self) -> &'static str {
        "NextWordCommand"
    }
}

impl Default for NextWordCommand {
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
    fn next_word_command_should_return_correct_name() {
        let command = NextWordCommand::new();
        assert_eq!(command.name(), "NextWordCommand");
    }

    #[test]
    fn next_word_command_should_be_relevant_for_w_in_normal_mode() {
        let command = NextWordCommand::new();

        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let w_key = KeyEvent::new(KeyCode::Char('w'), KeyModifiers::NONE);
        assert!(command.is_relevant(w_key, EditorMode::Normal, &context));
    }

    #[test]
    fn next_word_command_should_be_relevant_for_w_in_visual_mode() {
        let command = NextWordCommand::new();

        let context = CommandContext {
            current_mode: EditorMode::Visual,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: true,
            ex_command_buffer: String::new(),
        };

        let w_key = KeyEvent::new(KeyCode::Char('w'), KeyModifiers::NONE);
        assert!(command.is_relevant(w_key, EditorMode::Visual, &context));
    }

    #[test]
    fn next_word_command_should_be_relevant_for_w_in_visual_line_mode() {
        let command = NextWordCommand::new();

        let context = CommandContext {
            current_mode: EditorMode::VisualLine,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: true,
            ex_command_buffer: String::new(),
        };

        let w_key = KeyEvent::new(KeyCode::Char('w'), KeyModifiers::NONE);
        assert!(command.is_relevant(w_key, EditorMode::VisualLine, &context));
    }

    #[test]
    fn next_word_command_should_be_relevant_for_w_in_visual_block_mode() {
        let command = NextWordCommand::new();

        let context = CommandContext {
            current_mode: EditorMode::VisualBlock,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: true,
            ex_command_buffer: String::new(),
        };

        let w_key = KeyEvent::new(KeyCode::Char('w'), KeyModifiers::NONE);
        assert!(command.is_relevant(w_key, EditorMode::VisualBlock, &context));
    }

    #[test]
    fn next_word_command_should_not_be_relevant_for_w_in_insert_mode() {
        let command = NextWordCommand::new();

        let context = CommandContext {
            current_mode: EditorMode::Insert,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let w_key = KeyEvent::new(KeyCode::Char('w'), KeyModifiers::NONE);
        assert!(!command.is_relevant(w_key, EditorMode::Insert, &context));
    }

    #[test]
    fn next_word_command_should_not_be_relevant_for_w_in_command_mode() {
        let command = NextWordCommand::new();

        let context = CommandContext {
            current_mode: EditorMode::Command,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let w_key = KeyEvent::new(KeyCode::Char('w'), KeyModifiers::NONE);
        assert!(!command.is_relevant(w_key, EditorMode::Command, &context));
    }

    #[test]
    fn next_word_command_should_not_be_relevant_for_w_in_visual_block_insert_mode() {
        let command = NextWordCommand::new();

        let context = CommandContext {
            current_mode: EditorMode::VisualBlockInsert,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let w_key = KeyEvent::new(KeyCode::Char('w'), KeyModifiers::NONE);
        assert!(!command.is_relevant(w_key, EditorMode::VisualBlockInsert, &context));
    }

    #[test]
    fn next_word_command_should_work_in_read_only_panes() {
        let command = NextWordCommand::new();

        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Response,
            is_read_only: true,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let w_key = KeyEvent::new(KeyCode::Char('w'), KeyModifiers::NONE);
        assert!(command.is_relevant(w_key, EditorMode::Normal, &context));
    }

    #[test]
    fn next_word_command_should_not_be_relevant_for_w_with_modifiers() {
        let command = NextWordCommand::new();

        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let w_key_ctrl = KeyEvent::new(KeyCode::Char('w'), KeyModifiers::CONTROL);
        assert!(!command.is_relevant(w_key_ctrl, EditorMode::Normal, &context));

        let w_key_shift = KeyEvent::new(KeyCode::Char('w'), KeyModifiers::SHIFT);
        assert!(!command.is_relevant(w_key_shift, EditorMode::Normal, &context));

        let w_key_alt = KeyEvent::new(KeyCode::Char('w'), KeyModifiers::ALT);
        assert!(!command.is_relevant(w_key_alt, EditorMode::Normal, &context));
    }

    #[test]
    fn next_word_command_should_not_be_relevant_for_wrong_key() {
        let command = NextWordCommand::new();

        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let b_key = KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE);
        assert!(!command.is_relevant(b_key, EditorMode::Normal, &context));

        let e_key = KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE);
        assert!(!command.is_relevant(e_key, EditorMode::Normal, &context));

        let j_key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        assert!(!command.is_relevant(j_key, EditorMode::Normal, &context));
    }

    #[test]
    fn is_next_word_key_should_detect_w_without_modifiers() {
        let w_key = KeyEvent::new(KeyCode::Char('w'), KeyModifiers::NONE);
        assert!(NextWordCommand::is_next_word_key(w_key));
    }

    #[test]
    fn is_next_word_key_should_reject_w_with_modifiers() {
        let w_key_ctrl = KeyEvent::new(KeyCode::Char('w'), KeyModifiers::CONTROL);
        assert!(!NextWordCommand::is_next_word_key(w_key_ctrl));

        let w_key_shift = KeyEvent::new(KeyCode::Char('w'), KeyModifiers::SHIFT);
        assert!(!NextWordCommand::is_next_word_key(w_key_shift));

        let w_key_alt = KeyEvent::new(KeyCode::Char('w'), KeyModifiers::ALT);
        assert!(!NextWordCommand::is_next_word_key(w_key_alt));
    }

    #[test]
    fn is_next_word_key_should_reject_other_keys() {
        let b_key = KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE);
        assert!(!NextWordCommand::is_next_word_key(b_key));

        let e_key = KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE);
        assert!(!NextWordCommand::is_next_word_key(e_key));

        let right_key = KeyEvent::new(KeyCode::Right, KeyModifiers::NONE);
        assert!(!NextWordCommand::is_next_word_key(right_key));
    }

    #[test]
    fn next_word_command_should_generate_events_on_execution() {
        use crate::repl::models::AppState;
        use crate::repl::services::Services;

        let command = NextWordCommand::new();
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
        let command = NextWordCommand;
        assert_eq!(command.name(), "NextWordCommand");
    }

    #[test]
    fn is_navigation_mode_should_identify_correct_modes() {
        // Navigation modes
        assert!(NextWordCommand::is_navigation_mode(EditorMode::Normal));
        assert!(NextWordCommand::is_navigation_mode(EditorMode::Visual));
        assert!(NextWordCommand::is_navigation_mode(EditorMode::VisualLine));
        assert!(NextWordCommand::is_navigation_mode(EditorMode::VisualBlock));

        // Non-navigation modes
        assert!(!NextWordCommand::is_navigation_mode(EditorMode::Insert));
        assert!(!NextWordCommand::is_navigation_mode(
            EditorMode::VisualBlockInsert
        ));
        assert!(!NextWordCommand::is_navigation_mode(EditorMode::Command));
        assert!(!NextWordCommand::is_navigation_mode(EditorMode::GPrefix));
    }
}

// Auto-register this command using the inventory system
register_command!(NextWordCommand, "NextWordCommand");
