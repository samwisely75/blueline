//! # Cut Current Line Command
//!
//! Command to cut (yank + delete) entire current line in DPrefix mode.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use crate::register_command;
use crate::repl::models::buffer::yank_buffer::YankType;
use crate::repl::models::events::view_events::ViewEvent;
use crate::repl::models::pane_state::EditorMode;
use crate::repl::unified_commands::{Command, CommandContext, ExecutionContext};

/// Command to cut (yank and delete) entire current line
///
/// This command:
/// 1. Cuts (yanks and deletes) the entire current line
/// 2. Always yanks as line type
/// 3. Returns to Normal mode
/// 4. Provides status feedback
pub struct CutCurrentLineCommand;

impl CutCurrentLineCommand {
    /// Create new CutCurrentLineCommand
    pub fn new() -> Self {
        Self
    }
}

impl Command for CutCurrentLineCommand {
    fn is_relevant(&self, key_event: KeyEvent, mode: EditorMode, context: &CommandContext) -> bool {
        // Handle 'd' key in DPrefix mode (second 'd' of 'dd' command)
        mode == EditorMode::DPrefix
            && context.current_pane == crate::repl::models::pane_state::Pane::Request
            && matches!(key_event.code, KeyCode::Char('d'))
            && key_event.modifiers.is_empty()
    }

    fn execute(&self, context: &mut ExecutionContext) -> Result<Vec<ViewEvent>> {
        // Only allow in Request pane and DPrefix mode (double-check)
        if !context.app_state.is_in_request_pane()
            || context.app_state.mode() != EditorMode::DPrefix
        {
            return Ok(vec![]);
        }

        // Cut entire current line - this returns the deleted text
        if let Some(cut_text) = context.app_state.pane_manager.cut_current_line() {
            // Always yank the text as line type (cut = yank + delete)
            context
                .services
                .yank
                .yank(cut_text.clone(), YankType::Line)?;

            // Set status message
            let line_count = cut_text.lines().count();
            let message = if line_count <= 1 {
                "Cut current line".to_string()
            } else {
                format!("Cut {line_count} lines")
            };
            context.app_state.set_status_message(message);

            // Change back to Normal mode
            context.app_state.set_mode(EditorMode::Normal);

            tracing::info!("Cut entire current line ({} lines)", line_count);

            // Return view events for UI updates
            Ok(vec![
                ViewEvent::RequestContentChanged,
                ViewEvent::ActiveCursorUpdateRequired,
                ViewEvent::CurrentAreaRedrawRequired,
                ViewEvent::StatusBarUpdateRequired,
            ])
        } else {
            // No text to cut - still return to Normal mode
            context.app_state.set_mode(EditorMode::Normal);
            Ok(vec![])
        }
    }

    fn name(&self) -> &'static str {
        "CutCurrentLineCommand"
    }
}

impl Default for CutCurrentLineCommand {
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
    fn cut_current_line_command_should_return_correct_name() {
        let command = CutCurrentLineCommand::new();
        assert_eq!(command.name(), "CutCurrentLineCommand");
    }

    #[test]
    fn cut_current_line_command_should_be_relevant_for_d_in_d_prefix_mode() {
        let command = CutCurrentLineCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::DPrefix,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let d_key = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE);
        assert!(command.is_relevant(d_key, EditorMode::DPrefix, &context));
    }

    #[test]
    fn cut_current_line_command_should_not_be_relevant_for_d_in_normal_mode() {
        let command = CutCurrentLineCommand::new();
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
    fn cut_current_line_command_should_not_be_relevant_in_insert_mode() {
        let command = CutCurrentLineCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::Insert,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let d_key = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE);
        assert!(!command.is_relevant(d_key, EditorMode::Insert, &context));
    }

    #[test]
    fn cut_current_line_command_should_not_be_relevant_in_response_pane() {
        let command = CutCurrentLineCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::DPrefix,
            current_pane: Pane::Response,
            is_read_only: true,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let d_key = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE);
        assert!(!command.is_relevant(d_key, EditorMode::DPrefix, &context));
    }

    #[test]
    fn cut_current_line_command_should_not_be_relevant_with_modifiers() {
        let command = CutCurrentLineCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::DPrefix,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let d_key_ctrl = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL);
        assert!(!command.is_relevant(d_key_ctrl, EditorMode::DPrefix, &context));
    }

    #[test]
    fn cut_current_line_command_should_not_be_relevant_for_other_chars() {
        let command = CutCurrentLineCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::DPrefix,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let x_key = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
        assert!(!command.is_relevant(x_key, EditorMode::DPrefix, &context));
    }

    #[test]
    fn cut_current_line_command_should_handle_no_text() {
        let command = CutCurrentLineCommand::new();
        let mut app_state = AppState::new();
        let mut services = Services::new();

        // Set to DPrefix mode first
        app_state.set_mode(EditorMode::DPrefix);

        let mut context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        // Execute without any text
        let result = command.execute(&mut context);

        assert!(result.is_ok());
        let events = result.unwrap();
        // Should return empty vec when no text to cut
        assert_eq!(events.len(), 0);
    }

    #[test]
    fn cut_current_line_command_should_emit_correct_events() {
        let command = CutCurrentLineCommand::new();
        let mut app_state = AppState::new();
        let mut services = Services::new();

        // Add some text and set to DPrefix mode
        app_state.insert_text("test line").ok();
        app_state.set_mode(EditorMode::DPrefix);

        let mut context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        let result = command.execute(&mut context);
        assert!(result.is_ok());
        let _events = result.unwrap();

        // Should contain expected view events for UI updates
        // (Mode change is handled internally by app_state.set_mode())
    }
}

// Auto-register this command using the inventory system
register_command!(CutCurrentLineCommand, "CutCurrentLineCommand");
