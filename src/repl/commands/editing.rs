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
                    && matches!(
                        context.state.current_mode,
                        EditorMode::Insert | EditorMode::VisualBlockInsert
                    )
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
            && matches!(
                context.state.current_mode,
                EditorMode::Insert | EditorMode::VisualBlockInsert
            )
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

/// Insert tab character (Tab key in insert mode)
pub struct InsertTabCommand;

impl Command for InsertTabCommand {
    fn is_relevant(&self, context: &CommandContext, event: &KeyEvent) -> bool {
        matches!(event.code, KeyCode::Tab)
            && matches!(
                context.state.current_mode,
                EditorMode::Insert | EditorMode::VisualBlockInsert
            )
            && context.state.current_pane == Pane::Request
            && event.modifiers.is_empty()
    }

    fn execute(&self, _event: KeyEvent, context: &CommandContext) -> Result<Vec<CommandEvent>> {
        // Check if expandtab is enabled
        let text = if context.state.expand_tab {
            // Insert spaces instead of tab
            " ".repeat(context.state.tab_width)
        } else {
            // Insert actual tab character
            '\t'.to_string()
        };

        let text_event = CommandEvent::text_insert(text, context.state.cursor_position);
        Ok(vec![text_event])
    }

    fn name(&self) -> &'static str {
        "InsertTab"
    }
}

/// Delete character before cursor (Backspace in insert mode)
pub struct DeleteCharCommand;

impl Command for DeleteCharCommand {
    fn is_relevant(&self, context: &CommandContext, event: &KeyEvent) -> bool {
        matches!(event.code, KeyCode::Backspace)
            && matches!(
                context.state.current_mode,
                EditorMode::Insert | EditorMode::VisualBlockInsert
            )
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
            && matches!(
                context.state.current_mode,
                EditorMode::Insert | EditorMode::VisualBlockInsert
            )
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
                expand_tab: false,
                tab_width: 4,
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
    fn insert_tab_should_insert_tab_character_when_expandtab_off() {
        let mut context = create_test_context();
        context.state.expand_tab = false;
        context.state.tab_width = 4;
        let cmd = InsertTabCommand;
        let event = create_test_key_event(KeyCode::Tab);

        let result = cmd.execute(event, &context).unwrap();
        assert_eq!(result.len(), 1);
        if let CommandEvent::TextInsertRequested { text, .. } = &result[0] {
            assert_eq!(text, "\t");
        } else {
            panic!("Expected TextInsertRequested event");
        }
    }

    #[test]
    fn insert_tab_should_insert_spaces_when_expandtab_on() {
        let mut context = create_test_context();
        context.state.expand_tab = true;
        context.state.tab_width = 4;
        let cmd = InsertTabCommand;
        let event = create_test_key_event(KeyCode::Tab);

        let result = cmd.execute(event, &context).unwrap();
        assert_eq!(result.len(), 1);
        if let CommandEvent::TextInsertRequested { text, .. } = &result[0] {
            assert_eq!(text, "    "); // 4 spaces
        } else {
            panic!("Expected TextInsertRequested event");
        }
    }

    #[test]
    fn insert_tab_should_use_correct_tab_width() {
        let mut context = create_test_context();
        context.state.expand_tab = true;
        context.state.tab_width = 2;
        let cmd = InsertTabCommand;
        let event = create_test_key_event(KeyCode::Tab);

        let result = cmd.execute(event, &context).unwrap();
        assert_eq!(result.len(), 1);
        if let CommandEvent::TextInsertRequested { text, .. } = &result[0] {
            assert_eq!(text, "  "); // 2 spaces
        } else {
            panic!("Expected TextInsertRequested event");
        }
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

    // Tab command tests
    #[test]
    fn insert_tab_should_be_relevant_for_tab_key_in_insert_mode() {
        let context = create_test_context();
        let cmd = InsertTabCommand;
        let event = create_test_key_event(KeyCode::Tab);

        assert!(cmd.is_relevant(&context, &event));
    }

    #[test]
    fn insert_tab_should_not_be_relevant_in_normal_mode() {
        let mut context = create_test_context();
        context.state.current_mode = EditorMode::Normal;
        let cmd = InsertTabCommand;
        let event = create_test_key_event(KeyCode::Tab);

        assert!(!cmd.is_relevant(&context, &event));
    }

    #[test]
    fn insert_tab_should_not_be_relevant_with_modifiers() {
        let context = create_test_context();
        let cmd = InsertTabCommand;
        let event = KeyEvent::new(KeyCode::Tab, KeyModifiers::SHIFT);

        assert!(!cmd.is_relevant(&context, &event));
    }

    #[test]
    fn insert_tab_should_execute_tab_character_insertion() {
        let context = create_test_context();
        let cmd = InsertTabCommand;
        let event = create_test_key_event(KeyCode::Tab);

        let result = cmd.execute(event, &context).unwrap();
        assert_eq!(result.len(), 1);
        if let CommandEvent::TextInsertRequested { text, position } = &result[0] {
            assert_eq!(text, "\t");
            assert_eq!(*position, LogicalPosition { line: 0, column: 0 });
        } else {
            panic!("Expected TextInsertRequested event");
        }
    }
}
