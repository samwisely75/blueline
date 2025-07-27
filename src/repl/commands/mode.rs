//! # Mode Change Commands
//!
//! Commands for switching between editor modes (Normal, Insert, Command)

use crate::repl::events::{EditorMode, Pane};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use super::{Command, CommandContext, CommandEvent};

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
    fn enter_insert_mode_should_be_relevant_for_i_in_normal_mode() {
        let vm = ViewModel::new(); // Starts in Normal mode, Request pane
        let cmd = EnterInsertModeCommand;
        let event = create_test_key_event(KeyCode::Char('i'));

        assert!(cmd.is_relevant(&vm, &event));
    }

    #[test]
    fn enter_insert_mode_should_not_be_relevant_in_insert_mode() {
        let mut vm = ViewModel::new();
        vm.change_mode(EditorMode::Insert).unwrap();
        let cmd = EnterInsertModeCommand;
        let event = create_test_key_event(KeyCode::Char('i'));

        assert!(!cmd.is_relevant(&vm, &event));
    }

    #[test]
    fn exit_insert_mode_should_be_relevant_for_esc_in_insert_mode() {
        let mut vm = ViewModel::new();
        vm.change_mode(EditorMode::Insert).unwrap();
        let cmd = ExitInsertModeCommand;
        let event = create_test_key_event(KeyCode::Esc);

        assert!(cmd.is_relevant(&vm, &event));
    }

    #[test]
    fn exit_insert_mode_should_not_be_relevant_in_normal_mode() {
        let vm = ViewModel::new(); // Starts in Normal mode
        let cmd = ExitInsertModeCommand;
        let event = create_test_key_event(KeyCode::Esc);

        assert!(!cmd.is_relevant(&vm, &event));
    }

    #[test]
    fn enter_command_mode_should_be_relevant_for_colon_in_normal_mode() {
        let vm = ViewModel::new(); // Starts in Normal mode
        let cmd = EnterCommandModeCommand;
        let event = create_test_key_event(KeyCode::Char(':'));

        assert!(cmd.is_relevant(&vm, &event));
    }
}
}
*/
