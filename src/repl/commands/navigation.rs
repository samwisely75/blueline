//! # Movement Commands
//!
//! Commands for cursor movement including basic h,j,k,l navigation
//! and arrow key support for all modes.

use crate::repl::events::EditorMode;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::{Command, CommandContext, CommandEvent, MovementDirection};

/// Move cursor left (h key or left arrow)
pub struct MoveCursorLeftCommand;

impl Command for MoveCursorLeftCommand {
    fn is_relevant(&self, context: &CommandContext, event: &KeyEvent) -> bool {
        match event.code {
            KeyCode::Char('h') => {
                context.state.current_mode == EditorMode::Normal && event.modifiers.is_empty()
            }
            KeyCode::Left => {
                !event.modifiers.contains(KeyModifiers::SHIFT)
                    && !event.modifiers.contains(KeyModifiers::CONTROL)
            }
            _ => false,
        }
    }

    fn execute(&self, _event: KeyEvent, _context: &CommandContext) -> Result<Vec<CommandEvent>> {
        Ok(vec![CommandEvent::cursor_move(MovementDirection::Left)])
    }

    fn name(&self) -> &'static str {
        "MoveCursorLeft"
    }
}

/// Move cursor right (l key or right arrow)
pub struct MoveCursorRightCommand;

impl Command for MoveCursorRightCommand {
    fn is_relevant(&self, context: &CommandContext, event: &KeyEvent) -> bool {
        match event.code {
            KeyCode::Char('l') => {
                context.state.current_mode == EditorMode::Normal && event.modifiers.is_empty()
            }
            KeyCode::Right => {
                !event.modifiers.contains(KeyModifiers::SHIFT)
                    && !event.modifiers.contains(KeyModifiers::CONTROL)
            }
            _ => false,
        }
    }

    fn execute(&self, _event: KeyEvent, _context: &CommandContext) -> Result<Vec<CommandEvent>> {
        Ok(vec![CommandEvent::cursor_move(MovementDirection::Right)])
    }

    fn name(&self) -> &'static str {
        "MoveCursorRight"
    }
}

/// Move cursor up (k key or up arrow)
pub struct MoveCursorUpCommand;

impl Command for MoveCursorUpCommand {
    fn is_relevant(&self, context: &CommandContext, event: &KeyEvent) -> bool {
        match event.code {
            KeyCode::Char('k') => {
                context.state.current_mode == EditorMode::Normal && event.modifiers.is_empty()
            }
            KeyCode::Up => true,
            _ => false,
        }
    }

    fn execute(&self, _event: KeyEvent, _context: &CommandContext) -> Result<Vec<CommandEvent>> {
        Ok(vec![CommandEvent::cursor_move(MovementDirection::Up)])
    }

    fn name(&self) -> &'static str {
        "MoveCursorUp"
    }
}

/// Move cursor down (j key or down arrow)
pub struct MoveCursorDownCommand;

impl Command for MoveCursorDownCommand {
    fn is_relevant(&self, context: &CommandContext, event: &KeyEvent) -> bool {
        match event.code {
            KeyCode::Char('j') => {
                context.state.current_mode == EditorMode::Normal && event.modifiers.is_empty()
            }
            KeyCode::Down => true,
            _ => false,
        }
    }

    fn execute(&self, _event: KeyEvent, _context: &CommandContext) -> Result<Vec<CommandEvent>> {
        Ok(vec![CommandEvent::cursor_move(MovementDirection::Down)])
    }

    fn name(&self) -> &'static str {
        "MoveCursorDown"
    }
}

/// Scroll left horizontally (Shift+Left or Ctrl+Left)
pub struct ScrollLeftCommand;

impl Command for ScrollLeftCommand {
    fn is_relevant(&self, _context: &CommandContext, event: &KeyEvent) -> bool {
        let relevant = matches!(event.code, KeyCode::Left)
            && (event.modifiers.contains(KeyModifiers::SHIFT)
                || event.modifiers.contains(KeyModifiers::CONTROL));
        if relevant {
            tracing::debug!("ScrollLeftCommand is relevant for event: {:?}", event);
        }
        relevant
    }

    fn execute(&self, _event: KeyEvent, _context: &CommandContext) -> Result<Vec<CommandEvent>> {
        Ok(vec![CommandEvent::cursor_move_by(
            MovementDirection::ScrollLeft,
            5,
        )])
    }

