//! # Pane Management Commands
//!
//! Commands for switching between request and response panes

use crate::repl::events::{EditorMode, Pane};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use super::{Command, CommandContext, CommandEvent};

/// Switch between panes (Tab key)
pub struct SwitchPaneCommand;

impl Command for SwitchPaneCommand {
    fn is_relevant(&self, context: &CommandContext, event: &KeyEvent) -> bool {
        let relevant = matches!(event.code, KeyCode::Tab)
            && context.state.current_mode == EditorMode::Normal
            && event.modifiers.is_empty();
        if matches!(event.code, KeyCode::Tab) {
            tracing::debug!(
                "SwitchPaneCommand: Tab key pressed, mode={:?}, relevant={}",
                context.state.current_mode,
                relevant
            );
        }
        relevant
    }

    fn execute(&self, _event: KeyEvent, context: &CommandContext) -> Result<Vec<CommandEvent>> {
        // Use semantic switching - just switch to other area
        let new_pane = match context.state.current_pane {
            Pane::Request => Pane::Response,
            Pane::Response => Pane::Request,
        };
        tracing::debug!(
            "SwitchPaneCommand: switching from {:?} to {:?}",
            context.state.current_pane,
            new_pane
        );
        Ok(vec![CommandEvent::pane_switch(new_pane)])
    }

    fn name(&self) -> &'static str {
        "SwitchPane"
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
                terminal_dimensions: (80, 24),
                expand_tab: false,
                tab_width: 4,
            },
        }
    }

    #[test]
    fn switch_pane_should_be_relevant_for_tab_in_normal_mode() {
        let context = create_test_context();
        let cmd = SwitchPaneCommand;
        let event = create_test_key_event(KeyCode::Tab);

        assert!(cmd.is_relevant(&context, &event));
    }

    #[test]
    fn switch_pane_should_not_be_relevant_for_tab_in_insert_mode() {
        let mut context = create_test_context();
        context.state.current_mode = EditorMode::Insert;
        let cmd = SwitchPaneCommand;
        let event = create_test_key_event(KeyCode::Tab);

        assert!(!cmd.is_relevant(&context, &event));
    }

    #[test]
    fn switch_pane_should_not_be_relevant_for_tab_in_visual_mode() {
        let mut context = create_test_context();
        context.state.current_mode = EditorMode::Visual;
        let cmd = SwitchPaneCommand;
        let event = create_test_key_event(KeyCode::Tab);

        assert!(!cmd.is_relevant(&context, &event));
    }

    #[test]
    fn switch_pane_should_not_be_relevant_for_tab_in_command_mode() {
        let mut context = create_test_context();
        context.state.current_mode = EditorMode::Command;
        let cmd = SwitchPaneCommand;
        let event = create_test_key_event(KeyCode::Tab);

        assert!(!cmd.is_relevant(&context, &event));
    }

    #[test]
    fn switch_pane_should_not_be_relevant_for_other_keys() {
        let context = create_test_context();
        let cmd = SwitchPaneCommand;
        let event = create_test_key_event(KeyCode::Char('a'));

        assert!(!cmd.is_relevant(&context, &event));
    }

    #[test]
    fn switch_pane_should_not_be_relevant_for_tab_with_modifiers() {
        let context = create_test_context();
        let cmd = SwitchPaneCommand;
        let event = KeyEvent::new(KeyCode::Tab, KeyModifiers::SHIFT);

        assert!(!cmd.is_relevant(&context, &event));
    }

    #[test]
    fn switch_pane_should_produce_pane_switch_event_from_request() {
        let context = create_test_context(); // Defaults to Request pane
        let cmd = SwitchPaneCommand;
        let event = create_test_key_event(KeyCode::Tab);

        let result = cmd.execute(event, &context).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], CommandEvent::pane_switch(Pane::Response));
    }

    #[test]
    fn switch_pane_should_produce_pane_switch_event_from_response() {
        let mut context = create_test_context();
        context.state.current_pane = Pane::Response;
        let cmd = SwitchPaneCommand;
        let event = create_test_key_event(KeyCode::Tab);

        let result = cmd.execute(event, &context).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], CommandEvent::pane_switch(Pane::Request));
    }

    #[test]
    fn switch_pane_should_return_correct_command_name() {
        let cmd = SwitchPaneCommand;
        assert_eq!(cmd.name(), "SwitchPane");
    }
}
