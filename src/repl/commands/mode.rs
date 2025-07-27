//! # Mode Change Commands
//!
//! Commands for switching between editor modes (Normal, Insert, Command)

use crate::repl::events::{EditorMode, Pane};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::{Command, CommandContext, CommandEvent, MovementDirection};

/// Enter insert mode (i key)
pub struct EnterInsertModeCommand;

impl Command for EnterInsertModeCommand {
    fn is_relevant(&self, context: &CommandContext, event: &KeyEvent) -> bool {
        matches!(event.code, KeyCode::Char('i'))
            && context.state.current_mode == EditorMode::Normal
            && context.state.current_pane == Pane::Request
            && event.modifiers.is_empty()
    }

    fn execute(&self, _event: KeyEvent, _context: &CommandContext) -> Result<Vec<CommandEvent>> {
        Ok(vec![CommandEvent::mode_change(EditorMode::Insert)])
    }

    fn name(&self) -> &'static str {
        "EnterInsertMode"
    }
}

/// Exit insert mode (Escape key)
pub struct ExitInsertModeCommand;

impl Command for ExitInsertModeCommand {
    fn is_relevant(&self, context: &CommandContext, event: &KeyEvent) -> bool {
        matches!(event.code, KeyCode::Esc) && context.state.current_mode == EditorMode::Insert
    }

    fn execute(&self, _event: KeyEvent, _context: &CommandContext) -> Result<Vec<CommandEvent>> {
        Ok(vec![CommandEvent::mode_change(EditorMode::Normal)])
    }

    fn name(&self) -> &'static str {
        "ExitInsertMode"
    }
}

/// Enter command mode (: key)
pub struct EnterCommandModeCommand;

impl Command for EnterCommandModeCommand {
    fn is_relevant(&self, context: &CommandContext, event: &KeyEvent) -> bool {
        matches!(event.code, KeyCode::Char(':'))
            && context.state.current_mode == EditorMode::Normal
            && event.modifiers.is_empty()
    }

    fn execute(&self, _event: KeyEvent, _context: &CommandContext) -> Result<Vec<CommandEvent>> {
        Ok(vec![CommandEvent::mode_change(EditorMode::Command)])
    }

    fn name(&self) -> &'static str {
        "EnterCommandMode"
    }
}

/// Append at end of line (Shift+A)
pub struct AppendAtEndOfLineCommand;

impl Command for AppendAtEndOfLineCommand {
    fn is_relevant(&self, context: &CommandContext, event: &KeyEvent) -> bool {
        context.state.current_mode == EditorMode::Normal
            && context.state.current_pane == Pane::Request
            && (
                // Case 1: Uppercase 'A' without modifiers
                (matches!(event.code, KeyCode::Char('A')) && event.modifiers.is_empty())
                // Case 2: Lowercase 'a' with SHIFT modifier
                || (matches!(event.code, KeyCode::Char('a')) && event.modifiers.contains(KeyModifiers::SHIFT))
                // Case 3: Uppercase 'A' with SHIFT modifier (some terminals send this)
                || (matches!(event.code, KeyCode::Char('A')) && event.modifiers.contains(KeyModifiers::SHIFT))
            )
    }

    fn execute(&self, _event: KeyEvent, _context: &CommandContext) -> Result<Vec<CommandEvent>> {
        Ok(vec![
            CommandEvent::cursor_move(MovementDirection::LineEnd),
            CommandEvent::mode_change(EditorMode::Insert),
        ])
    }

    fn name(&self) -> &'static str {
        "AppendAtEndOfLine"
    }
}

/// Insert at beginning of line (Shift+I)
pub struct InsertAtBeginningOfLineCommand;

impl Command for InsertAtBeginningOfLineCommand {
    fn is_relevant(&self, context: &CommandContext, event: &KeyEvent) -> bool {
        context.state.current_mode == EditorMode::Normal
            && context.state.current_pane == Pane::Request
            && (
                // Case 1: Uppercase 'I' without modifiers
                (matches!(event.code, KeyCode::Char('I')) && event.modifiers.is_empty())
                // Case 2: Lowercase 'i' with SHIFT modifier
                || (matches!(event.code, KeyCode::Char('i')) && event.modifiers.contains(KeyModifiers::SHIFT))
                // Case 3: Uppercase 'I' with SHIFT modifier (some terminals send this)
                || (matches!(event.code, KeyCode::Char('I')) && event.modifiers.contains(KeyModifiers::SHIFT))
            )
    }

