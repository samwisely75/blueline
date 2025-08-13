//! # Text Editing Commands
//!
//! Commands for text insertion, deletion, and line operations
//! in insert mode.

use crate::repl::events::{EditorMode, Pane};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::{Command, CommandContext, CommandEvent};

/// Insert character in insert mode
pub struct InsertCharCommand;

impl Command for InsertCharCommand {
    fn is_relevant(&self, context: &CommandContext, event: &KeyEvent) -> bool {
        match event.code {
            KeyCode::Char(ch) => {
                !event.modifiers.contains(KeyModifiers::CONTROL)
                    && !ch.is_control()
                    && context.state.current_mode == EditorMode::Insert
                    && context.state.current_pane == Pane::Request
            }
            _ => false,
        }
    }

    fn execute(&self, event: KeyEvent, context: &CommandContext) -> Result<Vec<CommandEvent>> {
        if let KeyCode::Char(ch) = event.code {
            let text_event =
                CommandEvent::text_insert(ch.to_string(), context.state.cursor_position);
            Ok(vec![text_event])
        } else {
            Ok(vec![])
        }
    }

    fn name(&self) -> &'static str {
        "InsertChar"
    }
}

/// Insert new line (Enter in insert mode)
pub struct InsertNewLineCommand;

impl Command for InsertNewLineCommand {
    fn is_relevant(&self, context: &CommandContext, event: &KeyEvent) -> bool {
        matches!(event.code, KeyCode::Enter)
            && context.state.current_mode == EditorMode::Insert
            && context.state.current_pane == Pane::Request
    }

    fn execute(&self, _event: KeyEvent, context: &CommandContext) -> Result<Vec<CommandEvent>> {
        let text_event = CommandEvent::text_insert("\n".to_string(), context.state.cursor_position);
        Ok(vec![text_event])
    }

    fn name(&self) -> &'static str {
        "InsertNewLine"
    }
}

/// Delete character before cursor (Backspace in insert mode)
pub struct DeleteCharCommand;

impl Command for DeleteCharCommand {
    fn is_relevant(&self, context: &CommandContext, event: &KeyEvent) -> bool {
        matches!(event.code, KeyCode::Backspace)
            && context.state.current_mode == EditorMode::Insert
            && context.state.current_pane == Pane::Request
    }

    fn execute(&self, _event: KeyEvent, context: &CommandContext) -> Result<Vec<CommandEvent>> {
        use super::MovementDirection;
        tracing::debug!(
            "🗑️  DeleteCharBeforeCursorCommand::execute - creating TextDeleteRequested event"
        );
        tracing::debug!(
            "🗑️  Current cursor position: {:?}",
            context.state.cursor_position
        );
        tracing::debug!(
            "🗑️  Current mode: {:?}, pane: {:?}",
            context.state.current_mode,
            context.state.current_pane
        );

        let delete_event = CommandEvent::TextDeleteRequested {
            position: context.state.cursor_position,
            amount: 1,
            direction: MovementDirection::Left,
        };

        tracing::debug!("🗑️  Created TextDeleteRequested event: {:?}", delete_event);
        Ok(vec![delete_event])
    }

    fn name(&self) -> &'static str {
        "DeleteChar"
    }
}

/// Delete character at cursor (Delete key)
pub struct DeleteCharAtCursorCommand;

impl Command for DeleteCharAtCursorCommand {
    fn is_relevant(&self, context: &CommandContext, event: &KeyEvent) -> bool {
        matches!(event.code, KeyCode::Delete)
            && context.state.current_mode == EditorMode::Insert
            && context.state.current_pane == Pane::Request
    }

    fn execute(&self, _event: KeyEvent, context: &CommandContext) -> Result<Vec<CommandEvent>> {
        use super::MovementDirection;
        let delete_event = CommandEvent::TextDeleteRequested {
            position: context.state.cursor_position,
            amount: 1,
            direction: MovementDirection::Right,
        };
        Ok(vec![delete_event])
    }

    fn name(&self) -> &'static str {
        "DeleteCharAtCursor"
    }
}

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

/// Paste yanked text before cursor position
pub struct PasteBeforeCommand;

impl Command for PasteBeforeCommand {
    fn is_relevant(&self, context: &CommandContext, event: &KeyEvent) -> bool {
        matches!(event.code, KeyCode::Char('P'))
            && context.state.current_mode == EditorMode::Normal
            && context.state.current_pane == Pane::Request
            && event.modifiers == KeyModifiers::SHIFT
    }

    fn execute(&self, _event: KeyEvent, _context: &CommandContext) -> Result<Vec<CommandEvent>> {
        Ok(vec![CommandEvent::paste_before()])
    }

    fn name(&self) -> &'static str {
        "PasteBefore"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::commands::{CommandContext, MovementDirection, ViewModelSnapshot};
    use crate::repl::events::{EditorMode, LogicalPosition, Pane};
    use crossterm::event::KeyModifiers;