    fn name(&self) -> &'static str {
        "ScrollLeft"
    }
}

/// Scroll right horizontally (Shift+Right or Ctrl+Right)
pub struct ScrollRightCommand;

impl Command for ScrollRightCommand {
    fn is_relevant(&self, _context: &CommandContext, event: &KeyEvent) -> bool {
        let relevant = matches!(event.code, KeyCode::Right)
            && (event.modifiers.contains(KeyModifiers::SHIFT)
                || event.modifiers.contains(KeyModifiers::CONTROL));
        if relevant {
            tracing::debug!("ScrollRightCommand is relevant for event: {:?}", event);
        }
        relevant
    }

    fn execute(&self, _event: KeyEvent, _context: &CommandContext) -> Result<Vec<CommandEvent>> {
        Ok(vec![CommandEvent::cursor_move_by(
            MovementDirection::ScrollRight,
            5,
        )])
    }

    fn name(&self) -> &'static str {
        "ScrollRight"
    }
}

/// Enter G micro mode on first 'g' press
pub struct EnterGModeCommand;

impl Command for EnterGModeCommand {
    fn is_relevant(&self, context: &CommandContext, event: &KeyEvent) -> bool {
        matches!(event.code, KeyCode::Char('g'))
            && context.state.current_mode == EditorMode::Normal
            && event.modifiers.is_empty()
    }

    fn execute(&self, _event: KeyEvent, _context: &CommandContext) -> Result<Vec<CommandEvent>> {
        Ok(vec![CommandEvent::mode_change(EditorMode::GMode)])
    }

    fn name(&self) -> &'static str {
        "EnterGMode"
    }
}

/// Go to top of current pane (gg command)
pub struct GoToTopCommand;

impl Command for GoToTopCommand {
    fn is_relevant(&self, context: &CommandContext, event: &KeyEvent) -> bool {
        matches!(event.code, KeyCode::Char('g'))
            && context.state.current_mode == EditorMode::GMode
            && event.modifiers.is_empty()
    }

    fn execute(&self, _event: KeyEvent, _context: &CommandContext) -> Result<Vec<CommandEvent>> {
        Ok(vec![
            CommandEvent::cursor_move(MovementDirection::DocumentStart),
            CommandEvent::mode_change(EditorMode::Normal),
        ])
    }

    fn name(&self) -> &'static str {
        "GoToTop"
    }
}

/// Go to bottom of current pane (G command)
pub struct GoToBottomCommand;

impl Command for GoToBottomCommand {
    fn is_relevant(&self, context: &CommandContext, event: &KeyEvent) -> bool {
        matches!(event.code, KeyCode::Char('G'))
            && context.state.current_mode == EditorMode::Normal
            && event.modifiers.is_empty()
    }

    fn execute(&self, _event: KeyEvent, _context: &CommandContext) -> Result<Vec<CommandEvent>> {
        Ok(vec![CommandEvent::cursor_move(
            MovementDirection::DocumentEnd,
        )])
    }

    fn name(&self) -> &'static str {
        "GoToBottom"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::commands::context::ViewModelSnapshot;
    use crate::repl::events::{EditorMode, LogicalPosition, Pane};
    use crossterm::event::KeyModifiers;