    fn execute(&self, _event: KeyEvent, _context: &CommandContext) -> Result<Vec<CommandEvent>> {
        Ok(vec![
            CommandEvent::cursor_move(MovementDirection::LineStart),
            CommandEvent::mode_change(EditorMode::Insert),
        ])
    }

    fn name(&self) -> &'static str {
        "InsertAtBeginningOfLine"
    }
}

/// Handle all ex command mode input (typing, backspace, execute)
pub struct ExCommandModeCommand;

impl Command for ExCommandModeCommand {
    fn is_relevant(&self, context: &CommandContext, _event: &KeyEvent) -> bool {
        matches!(context.state.current_mode, EditorMode::Command)
    }

    fn execute(&self, event: KeyEvent, _context: &CommandContext) -> Result<Vec<CommandEvent>> {
        match event.code {
            KeyCode::Char(ch) if event.modifiers == KeyModifiers::NONE => {
                Ok(vec![CommandEvent::ExCommandCharRequested { ch }])
            }
            KeyCode::Backspace => Ok(vec![CommandEvent::ExCommandBackspaceRequested]),
            KeyCode::Enter => Ok(vec![CommandEvent::ExCommandExecuteRequested]),
            KeyCode::Esc => Ok(vec![CommandEvent::ModeChangeRequested {
                new_mode: EditorMode::Normal,
            }]),
            _ => Ok(vec![]),
        }
    }

    fn name(&self) -> &'static str {
        "ExCommandMode"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::commands::ViewModelSnapshot;
    use crate::repl::events::LogicalPosition;
    use crossterm::event::KeyModifiers;

