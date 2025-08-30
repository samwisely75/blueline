//! # Paste At Cursor Command
//!
//! Command to paste yanked text at the current cursor position.
//! This command migrates the functionality from handle_paste_at_cursor
//! in the app_view_model to the unified command architecture.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::register_command;
use crate::repl::models::events::view_events::ViewEvent;
use crate::repl::models::pane_state::EditorMode;
use crate::repl::unified_commands::{Command, CommandContext, ExecutionContext};

/// Command to paste yanked text at current cursor position
///
/// This command:
/// 1. Handles Ctrl+V key combination in Normal mode
/// 2. Gets yanked text from YankService
/// 3. Uses paste_with_type() to paste at cursor position with type awareness
/// 4. Updates status messages and logs the operation
/// 5. Only works in writable panes
#[derive(Default)]
pub struct PasteAtCursorCommand;

impl PasteAtCursorCommand {
    /// Create new PasteAtCursorCommand
    pub fn new() -> Self {
        Self
    }
}

impl Command for PasteAtCursorCommand {
    fn is_relevant(
        &self,
        key_event: KeyEvent,
        mode: EditorMode,
        context: &CommandContext,
    ) -> bool {
        // Ctrl+V key in Normal mode (standard paste shortcut)
        matches!(key_event.code, KeyCode::Char('v'))
            && key_event.modifiers == KeyModifiers::CONTROL
            && mode == EditorMode::Normal
            && !context.is_read_only
    }

    fn execute(&self, context: &mut ExecutionContext) -> Result<Vec<ViewEvent>> {
        // Get from YankService
        if let Some(yank_entry) = context.services.yank.paste() {
            tracing::debug!(
                "Retrieved yank entry with type: {:?}, text length: {}",
                yank_entry.yank_type,
                yank_entry.text.len()
            );

            // Paste the text at current position (before cursor) using type-aware paste
            context.app_state.paste_with_type(&yank_entry)?;

            let char_count = yank_entry.text.chars().count();
            let line_count = yank_entry.text.lines().count();

            // Clear any previous status message (e.g., "1 line yanked")
            context.app_state.clear_status_message();

            tracing::info!(
                "Pasted {} characters ({} lines) at cursor as {:?}",
                char_count,
                line_count,
                yank_entry.yank_type
            );
        } else {
            context
                .app_state
                .set_status_message("Nothing to paste".to_string());
            tracing::warn!("No text in yank buffer to paste");
        }

        // Return UI update events
        Ok(vec![
            ViewEvent::CurrentAreaRedrawRequired,
            ViewEvent::StatusBarUpdateRequired,
        ])
    }

    fn name(&self) -> &'static str {
        "PasteAtCursorCommand"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::models::pane_state::Pane;

    #[test]
    fn paste_at_cursor_command_should_return_correct_name() {
        let command = PasteAtCursorCommand::new();
        assert_eq!(command.name(), "PasteAtCursorCommand");
    }

    #[test]
    fn paste_at_cursor_command_should_be_relevant_for_ctrl_v_in_normal_mode() {
        let command = PasteAtCursorCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
        };

        let ctrl_v_key = KeyEvent::new(KeyCode::Char('v'), KeyModifiers::CONTROL);
        assert!(command.is_relevant(ctrl_v_key, EditorMode::Normal, &context));
    }

    #[test]
    fn paste_at_cursor_command_should_not_be_relevant_without_ctrl_modifier() {
        let command = PasteAtCursorCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
        };

        // Plain 'v' key should not be relevant
        let v_key = KeyEvent::new(KeyCode::Char('v'), KeyModifiers::NONE);
        assert!(!command.is_relevant(v_key, EditorMode::Normal, &context));

        // Shift+v should not be relevant
        let shift_v_key = KeyEvent::new(KeyCode::Char('V'), KeyModifiers::SHIFT);
        assert!(!command.is_relevant(shift_v_key, EditorMode::Normal, &context));
    }

    #[test]
    fn paste_at_cursor_command_should_not_be_relevant_in_insert_mode() {
        let command = PasteAtCursorCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::Insert,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
        };

        let ctrl_v_key = KeyEvent::new(KeyCode::Char('v'), KeyModifiers::CONTROL);
        assert!(!command.is_relevant(ctrl_v_key, EditorMode::Insert, &context));
    }

    #[test]
    fn paste_at_cursor_command_should_not_be_relevant_in_visual_modes() {
        let command = PasteAtCursorCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::Visual,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
        };

        let ctrl_v_key = KeyEvent::new(KeyCode::Char('v'), KeyModifiers::CONTROL);
        assert!(!command.is_relevant(ctrl_v_key, EditorMode::Visual, &context));
        assert!(!command.is_relevant(ctrl_v_key, EditorMode::VisualLine, &context));
        assert!(!command.is_relevant(ctrl_v_key, EditorMode::VisualBlock, &context));
    }

    #[test]
    fn paste_at_cursor_command_should_not_be_relevant_in_read_only_pane() {
        let command = PasteAtCursorCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Response,
            is_read_only: true,
            has_selection: false,
        };

        let ctrl_v_key = KeyEvent::new(KeyCode::Char('v'), KeyModifiers::CONTROL);
        assert!(!command.is_relevant(ctrl_v_key, EditorMode::Normal, &context));
    }

    #[test]
    fn paste_at_cursor_command_should_not_be_relevant_for_other_keys() {
        let command = PasteAtCursorCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
        };

        // Test other Ctrl combinations
        let ctrl_c_key = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert!(!command.is_relevant(ctrl_c_key, EditorMode::Normal, &context));

        let ctrl_x_key = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::CONTROL);
        assert!(!command.is_relevant(ctrl_x_key, EditorMode::Normal, &context));

        // Test other keys
        let enter_key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert!(!command.is_relevant(enter_key, EditorMode::Normal, &context));
    }

    #[test]
    fn default_should_create_new_instance() {
        let command = PasteAtCursorCommand;
        assert_eq!(command.name(), "PasteAtCursorCommand");
    }
}

// Auto-register this command using the inventory system
register_command!(PasteAtCursorCommand, "PasteAtCursorCommand");