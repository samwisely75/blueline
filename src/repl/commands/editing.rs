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
                    && (ch.is_ascii_graphic() || ch == ' ')
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
        let delete_event = CommandEvent::TextDeleteRequested {
            position: context.state.cursor_position,
            amount: 1,
            direction: MovementDirection::Left,
        };
        Ok(vec![delete_event])
    }

    fn name(&self) -> &'static str {
        "DeleteChar"
    }
}

// TODO: Update tests for new event-driven API
/*
#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::events::EditorMode;
    use crossterm::event::KeyModifiers;

    fn create_test_key_event(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::empty())
    }

    #[test]
    fn insert_char_should_be_relevant_for_printable_chars_in_insert_mode() {
        let mut vm = ViewModel::new();
        vm.change_mode(EditorMode::Insert).unwrap();
        let cmd = InsertCharCommand;
        let event = create_test_key_event(KeyCode::Char('a'));

        assert!(cmd.is_relevant(&vm, &event));
    }

    #[test]
    fn insert_char_should_not_be_relevant_in_normal_mode() {
        let vm = ViewModel::new(); // Starts in Normal mode
        let cmd = InsertCharCommand;
        let event = create_test_key_event(KeyCode::Char('a'));

        assert!(!cmd.is_relevant(&vm, &event));
    }

    #[test]
    fn insert_newline_should_be_relevant_for_enter_in_insert_mode() {
        let mut vm = ViewModel::new();
        vm.change_mode(EditorMode::Insert).unwrap();
        let cmd = InsertNewLineCommand;
        let event = create_test_key_event(KeyCode::Enter);

        assert!(cmd.is_relevant(&vm, &event));
    }

    #[test]
    fn delete_char_should_be_relevant_for_backspace_in_insert_mode() {
        let mut vm = ViewModel::new();
        vm.change_mode(EditorMode::Insert).unwrap();
        let cmd = DeleteCharCommand;
        let event = create_test_key_event(KeyCode::Backspace);

        assert!(cmd.is_relevant(&vm, &event));
    }
}
*/
