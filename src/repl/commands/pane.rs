//! # Pane Management Commands
//!
//! Commands for switching between request and response panes

use crate::repl::events::Pane;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use super::{Command, CommandContext, CommandEvent};

/// Switch between panes (Tab key)
pub struct SwitchPaneCommand;

impl Command for SwitchPaneCommand {
    fn is_relevant(&self, _context: &CommandContext, event: &KeyEvent) -> bool {
        matches!(event.code, KeyCode::Tab)
    }

    fn execute(&self, _event: KeyEvent, context: &CommandContext) -> Result<Vec<CommandEvent>> {
        // Use semantic switching - just switch to other area
        let new_pane = match context.state.current_pane {
            Pane::Request => Pane::Response,
            Pane::Response => Pane::Request,
        };
        Ok(vec![CommandEvent::pane_switch(new_pane)])
    }

    fn name(&self) -> &'static str {
        "SwitchPane"
    }
}

// TODO: Update tests for new event-driven API
/*
#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::events::Pane;
    use crossterm::event::KeyModifiers;

    fn create_test_key_event(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::empty())
    }

    #[test]
    fn switch_pane_should_be_relevant_for_tab() {
        let vm = ViewModel::new();
        let cmd = SwitchPaneCommand;
        let event = create_test_key_event(KeyCode::Tab);

        assert!(cmd.is_relevant(&vm, &event));
    }

    #[test]
    fn switch_pane_should_toggle_between_panes() {
        let mut vm = ViewModel::new();
        let cmd = SwitchPaneCommand;
        let event = create_test_key_event(KeyCode::Tab);

        // Should start in Request pane
        assert_eq!(vm.get_current_pane(), Pane::Request);

        // Execute command to switch to Response
        cmd.execute(event, &mut vm).unwrap();
        assert_eq!(vm.get_current_pane(), Pane::Response);

        // Execute again to switch back to Request
        let event = create_test_key_event(KeyCode::Tab);
        cmd.execute(event, &mut vm).unwrap();
        assert_eq!(vm.get_current_pane(), Pane::Request);
    }
}
}
*/