    fn create_test_key_event(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::empty())
    }

    fn create_test_context() -> CommandContext {
        CommandContext {
            state: ViewModelSnapshot {
                current_mode: EditorMode::Normal,
                current_pane: Pane::Request,
                cursor_position: LogicalPosition { line: 0, column: 0 },
                request_text: String::new(),
                response_text: String::new(),
                terminal_width: 80,
                terminal_height: 24,
                verbose: false,
            },
        }
    }

    #[test]
    fn append_at_end_of_line_should_be_relevant_for_uppercase_a_in_normal_mode() {
        let context = create_test_context();
        let cmd = AppendAtEndOfLineCommand;
        let event = create_test_key_event(KeyCode::Char('A'));

        assert!(cmd.is_relevant(&context, &event));
    }

    #[test]
    fn append_at_end_of_line_should_be_relevant_for_shift_a_in_normal_mode() {
        let context = create_test_context();
        let cmd = AppendAtEndOfLineCommand;
        let event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::SHIFT);

        assert!(cmd.is_relevant(&context, &event));
    }

    #[test]
    fn append_at_end_of_line_should_be_relevant_for_uppercase_a_with_shift_in_normal_mode() {
        let context = create_test_context();
        let cmd = AppendAtEndOfLineCommand;
        let event = KeyEvent::new(KeyCode::Char('A'), KeyModifiers::SHIFT);

        assert!(cmd.is_relevant(&context, &event));
    }

    #[test]
    fn append_at_end_of_line_should_not_be_relevant_for_lowercase_a() {
        let context = create_test_context();
        let cmd = AppendAtEndOfLineCommand;
        let event = create_test_key_event(KeyCode::Char('a'));

        assert!(!cmd.is_relevant(&context, &event));
    }

    #[test]
    fn append_at_end_of_line_should_not_be_relevant_in_insert_mode() {
        let mut context = create_test_context();
        context.state.current_mode = EditorMode::Insert;
        let cmd = AppendAtEndOfLineCommand;
        let event = create_test_key_event(KeyCode::Char('A'));

        assert!(!cmd.is_relevant(&context, &event));
    }

    #[test]
    fn append_at_end_of_line_should_execute_cursor_move_and_mode_change() {
        let context = create_test_context();
        let cmd = AppendAtEndOfLineCommand;
        let event = create_test_key_event(KeyCode::Char('A'));

        let result = cmd.execute(event, &context).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(
            result[0],
            CommandEvent::cursor_move(MovementDirection::LineEnd)
        );
        assert_eq!(result[1], CommandEvent::mode_change(EditorMode::Insert));
    }

    #[test]
    fn ex_command_mode_should_be_relevant_in_command_mode() {
        let mut context = create_test_context();
        context.state.current_mode = EditorMode::Command;
        let cmd = ExCommandModeCommand;
        let event = create_test_key_event(KeyCode::Char('q'));

        assert!(cmd.is_relevant(&context, &event));
    }

    #[test]
    fn ex_command_mode_should_not_be_relevant_in_normal_mode() {
        let context = create_test_context();
        let cmd = ExCommandModeCommand;
        let event = create_test_key_event(KeyCode::Char('q'));

        assert!(!cmd.is_relevant(&context, &event));
    }

    #[test]
    fn ex_command_mode_should_handle_character_input() {
        let context = create_test_context();
        let cmd = ExCommandModeCommand;
        let event = create_test_key_event(KeyCode::Char('q'));

        let result = cmd.execute(event, &context).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], CommandEvent::ExCommandCharRequested { ch: 'q' });
    }

    #[test]
    fn ex_command_mode_should_handle_backspace() {
        let context = create_test_context();
        let cmd = ExCommandModeCommand;
        let event = create_test_key_event(KeyCode::Backspace);

        let result = cmd.execute(event, &context).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], CommandEvent::ExCommandBackspaceRequested);
    }

    #[test]
    fn ex_command_mode_should_handle_enter() {
        let context = create_test_context();
        let cmd = ExCommandModeCommand;
        let event = create_test_key_event(KeyCode::Enter);

        let result = cmd.execute(event, &context).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], CommandEvent::ExCommandExecuteRequested);
    }

    #[test]
    fn ex_command_mode_should_handle_escape() {
        let context = create_test_context();
        let cmd = ExCommandModeCommand;
        let event = create_test_key_event(KeyCode::Esc);

        let result = cmd.execute(event, &context).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0],
            CommandEvent::ModeChangeRequested {
                new_mode: EditorMode::Normal
            }
        );
    }

    #[test]
    fn insert_at_beginning_of_line_should_be_relevant_for_uppercase_i_in_normal_mode() {
        let context = create_test_context();
        let cmd = InsertAtBeginningOfLineCommand;
        let event = create_test_key_event(KeyCode::Char('I'));

        assert!(cmd.is_relevant(&context, &event));
    }

    #[test]
    fn insert_at_beginning_of_line_should_be_relevant_for_shift_i_in_normal_mode() {
        let context = create_test_context();
        let cmd = InsertAtBeginningOfLineCommand;
        let event = KeyEvent::new(KeyCode::Char('i'), KeyModifiers::SHIFT);

        assert!(cmd.is_relevant(&context, &event));
    }

    #[test]
    fn insert_at_beginning_of_line_should_be_relevant_for_uppercase_i_with_shift_in_normal_mode() {
        let context = create_test_context();
        let cmd = InsertAtBeginningOfLineCommand;
        let event = KeyEvent::new(KeyCode::Char('I'), KeyModifiers::SHIFT);

        assert!(cmd.is_relevant(&context, &event));
    }

    #[test]
    fn insert_at_beginning_of_line_should_not_be_relevant_for_lowercase_i() {
        let context = create_test_context();
        let cmd = InsertAtBeginningOfLineCommand;
        let event = create_test_key_event(KeyCode::Char('i'));

        assert!(!cmd.is_relevant(&context, &event));
    }

    #[test]
    fn insert_at_beginning_of_line_should_not_be_relevant_in_insert_mode() {
        let mut context = create_test_context();
        context.state.current_mode = EditorMode::Insert;
        let cmd = InsertAtBeginningOfLineCommand;
        let event = create_test_key_event(KeyCode::Char('I'));

        assert!(!cmd.is_relevant(&context, &event));
    }

    #[test]
    fn insert_at_beginning_of_line_should_execute_cursor_move_and_mode_change() {
        let context = create_test_context();
        let cmd = InsertAtBeginningOfLineCommand;
        let event = create_test_key_event(KeyCode::Char('I'));

        let result = cmd.execute(event, &context).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(
            result[0],
            CommandEvent::cursor_move(MovementDirection::LineStart)
        );
        assert_eq!(result[1], CommandEvent::mode_change(EditorMode::Insert));
    }
}
