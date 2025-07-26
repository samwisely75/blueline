//! # HTTP Request Commands
//!
//! Commands for executing HTTP requests and related operations

use crate::repl::events::{EditorMode, Pane};
use crate::repl::view_models::ViewModel;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use super::Command;

/// Execute HTTP request (Enter in normal mode)
pub struct ExecuteRequestCommand;

impl Command for ExecuteRequestCommand {
    fn is_relevant(&self, view_model: &ViewModel, event: &KeyEvent) -> bool {
        matches!(event.code, KeyCode::Enter)
            && view_model.get_mode() == EditorMode::Normal
            && view_model.get_current_pane() == Pane::Request
    }

    fn execute(&self, _event: KeyEvent, view_model: &mut ViewModel) -> Result<bool> {
        view_model.execute_request()?;
        Ok(true)
    }

    fn name(&self) -> &'static str {
        "ExecuteRequest"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::events::{EditorMode, Pane};
    use crossterm::event::KeyModifiers;

    fn create_test_key_event(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::empty())
    }

    #[test]
    fn execute_request_should_be_relevant_for_enter_in_normal_mode() {
        let vm = ViewModel::new(); // Starts in Normal mode, Request pane
        let cmd = ExecuteRequestCommand;
        let event = create_test_key_event(KeyCode::Enter);

        assert!(cmd.is_relevant(&vm, &event));
    }

    #[test]
    fn execute_request_should_not_be_relevant_in_insert_mode() {
        let mut vm = ViewModel::new();
        vm.change_mode(EditorMode::Insert).unwrap();
        let cmd = ExecuteRequestCommand;
        let event = create_test_key_event(KeyCode::Enter);

        assert!(!cmd.is_relevant(&vm, &event));
    }

    #[test]
    fn execute_request_should_not_be_relevant_in_response_pane() {
        let mut vm = ViewModel::new();
        vm.switch_pane(Pane::Response).unwrap();
        let cmd = ExecuteRequestCommand;
        let event = create_test_key_event(KeyCode::Enter);

        assert!(!cmd.is_relevant(&vm, &event));
    }
}
