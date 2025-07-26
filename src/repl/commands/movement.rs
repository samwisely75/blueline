//! # Movement Commands
//!
//! Commands for cursor movement including basic h,j,k,l navigation
//! and arrow key support for all modes.

use crate::repl::events::EditorMode;
use crate::repl::view_models::ViewModel;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use super::Command;

/// Move cursor left (h key or left arrow)
pub struct MoveCursorLeftCommand;

impl Command for MoveCursorLeftCommand {
    fn is_relevant(&self, view_model: &ViewModel, event: &KeyEvent) -> bool {
        match event.code {
            KeyCode::Char('h') => {
                view_model.get_mode() == EditorMode::Normal && event.modifiers.is_empty()
            }
            KeyCode::Left => true,
            _ => false,
        }
    }

    fn execute(&self, _event: KeyEvent, view_model: &mut ViewModel) -> Result<bool> {
        view_model.move_cursor_left()?;
        Ok(true)
    }

    fn name(&self) -> &'static str {
        "MoveCursorLeft"
    }
}

/// Move cursor right (l key or right arrow)
pub struct MoveCursorRightCommand;

impl Command for MoveCursorRightCommand {
    fn is_relevant(&self, view_model: &ViewModel, event: &KeyEvent) -> bool {
        match event.code {
            KeyCode::Char('l') => {
                view_model.get_mode() == EditorMode::Normal && event.modifiers.is_empty()
            }
            KeyCode::Right => true,
            _ => false,
        }
    }

    fn execute(&self, _event: KeyEvent, view_model: &mut ViewModel) -> Result<bool> {
        view_model.move_cursor_right()?;
        Ok(true)
    }

    fn name(&self) -> &'static str {
        "MoveCursorRight"
    }
}

/// Move cursor up (k key or up arrow)
pub struct MoveCursorUpCommand;

impl Command for MoveCursorUpCommand {
    fn is_relevant(&self, view_model: &ViewModel, event: &KeyEvent) -> bool {
        match event.code {
            KeyCode::Char('k') => {
                view_model.get_mode() == EditorMode::Normal && event.modifiers.is_empty()
            }
            KeyCode::Up => true,
            _ => false,
        }
    }

    fn execute(&self, _event: KeyEvent, view_model: &mut ViewModel) -> Result<bool> {
        view_model.move_cursor_up()?;
        Ok(true)
    }

    fn name(&self) -> &'static str {
        "MoveCursorUp"
    }
}

/// Move cursor down (j key or down arrow)
pub struct MoveCursorDownCommand;

impl Command for MoveCursorDownCommand {
    fn is_relevant(&self, view_model: &ViewModel, event: &KeyEvent) -> bool {
        match event.code {
            KeyCode::Char('j') => {
                view_model.get_mode() == EditorMode::Normal && event.modifiers.is_empty()
            }
            KeyCode::Down => true,
            _ => false,
        }
    }

    fn execute(&self, _event: KeyEvent, view_model: &mut ViewModel) -> Result<bool> {
        view_model.move_cursor_down()?;
        Ok(true)
    }

    fn name(&self) -> &'static str {
        "MoveCursorDown"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::events::EditorMode;
    use crossterm::event::KeyModifiers;

    fn create_test_key_event(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::empty())
    }

    #[test]
    fn move_cursor_left_should_be_relevant_for_h_in_normal_mode() {
        let vm = ViewModel::new();
        let cmd = MoveCursorLeftCommand;
        let event = create_test_key_event(KeyCode::Char('h'));

        assert!(cmd.is_relevant(&vm, &event));
    }

    #[test]
    fn move_cursor_left_should_be_relevant_for_left_arrow() {
        let mut vm = ViewModel::new();
        vm.change_mode(EditorMode::Insert).unwrap();
        let cmd = MoveCursorLeftCommand;
        let event = create_test_key_event(KeyCode::Left);

        assert!(cmd.is_relevant(&vm, &event));
    }

    #[test]
    fn move_cursor_left_should_not_be_relevant_for_h_in_insert_mode() {
        let mut vm = ViewModel::new();
        vm.change_mode(EditorMode::Insert).unwrap();
        let cmd = MoveCursorLeftCommand;
        let event = create_test_key_event(KeyCode::Char('h'));

        assert!(!cmd.is_relevant(&vm, &event));
    }
}
