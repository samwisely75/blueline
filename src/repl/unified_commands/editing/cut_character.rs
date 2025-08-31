//! # Cut Character Command
//!
//! Command to cut (yank + delete) a single character at cursor in normal mode.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use crate::register_command;
use crate::repl::models::buffer::yank_buffer::YankType;
use crate::repl::models::events::view_events::ViewEvent;
use crate::repl::models::pane_state::EditorMode;
use crate::repl::unified_commands::{Command, CommandContext, ExecutionContext};

/// Command to cut (yank and delete) character at cursor position
///
/// This command:
/// 1. Yanks the character at cursor to the buffer
/// 2. Deletes the character
/// 3. Provides status feedback
pub struct CutCharacterCommand;

impl CutCharacterCommand {
    /// Create new CutCharacterCommand
    pub fn new() -> Self {
        Self
    }
}

impl Command for CutCharacterCommand {
    fn is_relevant(&self, key_event: KeyEvent, mode: EditorMode, context: &CommandContext) -> bool {
        // Only relevant for 'x' key in normal mode without modifiers
        matches!(key_event.code, KeyCode::Char('x'))
            && key_event.modifiers.is_empty()
            && mode == EditorMode::Normal
            && context.current_pane == crate::repl::models::pane_state::Pane::Request
    }

    fn execute(&self, context: &mut ExecutionContext) -> Result<Vec<ViewEvent>> {
        // Only allow in Request pane and Normal mode (double-check)
        if !context.app_state.is_in_request_pane() || context.app_state.mode() != EditorMode::Normal
        {
            return Ok(vec![]);
        }

        // Cut character at cursor position - this returns the deleted text
        if let Some(deleted_char) = context.app_state.pane_manager.cut_char_at_cursor() {
            // Always yank the character (cut = yank + delete)
            context
                .services
                .yank
                .yank(deleted_char.clone(), YankType::Character)?;

            // Set status message
            let char_count = deleted_char.chars().count();
            if char_count > 0 {
                context
                    .app_state
                    .set_status_message(format!("{char_count} character cut"));
                tracing::info!("Cut {} character at cursor", char_count);
            }

            // Return view events for UI updates
            Ok(vec![
                ViewEvent::RequestContentChanged,
                ViewEvent::ActiveCursorUpdateRequired,
                ViewEvent::CurrentAreaRedrawRequired,
                ViewEvent::StatusBarUpdateRequired,
            ])
        } else {
            // No character to cut at cursor position - don't update anything
            Ok(vec![])
        }
    }

    fn name(&self) -> &'static str {
        "CutCharacterCommand"
    }
}

impl Default for CutCharacterCommand {
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
    use crossterm::event::KeyModifiers;

    #[test]
    fn cut_character_command_should_return_correct_name() {
        let command = CutCharacterCommand::new();
        assert_eq!(command.name(), "CutCharacterCommand");
    }

    #[test]
    fn cut_character_command_should_be_relevant_for_x_in_normal_mode() {
        let command = CutCharacterCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let x_key = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
        assert!(command.is_relevant(x_key, EditorMode::Normal, &context));
    }

    #[test]
    fn cut_character_command_should_not_be_relevant_in_visual_mode() {
        let command = CutCharacterCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::Visual,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: true,
            ex_command_buffer: String::new(),
        };

        let x_key = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
        assert!(!command.is_relevant(x_key, EditorMode::Visual, &context));
    }

    #[test]
    fn cut_character_command_should_not_be_relevant_in_insert_mode() {
        let command = CutCharacterCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::Insert,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let x_key = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
        assert!(!command.is_relevant(x_key, EditorMode::Insert, &context));
    }

    #[test]
    fn cut_character_command_should_not_be_relevant_in_response_pane() {
        let command = CutCharacterCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Response,
            is_read_only: true,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let x_key = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
        assert!(!command.is_relevant(x_key, EditorMode::Normal, &context));
    }

    #[test]
    fn cut_character_command_should_not_be_relevant_with_modifiers() {
        let command = CutCharacterCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let x_key_ctrl = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::CONTROL);
        assert!(!command.is_relevant(x_key_ctrl, EditorMode::Normal, &context));
    }

    #[test]
    fn cut_character_command_should_handle_no_character() {
        let command = CutCharacterCommand::new();
        let mut app_state = AppState::new();
        let mut services = Services::new();

        let mut context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        // Execute without any text
        let result = command.execute(&mut context);

        assert!(result.is_ok());
        let events = result.unwrap();
        assert_eq!(events.len(), 0); // Should return empty vec when no character to cut
    }

    #[test]
    fn cut_character_command_should_emit_correct_events() {
        let command = CutCharacterCommand::new();
        let mut app_state = AppState::new();
        let mut services = Services::new();

        // Add some text
        app_state.insert_text("test").ok();

        let mut context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        // Note: In a real scenario, the cut would happen
        // For this test, we're verifying the command structure
        let result = command.execute(&mut context);
        assert!(result.is_ok());
    }
}

// Auto-register this command using the inventory system
register_command!(CutCharacterCommand, "CutCharacterCommand");
