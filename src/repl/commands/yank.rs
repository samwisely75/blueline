//! Yank and paste commands for text manipulation

use super::{Command, CommandContext, CommandEvent};
use crate::repl::events::{EditorMode, Pane};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Yank (copy) selected text in visual mode
pub struct YankCommand;

impl Command for YankCommand {
    fn is_relevant(&self, context: &CommandContext, event: &KeyEvent) -> bool {
        matches!(event.code, KeyCode::Char('y'))
            && context.state.current_mode == EditorMode::Visual
            && event.modifiers.is_empty()
    }

    fn execute(&self, _event: KeyEvent, _context: &CommandContext) -> Result<Vec<CommandEvent>> {
        // Return yank event and exit visual mode
        Ok(vec![
            CommandEvent::yank_selection(),
            CommandEvent::mode_change(EditorMode::Normal),
        ])
    }

    fn name(&self) -> &'static str {
        "Yank"
    }
}

/// Paste yanked text after cursor position
pub struct PasteAfterCommand;

impl Command for PasteAfterCommand {
    fn is_relevant(&self, context: &CommandContext, event: &KeyEvent) -> bool {
        matches!(event.code, KeyCode::Char('p'))
            && context.state.current_mode == EditorMode::Normal
            && context.state.current_pane == Pane::Request
            && event.modifiers.is_empty()
    }

    fn execute(&self, _event: KeyEvent, _context: &CommandContext) -> Result<Vec<CommandEvent>> {
        Ok(vec![CommandEvent::paste_after()])
    }

    fn name(&self) -> &'static str {
        "PasteAfter"
    }
}

/// Paste yanked text at current cursor position
pub struct PasteAtCursorCommand;

impl Command for PasteAtCursorCommand {
    fn is_relevant(&self, context: &CommandContext, event: &KeyEvent) -> bool {
        matches!(event.code, KeyCode::Char('P'))
            && context.state.current_mode == EditorMode::Normal
            && context.state.current_pane == Pane::Request
            && event.modifiers == KeyModifiers::SHIFT
    }

    fn execute(&self, _event: KeyEvent, _context: &CommandContext) -> Result<Vec<CommandEvent>> {
        Ok(vec![CommandEvent::paste_at_cursor()])
    }

    fn name(&self) -> &'static str {
        "PasteAtCursor"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::commands::ViewModelSnapshot;
    use crate::repl::events::LogicalPosition;

    fn create_test_context(mode: EditorMode, pane: Pane) -> CommandContext {
        CommandContext {
            state: ViewModelSnapshot {
                current_mode: mode,
                current_pane: pane,
                cursor_position: LogicalPosition { line: 0, column: 0 },
                request_text: String::new(),
                response_text: String::new(),
                terminal_dimensions: (80, 24),
            },
        }
    }

    #[test]
    fn yank_command_should_be_relevant_in_visual_mode() {
        let context = create_test_context(EditorMode::Visual, Pane::Request);
        let event = KeyEvent::new(KeyCode::Char('y'), KeyModifiers::empty());
        let command = YankCommand;
        assert!(command.is_relevant(&context, &event));
    }

    #[test]
    fn yank_command_should_not_be_relevant_in_normal_mode() {
        let context = create_test_context(EditorMode::Normal, Pane::Request);
        let event = KeyEvent::new(KeyCode::Char('y'), KeyModifiers::empty());
        let command = YankCommand;
        assert!(!command.is_relevant(&context, &event));
    }

    #[test]
    fn paste_after_should_be_relevant_for_p_in_normal_mode() {
        let context = create_test_context(EditorMode::Normal, Pane::Request);
        let event = KeyEvent::new(KeyCode::Char('p'), KeyModifiers::empty());
        let command = PasteAfterCommand;
        assert!(command.is_relevant(&context, &event));
    }

    #[test]
    fn paste_at_cursor_should_be_relevant_for_shift_p_in_normal_mode() {
        let context = create_test_context(EditorMode::Normal, Pane::Request);
        let event = KeyEvent::new(KeyCode::Char('P'), KeyModifiers::SHIFT);
        let command = PasteAtCursorCommand;
        assert!(command.is_relevant(&context, &event));
    }
}