    fn create_test_key_event(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::empty())
    }

    fn create_test_context() -> CommandContext {
        CommandContext {
            state: ViewModelSnapshot {
                current_mode: EditorMode::Insert,
                current_pane: Pane::Request,
                cursor_position: LogicalPosition { line: 0, column: 0 },
                request_text: String::new(),
                response_text: String::new(),
                terminal_dimensions: (80, 24),
            },
        }
    }

    #[test]
    fn insert_char_should_be_relevant_for_printable_chars_in_insert_mode() {
        let context = create_test_context();
        let cmd = InsertCharCommand;
        let event = create_test_key_event(KeyCode::Char('a'));

        assert!(cmd.is_relevant(&context, &event));
    }

    #[test]
    fn insert_char_should_be_relevant_for_space_in_insert_mode() {
        let context = create_test_context();
        let cmd = InsertCharCommand;
        let event = create_test_key_event(KeyCode::Char(' '));

        assert!(cmd.is_relevant(&context, &event));
    }

    #[test]
    fn insert_char_should_be_relevant_for_capital_g_with_shift() {
        let context = create_test_context();
        let cmd = InsertCharCommand;
        let event = KeyEvent::new(KeyCode::Char('G'), KeyModifiers::SHIFT);

        assert!(cmd.is_relevant(&context, &event));
    }

    #[test]
    fn insert_char_should_be_relevant_for_capital_e_with_shift() {
        let context = create_test_context();
        let cmd = InsertCharCommand;
        let event = KeyEvent::new(KeyCode::Char('E'), KeyModifiers::SHIFT);

        assert!(cmd.is_relevant(&context, &event));
    }

    #[test]
    fn insert_char_should_be_relevant_for_capital_t_with_shift() {
        let context = create_test_context();
        let cmd = InsertCharCommand;
        let event = KeyEvent::new(KeyCode::Char('T'), KeyModifiers::SHIFT);

        assert!(cmd.is_relevant(&context, &event));
    }

    #[test]
    fn insert_char_should_be_relevant_for_japanese_hiragana_in_insert_mode() {
        let context = create_test_context();
        let cmd = InsertCharCommand;
        let event = create_test_key_event(KeyCode::Char('あ')); // Hiragana 'a'

        assert!(cmd.is_relevant(&context, &event));
    }

    #[test]
    fn insert_char_should_be_relevant_for_japanese_katakana_in_insert_mode() {
        let context = create_test_context();
        let cmd = InsertCharCommand;
        let event = create_test_key_event(KeyCode::Char('ア')); // Katakana 'a'

        assert!(cmd.is_relevant(&context, &event));
    }

    #[test]
    fn insert_char_should_be_relevant_for_japanese_kanji_in_insert_mode() {
        let context = create_test_context();
        let cmd = InsertCharCommand;
        let event = create_test_key_event(KeyCode::Char('漢')); // Kanji character

        assert!(cmd.is_relevant(&context, &event));
    }

    #[test]
    fn insert_char_should_execute_japanese_character_insertion() {
        let context = create_test_context();
        let cmd = InsertCharCommand;
        let event = create_test_key_event(KeyCode::Char('こ')); // Single hiragana character

        let result = cmd.execute(event, &context).unwrap();
        assert_eq!(result.len(), 1);
        if let CommandEvent::TextInsertRequested { text, .. } = &result[0] {
            assert_eq!(text, "こ");
        } else {
            panic!("Expected TextInsertRequested event");
        }
    }

    #[test]
    fn insert_char_should_not_be_relevant_in_normal_mode() {
        let mut context = create_test_context();
        context.state.current_mode = EditorMode::Normal;
        let cmd = InsertCharCommand;
        let event = create_test_key_event(KeyCode::Char('a'));

        assert!(!cmd.is_relevant(&context, &event));
    }

    #[test]
    fn delete_char_should_be_relevant_for_backspace_in_insert_mode() {
        let context = create_test_context();
        let cmd = DeleteCharCommand;
        let event = create_test_key_event(KeyCode::Backspace);

        assert!(cmd.is_relevant(&context, &event));
    }

    #[test]
    fn delete_char_at_cursor_should_be_relevant_for_delete_key() {
        let context = create_test_context();
        let cmd = DeleteCharAtCursorCommand;
        let event = create_test_key_event(KeyCode::Delete);

        assert!(cmd.is_relevant(&context, &event));
    }

    #[test]
    fn delete_char_at_cursor_should_not_be_relevant_in_normal_mode() {
        let mut context = create_test_context();
        context.state.current_mode = EditorMode::Normal;
        let cmd = DeleteCharAtCursorCommand;
        let event = create_test_key_event(KeyCode::Delete);

        assert!(!cmd.is_relevant(&context, &event));
    }

    #[test]
    fn delete_char_at_cursor_should_execute_right_deletion() {
        let context = create_test_context();
        let cmd = DeleteCharAtCursorCommand;
        let event = create_test_key_event(KeyCode::Delete);

        let result = cmd.execute(event, &context).unwrap();
        assert_eq!(result.len(), 1);
        if let CommandEvent::TextDeleteRequested {
            direction, amount, ..
        } = &result[0]
        {
            assert_eq!(*direction, MovementDirection::Right);
            assert_eq!(*amount, 1);
        } else {
            panic!("Expected TextDeleteRequested event");
        }
    }
}
