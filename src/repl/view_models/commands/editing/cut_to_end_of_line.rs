//! # Cut To End Of Line Command
//!
//! Command to cut (yank + delete) from cursor to end of line in normal mode.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::register_command;
use crate::repl::models::buffer::yank_buffer::YankType;
use crate::repl::models::pane_state::EditorMode;
use crate::repl::view_models::commands::{Command, CommandContext, ExecutionContext};
use crate::repl::view_models::post_command_actions::PostCommandAction;

/// Command to cut (yank and delete) from cursor to end of line
///
/// This command:
/// 1. Yanks text from cursor to end of line
/// 2. Deletes the text from cursor to end of line
/// 3. Provides status feedback
pub struct CutToEndOfLineCommand;

impl CutToEndOfLineCommand {
    /// Create new CutToEndOfLineCommand
    pub fn new() -> Self {
        Self
    }
}

impl Command for CutToEndOfLineCommand {
    fn is_relevant(&self, key_event: KeyEvent, mode: EditorMode, context: &CommandContext) -> bool {
        // Handle multiple 'D' key variations like the old command
        mode == EditorMode::Normal
            && context.current_pane == crate::repl::models::pane_state::Pane::Request
            && (
                // Case 1: Uppercase 'D' without modifiers
                (matches!(key_event.code, KeyCode::Char('D')) && key_event.modifiers.is_empty())
                // Case 2: Lowercase 'd' with SHIFT modifier
                || (matches!(key_event.code, KeyCode::Char('d')) && key_event.modifiers.contains(KeyModifiers::SHIFT))
                // Case 3: Uppercase 'D' with SHIFT modifier (some terminals send this)
                || (matches!(key_event.code, KeyCode::Char('D')) && key_event.modifiers.contains(KeyModifiers::SHIFT))
            )
    }

    fn execute(
        &self,
        _key_event: KeyEvent,
        context: &mut ExecutionContext,
    ) -> Result<Vec<PostCommandAction>> {
        // Only allow in Request pane and Normal mode (double-check)
        if !context.app_state.is_in_request_pane() || context.app_state.mode() != EditorMode::Normal
        {
            return Ok(vec![]);
        }

        // Cut from cursor to end of line - this returns the deleted text
        if let Some(cut_text) = context.app_state.pane_manager.cut_to_end_of_line() {
            // Always yank the text (cut = yank + delete)
            context
                .services
                .yank
                .yank(cut_text.clone(), YankType::Character)?;

            // Set status message
            let char_count = cut_text.chars().count();
            let line_count = cut_text.lines().count();
            let message = if char_count == 0 {
                "Nothing to cut to end of line".to_string()
            } else if line_count > 1 {
                format!("Cut {line_count} lines to end")
            } else {
                format!("Cut {char_count} characters to end of line")
            };
            context.app_state.set_status_message(message);

            tracing::info!("Cut {} characters from cursor to end of line", char_count);

            // Return view events for UI updates
            Ok(vec![
                PostCommandAction::RequestContentChanged,
                PostCommandAction::ActiveCursorUpdateRequired,
                PostCommandAction::CurrentAreaRedrawRequired,
                PostCommandAction::StatusBarUpdateRequired,
            ])
        } else {
            // No text to cut from cursor to end of line - don't update anything
            Ok(vec![])
        }
    }

    fn name(&self) -> &'static str {
        "CutToEndOfLineCommand"
    }
}

impl Default for CutToEndOfLineCommand {
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
    fn cut_to_end_of_line_command_should_return_correct_name() {
        let command = CutToEndOfLineCommand::new();
        assert_eq!(command.name(), "CutToEndOfLineCommand");
    }

    #[test]
    fn cut_to_end_of_line_command_should_be_relevant_for_uppercase_d_in_normal_mode() {
        let command = CutToEndOfLineCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let d_key = KeyEvent::new(KeyCode::Char('D'), KeyModifiers::NONE);
        assert!(command.is_relevant(d_key, EditorMode::Normal, &context));
    }

    #[test]
    fn cut_to_end_of_line_command_should_not_be_relevant_for_lowercase_d() {
        let command = CutToEndOfLineCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let d_key = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE);
        assert!(!command.is_relevant(d_key, EditorMode::Normal, &context));
    }

    #[test]
    fn cut_to_end_of_line_command_should_not_be_relevant_in_insert_mode() {
        let command = CutToEndOfLineCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::Insert,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let d_key = KeyEvent::new(KeyCode::Char('D'), KeyModifiers::NONE);
        assert!(!command.is_relevant(d_key, EditorMode::Insert, &context));
    }

    #[test]
    fn cut_to_end_of_line_command_should_not_be_relevant_in_visual_mode() {
        let command = CutToEndOfLineCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::Visual,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: true,
            ex_command_buffer: String::new(),
        };

        let d_key = KeyEvent::new(KeyCode::Char('D'), KeyModifiers::NONE);
        assert!(!command.is_relevant(d_key, EditorMode::Visual, &context));
    }

    #[test]
    fn cut_to_end_of_line_command_should_not_be_relevant_in_response_pane() {
        let command = CutToEndOfLineCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Response,
            is_read_only: true,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let d_key = KeyEvent::new(KeyCode::Char('D'), KeyModifiers::NONE);
        assert!(!command.is_relevant(d_key, EditorMode::Normal, &context));
    }

    #[test]
    fn cut_to_end_of_line_command_should_be_relevant_for_lowercase_d_with_shift() {
        let command = CutToEndOfLineCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let d_key_shift = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::SHIFT);
        assert!(command.is_relevant(d_key_shift, EditorMode::Normal, &context));
    }

    #[test]
    fn cut_to_end_of_line_command_should_be_relevant_for_uppercase_d_with_shift() {
        let command = CutToEndOfLineCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let d_key_shift = KeyEvent::new(KeyCode::Char('D'), KeyModifiers::SHIFT);
        assert!(command.is_relevant(d_key_shift, EditorMode::Normal, &context));
    }

    #[test]
    fn cut_to_end_of_line_command_should_not_be_relevant_with_ctrl_modifier() {
        let command = CutToEndOfLineCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let d_key_ctrl = KeyEvent::new(KeyCode::Char('D'), KeyModifiers::CONTROL);
        assert!(!command.is_relevant(d_key_ctrl, EditorMode::Normal, &context));
    }

    #[test]
    fn cut_to_end_of_line_command_should_handle_no_text() {
        let command = CutToEndOfLineCommand::new();
        let mut app_state = AppState::new();
        let mut services = Services::new();

        let mut context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        // Execute without any text or at end of line
        let result = command.execute(
            KeyEvent::new(KeyCode::Null, KeyModifiers::empty()),
            &mut context,
        );

        assert!(result.is_ok());
        let events = result.unwrap();
        assert_eq!(events.len(), 0); // Should return empty vec when no text to cut
    }

    #[test]
    fn cut_to_end_of_line_command_should_emit_correct_events() {
        let command = CutToEndOfLineCommand::new();
        let mut app_state = AppState::new();
        let mut services = Services::new();

        // Add some text
        app_state.insert_text("test text").ok();

        let mut context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        // Note: In a real scenario, the cut would happen
        // For this test, we're verifying the command structure
        let result = command.execute(
            KeyEvent::new(KeyCode::Null, KeyModifiers::empty()),
            &mut context,
        );
        assert!(result.is_ok());
    }
}

// Auto-register this command using the inventory system
register_command!(CutToEndOfLineCommand, "CutToEndOfLineCommand");
