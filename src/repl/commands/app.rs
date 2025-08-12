//! # Application Control Commands
//!
//! Commands for controlling the application lifecycle such as quit/terminate operations.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::{Command, CommandContext, CommandEvent};

/// Terminate application (Ctrl+C)
pub struct AppTerminateCommand;

impl Command for AppTerminateCommand {
    fn is_relevant(&self, _context: &CommandContext, event: &KeyEvent) -> bool {
        // Ctrl+C to quit
        matches!(event.code, KeyCode::Char('c')) && event.modifiers.contains(KeyModifiers::CONTROL)
    }

    fn execute(&self, _event: KeyEvent, _context: &CommandContext) -> Result<Vec<CommandEvent>> {
        Ok(vec![CommandEvent::QuitRequested])
    }

    fn name(&self) -> &'static str {
        "AppTerminate"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::commands::ViewModelSnapshot;
    use crate::repl::events::{EditorMode, LogicalPosition, Pane};

    fn create_test_key_event(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent::new(code, modifiers)
    }

    fn create_test_context() -> CommandContext {
        let snapshot = ViewModelSnapshot {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            cursor_position: LogicalPosition::zero(),
            request_text: String::new(),
            response_text: String::new(),
            terminal_dimensions: (80, 24),
        };
        CommandContext::new(snapshot)
    }

    #[test]
    fn app_terminate_should_be_relevant_for_ctrl_c() {
        let context = create_test_context();
        let cmd = AppTerminateCommand;
        let event = create_test_key_event(KeyCode::Char('c'), KeyModifiers::CONTROL);

        assert!(cmd.is_relevant(&context, &event));
    }

    #[test]
    fn app_terminate_should_not_be_relevant_for_regular_c() {
        let context = create_test_context();
        let cmd = AppTerminateCommand;
        let event = create_test_key_event(KeyCode::Char('c'), KeyModifiers::NONE);

        assert!(!cmd.is_relevant(&context, &event));
    }

    #[test]
    fn app_terminate_should_not_be_relevant_for_other_ctrl_keys() {
        let context = create_test_context();
        let cmd = AppTerminateCommand;
        let event = create_test_key_event(KeyCode::Char('x'), KeyModifiers::CONTROL);

        assert!(!cmd.is_relevant(&context, &event));
    }

    #[test]
    fn app_terminate_should_produce_quit_event() {
        let context = create_test_context();
        let cmd = AppTerminateCommand;
        let event = create_test_key_event(KeyCode::Char('c'), KeyModifiers::CONTROL);

        let events = cmd.execute(event, &context).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0], CommandEvent::QuitRequested);
    }
}