    fn create_test_key_event(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::empty())
    }

    fn create_test_context(mode: EditorMode) -> CommandContext {
        let snapshot = ViewModelSnapshot {
            current_mode: mode,
            current_pane: Pane::Request,
            cursor_position: LogicalPosition::zero(),
            request_text: String::new(),
            response_text: String::new(),
            terminal_width: 80,
            terminal_height: 24,
            verbose: false,
        };
        CommandContext::new(snapshot)
    }

    // Tests for G mode commands
    #[test]
    fn enter_g_mode_should_be_relevant_for_g_in_normal_mode() {
        let context = create_test_context(EditorMode::Normal);
        let cmd = EnterGModeCommand;
        let event = create_test_key_event(KeyCode::Char('g'));

        assert!(cmd.is_relevant(&context, &event));
    }

    #[test]
    fn enter_g_mode_should_not_be_relevant_in_insert_mode() {
        let context = create_test_context(EditorMode::Insert);
        let cmd = EnterGModeCommand;
        let event = create_test_key_event(KeyCode::Char('g'));

        assert!(!cmd.is_relevant(&context, &event));
    }

    #[test]
    fn enter_g_mode_should_not_be_relevant_in_g_mode() {
        let context = create_test_context(EditorMode::GMode);
        let cmd = EnterGModeCommand;
        let event = create_test_key_event(KeyCode::Char('g'));

        assert!(!cmd.is_relevant(&context, &event));
    }

    #[test]
    fn enter_g_mode_should_produce_mode_change_event() {
        let context = create_test_context(EditorMode::Normal);
        let cmd = EnterGModeCommand;
        let event = create_test_key_event(KeyCode::Char('g'));

        let events = cmd.execute(event, &context).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0], CommandEvent::mode_change(EditorMode::GMode));
    }

    #[test]
    fn go_to_top_should_be_relevant_for_g_in_g_mode() {
        let context = create_test_context(EditorMode::GMode);
        let cmd = GoToTopCommand;
        let event = create_test_key_event(KeyCode::Char('g'));

        assert!(cmd.is_relevant(&context, &event));
    }

    #[test]
    fn go_to_top_should_not_be_relevant_in_normal_mode() {
        let context = create_test_context(EditorMode::Normal);
        let cmd = GoToTopCommand;
        let event = create_test_key_event(KeyCode::Char('g'));

        assert!(!cmd.is_relevant(&context, &event));
    }

    #[test]
    fn go_to_top_should_not_be_relevant_in_insert_mode() {
        let context = create_test_context(EditorMode::Insert);
        let cmd = GoToTopCommand;
        let event = create_test_key_event(KeyCode::Char('g'));

        assert!(!cmd.is_relevant(&context, &event));
    }

    #[test]
    fn go_to_top_should_produce_document_start_and_normal_mode_events() {
        let context = create_test_context(EditorMode::GMode);
        let cmd = GoToTopCommand;
        let event = create_test_key_event(KeyCode::Char('g'));

        let events = cmd.execute(event, &context).unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(
            events[0],
            CommandEvent::cursor_move(MovementDirection::DocumentStart)
        );
        assert_eq!(events[1], CommandEvent::mode_change(EditorMode::Normal));
    }

    #[test]
    fn move_cursor_left_should_be_relevant_for_h_in_normal_mode() {
        let context = create_test_context(EditorMode::Normal);
        let cmd = MoveCursorLeftCommand;
        let event = create_test_key_event(KeyCode::Char('h'));

        assert!(cmd.is_relevant(&context, &event));
    }

    #[test]
    fn go_to_bottom_should_be_relevant_for_g_in_normal_mode() {
        let context = create_test_context(EditorMode::Normal);
        let cmd = GoToBottomCommand;
        let event = create_test_key_event(KeyCode::Char('G'));

        assert!(cmd.is_relevant(&context, &event));
    }

    #[test]
    fn go_to_bottom_should_not_be_relevant_in_insert_mode() {
        let context = create_test_context(EditorMode::Insert);
        let cmd = GoToBottomCommand;
        let event = create_test_key_event(KeyCode::Char('G'));

        assert!(!cmd.is_relevant(&context, &event));
    }

    #[test]
    fn go_to_bottom_should_not_be_relevant_in_g_mode() {
        let context = create_test_context(EditorMode::GMode);
        let cmd = GoToBottomCommand;
        let event = create_test_key_event(KeyCode::Char('G'));

        assert!(!cmd.is_relevant(&context, &event));
    }

    #[test]
    fn go_to_bottom_should_not_be_relevant_for_lowercase_g() {
        let context = create_test_context(EditorMode::Normal);
        let cmd = GoToBottomCommand;
        let event = create_test_key_event(KeyCode::Char('g'));

        assert!(!cmd.is_relevant(&context, &event));
    }

    #[test]
    fn go_to_bottom_should_produce_document_end_event() {
        let context = create_test_context(EditorMode::Normal);
        let cmd = GoToBottomCommand;
        let event = create_test_key_event(KeyCode::Char('G'));

        let events = cmd.execute(event, &context).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0],
            CommandEvent::cursor_move(MovementDirection::DocumentEnd)
        );
    }
}
