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

/// Scroll down one page (Ctrl+f)
pub struct ScrollPageDownCommand;

impl Command for ScrollPageDownCommand {
    fn is_relevant(&self, context: &CommandContext, event: &KeyEvent) -> bool {
        matches!(event.code, KeyCode::Char('f'))
            && event.modifiers.contains(KeyModifiers::CONTROL)
            && context.state.current_mode == EditorMode::Normal
    }

    fn execute(&self, _event: KeyEvent, _context: &CommandContext) -> Result<Vec<CommandEvent>> {
        Ok(vec![CommandEvent::cursor_move(MovementDirection::PageDown)])
    }

    fn name(&self) -> &'static str {
        "ScrollPageDown"
    }
}

/// Enter G prefix mode on first 'g' press
pub struct EnterGPrefixCommand;

impl Command for EnterGPrefixCommand {
    fn is_relevant(&self, context: &CommandContext, event: &KeyEvent) -> bool {
        matches!(event.code, KeyCode::Char('g'))
            && context.state.current_mode == EditorMode::Normal
            && event.modifiers.is_empty()
    }

    fn execute(&self, _event: KeyEvent, _context: &CommandContext) -> Result<Vec<CommandEvent>> {
        Ok(vec![CommandEvent::mode_change(EditorMode::GPrefix)])
    }

    fn name(&self) -> &'static str {
        "EnterGPrefix"
    }
}

/// Go to top of current pane (gg command)
pub struct GoToTopCommand;

impl Command for GoToTopCommand {
    fn is_relevant(&self, context: &CommandContext, event: &KeyEvent) -> bool {
        matches!(event.code, KeyCode::Char('g'))
            && context.state.current_mode == EditorMode::GPrefix
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
        context.state.current_mode == EditorMode::Normal
            && (
                // Case 1: Uppercase 'G' without modifiers
                (matches!(event.code, KeyCode::Char('G')) && event.modifiers.is_empty())
                // BUGFIX: Handle Shift+g key combination across different terminals
                // Without these cases, G command wouldn't respond to user input in manual testing
                // Different terminals send different key combinations for Shift+g:
                // Case 2: Some terminals send lowercase 'g' with SHIFT modifier
                || (matches!(event.code, KeyCode::Char('g')) && event.modifiers.contains(KeyModifiers::SHIFT))
                // Case 3: Other terminals send uppercase 'G' with SHIFT modifier
                || (matches!(event.code, KeyCode::Char('G')) && event.modifiers.contains(KeyModifiers::SHIFT))
            )
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
        let cmd = EnterGPrefixCommand;
        let event = create_test_key_event(KeyCode::Char('g'));

        assert!(cmd.is_relevant(&context, &event));
    }

    #[test]
    fn enter_g_mode_should_not_be_relevant_in_insert_mode() {
        let context = create_test_context(EditorMode::Insert);
        let cmd = EnterGPrefixCommand;
        let event = create_test_key_event(KeyCode::Char('g'));

        assert!(!cmd.is_relevant(&context, &event));
    }

    #[test]
    fn enter_g_mode_should_not_be_relevant_in_g_mode() {
        let context = create_test_context(EditorMode::GPrefix);
        let cmd = EnterGPrefixCommand;
        let event = create_test_key_event(KeyCode::Char('g'));

        assert!(!cmd.is_relevant(&context, &event));
    }

    #[test]
    fn enter_g_mode_should_produce_mode_change_event() {
        let context = create_test_context(EditorMode::Normal);
        let cmd = EnterGPrefixCommand;
        let event = create_test_key_event(KeyCode::Char('g'));

        let events = cmd.execute(event, &context).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0], CommandEvent::mode_change(EditorMode::GPrefix));
    }

    #[test]
    fn go_to_top_should_be_relevant_for_g_in_g_mode() {
        let context = create_test_context(EditorMode::GPrefix);
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
        let context = create_test_context(EditorMode::GPrefix);
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
    fn go_to_bottom_should_be_relevant_for_uppercase_g_in_normal_mode() {
        let context = create_test_context(EditorMode::Normal);
        let cmd = GoToBottomCommand;
        let event = create_test_key_event(KeyCode::Char('G'));

        assert!(cmd.is_relevant(&context, &event));
    }

    #[test]
    fn go_to_bottom_should_be_relevant_for_shift_g_in_normal_mode() {
        let context = create_test_context(EditorMode::Normal);
        let cmd = GoToBottomCommand;
        let event = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::SHIFT);

        assert!(cmd.is_relevant(&context, &event));
    }

    #[test]
    fn go_to_bottom_should_be_relevant_for_uppercase_g_with_shift_in_normal_mode() {
        let context = create_test_context(EditorMode::Normal);
        let cmd = GoToBottomCommand;
        let event = KeyEvent::new(KeyCode::Char('G'), KeyModifiers::SHIFT);

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
        let context = create_test_context(EditorMode::GPrefix);
        let cmd = GoToBottomCommand;
        let event = create_test_key_event(KeyCode::Char('G'));

        assert!(!cmd.is_relevant(&context, &event));
    }

    #[test]
    fn go_to_bottom_should_not_be_relevant_for_lowercase_g_without_shift() {
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

    #[test]
    fn scroll_page_down_should_be_relevant_for_ctrl_f_in_normal_mode() {
        let context = create_test_context(EditorMode::Normal);
        let cmd = ScrollPageDownCommand;
        let event = KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL);

        assert!(cmd.is_relevant(&context, &event));
    }

    #[test]
    fn scroll_page_down_should_not_be_relevant_for_f_without_ctrl() {
        let context = create_test_context(EditorMode::Normal);
        let cmd = ScrollPageDownCommand;
        let event = create_test_key_event(KeyCode::Char('f'));

        assert!(!cmd.is_relevant(&context, &event));
    }

    #[test]
    fn scroll_page_down_should_not_be_relevant_in_insert_mode() {
        let context = create_test_context(EditorMode::Insert);
        let cmd = ScrollPageDownCommand;
        let event = KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL);

        assert!(!cmd.is_relevant(&context, &event));
    }

    #[test]
    fn scroll_page_down_should_not_be_relevant_in_g_prefix_mode() {
        let context = create_test_context(EditorMode::GPrefix);
        let cmd = ScrollPageDownCommand;
        let event = KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL);

        assert!(!cmd.is_relevant(&context, &event));
    }

    #[test]
    fn scroll_page_down_should_produce_page_down_event() {
        let context = create_test_context(EditorMode::Normal);
        let cmd = ScrollPageDownCommand;
        let event = KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL);

        let events = cmd.execute(event, &context).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0],
            CommandEvent::cursor_move(MovementDirection::PageDown)
        );
    }
}
