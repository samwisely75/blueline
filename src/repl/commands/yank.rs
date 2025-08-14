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
            && matches!(
                context.state.current_mode,
                EditorMode::Visual | EditorMode::VisualLine | EditorMode::VisualBlock
            )
            && event.modifiers.is_empty()
    }

    fn execute(&self, _event: KeyEvent, _context: &CommandContext) -> Result<Vec<CommandEvent>> {
        // Return yank event - mode change handled by yank handler
        Ok(vec![CommandEvent::yank_selection()])
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

/// Delete selected text in visual mode
pub struct DeleteSelectionCommand;

impl Command for DeleteSelectionCommand {
    fn is_relevant(&self, context: &CommandContext, event: &KeyEvent) -> bool {
        matches!(event.code, KeyCode::Char('d'))
            && matches!(
                context.state.current_mode,
                EditorMode::Visual | EditorMode::VisualLine | EditorMode::VisualBlock
            )
            && context.state.current_pane == Pane::Request
            && event.modifiers.is_empty()
    }

    fn execute(&self, _event: KeyEvent, _context: &CommandContext) -> Result<Vec<CommandEvent>> {
        // Delete selection - mode change handled by delete handler
        Ok(vec![CommandEvent::delete_selection()])
    }

    fn name(&self) -> &'static str {
        "DeleteSelection"
    }
}

/// Cut (delete + yank) selected text in visual mode
pub struct CutSelectionCommand;

impl Command for CutSelectionCommand {
    fn is_relevant(&self, context: &CommandContext, event: &KeyEvent) -> bool {
        matches!(event.code, KeyCode::Char('x'))
            && matches!(
                context.state.current_mode,
                EditorMode::Visual | EditorMode::VisualLine | EditorMode::VisualBlock
            )
            && context.state.current_pane == Pane::Request
            && event.modifiers.is_empty()
    }

    fn execute(&self, _event: KeyEvent, _context: &CommandContext) -> Result<Vec<CommandEvent>> {
        // Cut selection (yank + delete) - mode change handled by cut handler
        Ok(vec![CommandEvent::cut_selection()])
    }

    fn name(&self) -> &'static str {
        "CutSelection"
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
                expand_tab: false,
                tab_width: 4,
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

    // Tests for DeleteSelectionCommand
    #[test]
    fn delete_selection_should_be_relevant_for_d_in_visual_mode() {
        let context = create_test_context(EditorMode::Visual, Pane::Request);
        let event = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::empty());
        let command = DeleteSelectionCommand;
        assert!(command.is_relevant(&context, &event));
    }

    #[test]
    fn delete_selection_should_be_relevant_for_d_in_visual_line_mode() {
        let context = create_test_context(EditorMode::VisualLine, Pane::Request);
        let event = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::empty());
        let command = DeleteSelectionCommand;
        assert!(command.is_relevant(&context, &event));
    }

    #[test]
    fn delete_selection_should_be_relevant_for_d_in_visual_block_mode() {
        let context = create_test_context(EditorMode::VisualBlock, Pane::Request);
        let event = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::empty());
        let command = DeleteSelectionCommand;
        assert!(command.is_relevant(&context, &event));
    }

    #[test]
    fn delete_selection_should_not_be_relevant_in_normal_mode() {
        let context = create_test_context(EditorMode::Normal, Pane::Request);
        let event = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::empty());
        let command = DeleteSelectionCommand;
        assert!(!command.is_relevant(&context, &event));
    }

    #[test]
    fn delete_selection_should_not_be_relevant_in_response_pane() {
        let context = create_test_context(EditorMode::Visual, Pane::Response);
        let event = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::empty());
        let command = DeleteSelectionCommand;
        assert!(!command.is_relevant(&context, &event));
    }

    #[test]
    fn delete_selection_should_execute_delete_and_mode_change() {
        let context = create_test_context(EditorMode::Visual, Pane::Request);
        let event = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::empty());
        let command = DeleteSelectionCommand;
        let result = command.execute(event, &context).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], CommandEvent::delete_selection());
    }

    // Tests for CutSelectionCommand
    #[test]
    fn cut_selection_should_be_relevant_for_x_in_visual_mode() {
        let context = create_test_context(EditorMode::Visual, Pane::Request);
        let event = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::empty());
        let command = CutSelectionCommand;
        assert!(command.is_relevant(&context, &event));
    }

    #[test]
    fn cut_selection_should_be_relevant_for_x_in_visual_line_mode() {
        let context = create_test_context(EditorMode::VisualLine, Pane::Request);
        let event = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::empty());
        let command = CutSelectionCommand;
        assert!(command.is_relevant(&context, &event));
    }

    #[test]
    fn cut_selection_should_be_relevant_for_x_in_visual_block_mode() {
        let context = create_test_context(EditorMode::VisualBlock, Pane::Request);
        let event = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::empty());
        let command = CutSelectionCommand;
        assert!(command.is_relevant(&context, &event));
    }

    #[test]
    fn cut_selection_should_not_be_relevant_in_normal_mode() {
        let context = create_test_context(EditorMode::Normal, Pane::Request);
        let event = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::empty());
        let command = CutSelectionCommand;
        assert!(!command.is_relevant(&context, &event));
    }

    #[test]
    fn cut_selection_should_not_be_relevant_in_response_pane() {
        let context = create_test_context(EditorMode::Visual, Pane::Response);
        let event = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::empty());
        let command = CutSelectionCommand;
        assert!(!command.is_relevant(&context, &event));
    }

    #[test]
    fn cut_selection_should_execute_cut_and_mode_change() {
        let context = create_test_context(EditorMode::Visual, Pane::Request);
        let event = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::empty());
        let command = CutSelectionCommand;
        let result = command.execute(event, &context).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], CommandEvent::cut_selection());
    }

    // Tests for enhanced YankCommand
    #[test]
    fn yank_command_should_be_relevant_in_visual_line_mode() {
        let context = create_test_context(EditorMode::VisualLine, Pane::Request);
        let event = KeyEvent::new(KeyCode::Char('y'), KeyModifiers::empty());
        let command = YankCommand;
        assert!(command.is_relevant(&context, &event));
    }

    #[test]
    fn yank_command_should_be_relevant_in_visual_block_mode() {
        let context = create_test_context(EditorMode::VisualBlock, Pane::Request);
        let event = KeyEvent::new(KeyCode::Char('y'), KeyModifiers::empty());
        let command = YankCommand;
        assert!(command.is_relevant(&context, &event));
    }
}
