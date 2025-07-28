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

// TODO: Update tests for new event-driven API
/*
#[cfg(test)]
mod tests {
    use super::*;
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

    #[test]
    fn move_cursor_left_should_be_relevant_for_h_in_normal_mode() {
        let context = create_test_context(EditorMode::Normal);
        let cmd = MoveCursorLeftCommand;
        let event = create_test_key_event(KeyCode::Char('h'));

        assert!(cmd.is_relevant(&context, &event));
    }

    #[test]
    fn move_cursor_left_should_be_relevant_for_left_arrow() {
        let context = create_test_context(EditorMode::Insert);
        let cmd = MoveCursorLeftCommand;
        let event = create_test_key_event(KeyCode::Left);

        assert!(cmd.is_relevant(&context, &event));
    }

    #[test]
    fn move_cursor_left_should_not_be_relevant_for_h_in_insert_mode() {
        let context = create_test_context(EditorMode::Insert);
        let cmd = MoveCursorLeftCommand;
        let event = create_test_key_event(KeyCode::Char('h'));

        assert!(!cmd.is_relevant(&context, &event));
    }

    #[test]
    fn move_cursor_left_should_produce_movement_event() {
        let context = create_test_context(EditorMode::Normal);
        let cmd = MoveCursorLeftCommand;
        let event = create_test_key_event(KeyCode::Char('h'));

        let events = cmd.execute(event, &context).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0],
            CommandEvent::CursorMoveRequested {
                direction: MovementDirection::Left,
                amount: 1
            }
        );
    }
}
}
*/
