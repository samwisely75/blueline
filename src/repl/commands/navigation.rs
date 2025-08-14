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
                (context.state.current_mode == EditorMode::Normal
                    || context.state.current_mode == EditorMode::Visual)
                    && event.modifiers.is_empty()
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
                (context.state.current_mode == EditorMode::Normal
                    || context.state.current_mode == EditorMode::Visual)
                    && event.modifiers.is_empty()
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
                (context.state.current_mode == EditorMode::Normal
                    || context.state.current_mode == EditorMode::Visual)
                    && event.modifiers.is_empty()
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
                (context.state.current_mode == EditorMode::Normal
                    || context.state.current_mode == EditorMode::Visual)
                    && event.modifiers.is_empty()
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

/// Enter G prefix mode on first 'g' press
pub struct EnterGPrefixCommand;

impl Command for EnterGPrefixCommand {
    fn is_relevant(&self, context: &CommandContext, event: &KeyEvent) -> bool {
        matches!(event.code, KeyCode::Char('g'))
            && (context.state.current_mode == EditorMode::Normal
                || context.state.current_mode == EditorMode::Visual)
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
        (context.state.current_mode == EditorMode::Normal
            || context.state.current_mode == EditorMode::Visual)
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

/// Move to next word (w command)
pub struct NextWordCommand;

impl Command for NextWordCommand {
    fn is_relevant(&self, context: &CommandContext, event: &KeyEvent) -> bool {
        matches!(event.code, KeyCode::Char('w'))
            && (context.state.current_mode == EditorMode::Normal
                || context.state.current_mode == EditorMode::Visual)
            && event.modifiers.is_empty()
    }

    fn execute(&self, _event: KeyEvent, _context: &CommandContext) -> Result<Vec<CommandEvent>> {
        Ok(vec![CommandEvent::cursor_move(
            MovementDirection::WordForward,
        )])
    }

    fn name(&self) -> &'static str {
        "NextWord"
    }
}

/// Move to previous word (b command)
pub struct PreviousWordCommand;

impl Command for PreviousWordCommand {
    fn is_relevant(&self, context: &CommandContext, event: &KeyEvent) -> bool {
        matches!(event.code, KeyCode::Char('b'))
            && (context.state.current_mode == EditorMode::Normal
                || context.state.current_mode == EditorMode::Visual)
            && event.modifiers.is_empty()
    }

    fn execute(&self, _event: KeyEvent, _context: &CommandContext) -> Result<Vec<CommandEvent>> {
        Ok(vec![CommandEvent::cursor_move(
            MovementDirection::WordBackward,
        )])
    }

    fn name(&self) -> &'static str {
        "PreviousWord"
    }
}

/// Move to end of word (e command)
pub struct EndOfWordCommand;

impl Command for EndOfWordCommand {
    fn is_relevant(&self, context: &CommandContext, event: &KeyEvent) -> bool {
        matches!(event.code, KeyCode::Char('e'))
            && (context.state.current_mode == EditorMode::Normal
                || context.state.current_mode == EditorMode::Visual)
            && event.modifiers.is_empty()
    }

    fn execute(&self, _event: KeyEvent, _context: &CommandContext) -> Result<Vec<CommandEvent>> {
        Ok(vec![CommandEvent::cursor_move(MovementDirection::WordEnd)])
    }

    fn name(&self) -> &'static str {
        "EndOfWord"
    }
}

/// Move to beginning of line (0 command)
pub struct BeginningOfLineCommand;

impl Command for BeginningOfLineCommand {
    fn is_relevant(&self, context: &CommandContext, event: &KeyEvent) -> bool {
        matches!(event.code, KeyCode::Char('0'))
            && (context.state.current_mode == EditorMode::Normal
                || context.state.current_mode == EditorMode::Visual)
            && event.modifiers.is_empty()
    }

    fn execute(&self, _event: KeyEvent, _context: &CommandContext) -> Result<Vec<CommandEvent>> {
        Ok(vec![CommandEvent::cursor_move(
            MovementDirection::LineStart,
        )])
    }

    fn name(&self) -> &'static str {
        "BeginningOfLine"
    }
}

/// Move to end of line ($ command)
pub struct EndOfLineCommand;

impl Command for EndOfLineCommand {
    fn is_relevant(&self, context: &CommandContext, event: &KeyEvent) -> bool {
        matches!(event.code, KeyCode::Char('$'))
            && (context.state.current_mode == EditorMode::Normal
                || context.state.current_mode == EditorMode::Visual)
            && event.modifiers.is_empty()
    }

    fn execute(&self, _event: KeyEvent, _context: &CommandContext) -> Result<Vec<CommandEvent>> {
        Ok(vec![CommandEvent::cursor_move(MovementDirection::LineEnd)])
    }

    fn name(&self) -> &'static str {
        "EndOfLine"
    }
}

/// Move to beginning of line (Home key)
pub struct HomeKeyCommand;

impl Command for HomeKeyCommand {
    fn is_relevant(&self, _context: &CommandContext, event: &KeyEvent) -> bool {
        matches!(event.code, KeyCode::Home) && event.modifiers.is_empty()
    }

    fn execute(&self, _event: KeyEvent, _context: &CommandContext) -> Result<Vec<CommandEvent>> {
        Ok(vec![CommandEvent::cursor_move(
            MovementDirection::LineStart,
        )])
    }

    fn name(&self) -> &'static str {
        "HomeKey"
    }
}

/// Move to end of line (End key)
pub struct EndKeyCommand;

impl Command for EndKeyCommand {
    fn is_relevant(&self, _context: &CommandContext, event: &KeyEvent) -> bool {
        matches!(event.code, KeyCode::End) && event.modifiers.is_empty()
    }

    fn execute(&self, _event: KeyEvent, _context: &CommandContext) -> Result<Vec<CommandEvent>> {
        Ok(vec![CommandEvent::cursor_move(MovementDirection::LineEnd)])
    }

    fn name(&self) -> &'static str {
        "EndKey"
    }
}

/// Page down navigation (Ctrl+f)
pub struct PageDownCommand;

/// Page up navigation (Ctrl+b)
pub struct PageUpCommand;

/// Half page down navigation (Ctrl+d)
pub struct HalfPageDownCommand;

/// Half page up navigation (Ctrl+u)
pub struct HalfPageUpCommand;

impl Command for PageDownCommand {
    fn is_relevant(&self, context: &CommandContext, event: &KeyEvent) -> bool {
        let is_ctrl_f = matches!(event.code, KeyCode::Char('f'))
            && event.modifiers.contains(KeyModifiers::CONTROL)
            && !event.modifiers.contains(KeyModifiers::SHIFT)
            && !event.modifiers.contains(KeyModifiers::ALT);

        let is_normal_or_visual_mode = context.state.current_mode == EditorMode::Normal
            || context.state.current_mode == EditorMode::Visual;

        let is_relevant = is_ctrl_f && is_normal_or_visual_mode;

        if is_ctrl_f {
            tracing::debug!(
                "PageDownCommand.is_relevant(): ctrl+f={}, mode={:?}, result={}",
                is_ctrl_f,
                context.state.current_mode,
                is_relevant
            );
        }

        is_relevant
    }

    fn execute(&self, _event: KeyEvent, _context: &CommandContext) -> Result<Vec<CommandEvent>> {
        Ok(vec![CommandEvent::cursor_move(MovementDirection::PageDown)])
    }

    fn name(&self) -> &'static str {
        "PageDown"
    }
}

impl Command for PageUpCommand {
    fn is_relevant(&self, context: &CommandContext, event: &KeyEvent) -> bool {
        let is_ctrl_b = matches!(event.code, KeyCode::Char('b'))
            && event.modifiers.contains(KeyModifiers::CONTROL)
            && !event.modifiers.contains(KeyModifiers::SHIFT)
            && !event.modifiers.contains(KeyModifiers::ALT);

        let is_normal_or_visual_mode = context.state.current_mode == EditorMode::Normal
            || context.state.current_mode == EditorMode::Visual;

        let is_relevant = is_ctrl_b && is_normal_or_visual_mode;

        if is_ctrl_b {
            tracing::debug!(
                "PageUpCommand.is_relevant(): ctrl+b={}, mode={:?}, result={}",
                is_ctrl_b,
                context.state.current_mode,
                is_relevant
            );
        }

        is_relevant
    }

    fn execute(&self, _event: KeyEvent, _context: &CommandContext) -> Result<Vec<CommandEvent>> {
        Ok(vec![CommandEvent::cursor_move(MovementDirection::PageUp)])
    }

    fn name(&self) -> &'static str {
        "PageUp"
    }
}

impl Command for HalfPageDownCommand {
    fn is_relevant(&self, context: &CommandContext, event: &KeyEvent) -> bool {
        let is_ctrl_d = matches!(event.code, KeyCode::Char('d'))
            && event.modifiers.contains(KeyModifiers::CONTROL)
            && !event.modifiers.contains(KeyModifiers::SHIFT)
            && !event.modifiers.contains(KeyModifiers::ALT);

        let is_normal_or_visual_mode = context.state.current_mode == EditorMode::Normal
            || context.state.current_mode == EditorMode::Visual;

        let is_relevant = is_ctrl_d && is_normal_or_visual_mode;

        if is_ctrl_d {
            tracing::debug!(
                "HalfPageDownCommand.is_relevant(): ctrl+d={}, mode={:?}, result={}",
                is_ctrl_d,
                context.state.current_mode,
                is_relevant
            );
        }

        is_relevant
    }

    fn execute(&self, _event: KeyEvent, _context: &CommandContext) -> Result<Vec<CommandEvent>> {
        Ok(vec![CommandEvent::cursor_move(
            MovementDirection::HalfPageDown,
        )])
    }

    fn name(&self) -> &'static str {
        "HalfPageDown"
    }
}

impl Command for HalfPageUpCommand {
    fn is_relevant(&self, context: &CommandContext, event: &KeyEvent) -> bool {
        let is_ctrl_u = matches!(event.code, KeyCode::Char('u'))
            && event.modifiers.contains(KeyModifiers::CONTROL)
            && !event.modifiers.contains(KeyModifiers::SHIFT)
            && !event.modifiers.contains(KeyModifiers::ALT);

        let is_normal_or_visual_mode = context.state.current_mode == EditorMode::Normal
            || context.state.current_mode == EditorMode::Visual;

        let is_relevant = is_ctrl_u && is_normal_or_visual_mode;

        if is_ctrl_u {
            tracing::debug!(
                "HalfPageUpCommand.is_relevant(): ctrl+u={}, mode={:?}, result={}",
                is_ctrl_u,
                context.state.current_mode,
                is_relevant
            );
        }

        is_relevant
    }

    fn execute(&self, _event: KeyEvent, _context: &CommandContext) -> Result<Vec<CommandEvent>> {
        Ok(vec![CommandEvent::cursor_move(
            MovementDirection::HalfPageUp,
        )])
    }

    fn name(&self) -> &'static str {
        "HalfPageUp"
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
            terminal_dimensions: (80, 24),
            expand_tab: false,
            tab_width: 4,
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

    // Tests for NextWordCommand (w)
    #[test]
    fn next_word_should_be_relevant_for_w_in_normal_mode() {
        let context = create_test_context(EditorMode::Normal);
        let cmd = NextWordCommand;
        let event = create_test_key_event(KeyCode::Char('w'));

        assert!(cmd.is_relevant(&context, &event));
    }

    #[test]
    fn next_word_should_not_be_relevant_for_w_in_insert_mode() {
        let context = create_test_context(EditorMode::Insert);
        let cmd = NextWordCommand;
        let event = create_test_key_event(KeyCode::Char('w'));

        assert!(!cmd.is_relevant(&context, &event));
    }

    #[test]
    fn next_word_should_not_be_relevant_for_w_in_command_mode() {
        let context = create_test_context(EditorMode::Command);
        let cmd = NextWordCommand;
        let event = create_test_key_event(KeyCode::Char('w'));

        assert!(!cmd.is_relevant(&context, &event));
    }

    #[test]
    fn next_word_should_not_be_relevant_for_w_in_g_prefix_mode() {
        let context = create_test_context(EditorMode::GPrefix);
        let cmd = NextWordCommand;
        let event = create_test_key_event(KeyCode::Char('w'));

        assert!(!cmd.is_relevant(&context, &event));
    }

    #[test]
    fn next_word_should_not_be_relevant_for_w_with_modifiers() {
        let context = create_test_context(EditorMode::Normal);
        let cmd = NextWordCommand;
        let event = KeyEvent::new(KeyCode::Char('w'), KeyModifiers::CONTROL);

        assert!(!cmd.is_relevant(&context, &event));
    }

    #[test]
    fn next_word_should_not_be_relevant_for_other_keys() {
        let context = create_test_context(EditorMode::Normal);
        let cmd = NextWordCommand;
        let event = create_test_key_event(KeyCode::Char('x'));

        assert!(!cmd.is_relevant(&context, &event));
    }

    #[test]
    fn next_word_should_produce_word_forward_event() {
        let context = create_test_context(EditorMode::Normal);
        let cmd = NextWordCommand;
        let event = create_test_key_event(KeyCode::Char('w'));

        let events = cmd.execute(event, &context).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0],
            CommandEvent::cursor_move(MovementDirection::WordForward)
        );
    }

    // Tests for PreviousWordCommand (b)
    #[test]
    fn previous_word_should_be_relevant_for_b_in_normal_mode() {
        let context = create_test_context(EditorMode::Normal);
        let cmd = PreviousWordCommand;
        let event = create_test_key_event(KeyCode::Char('b'));

        assert!(cmd.is_relevant(&context, &event));
    }

    #[test]
    fn previous_word_should_not_be_relevant_for_b_in_insert_mode() {
        let context = create_test_context(EditorMode::Insert);
        let cmd = PreviousWordCommand;
        let event = create_test_key_event(KeyCode::Char('b'));

        assert!(!cmd.is_relevant(&context, &event));
    }

    #[test]
    fn previous_word_should_not_be_relevant_for_b_with_ctrl() {
        let context = create_test_context(EditorMode::Normal);
        let cmd = PreviousWordCommand;
        let event = KeyEvent::new(KeyCode::Char('b'), KeyModifiers::CONTROL);

        assert!(!cmd.is_relevant(&context, &event));
    }

    #[test]
    fn previous_word_should_produce_word_backward_event() {
        let context = create_test_context(EditorMode::Normal);
        let cmd = PreviousWordCommand;
        let event = create_test_key_event(KeyCode::Char('b'));

        let events = cmd.execute(event, &context).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0],
            CommandEvent::cursor_move(MovementDirection::WordBackward)
        );
    }

    // Tests for EndOfWordCommand (e)
    #[test]
    fn end_of_word_should_be_relevant_for_e_in_normal_mode() {
        let context = create_test_context(EditorMode::Normal);
        let cmd = EndOfWordCommand;
        let event = create_test_key_event(KeyCode::Char('e'));

        assert!(cmd.is_relevant(&context, &event));
    }

    #[test]
    fn end_of_word_should_not_be_relevant_for_e_in_insert_mode() {
        let context = create_test_context(EditorMode::Insert);
        let cmd = EndOfWordCommand;
        let event = create_test_key_event(KeyCode::Char('e'));

        assert!(!cmd.is_relevant(&context, &event));
    }

    #[test]
    fn end_of_word_should_produce_word_end_event() {
        let context = create_test_context(EditorMode::Normal);
        let cmd = EndOfWordCommand;
        let event = create_test_key_event(KeyCode::Char('e'));

        let events = cmd.execute(event, &context).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0],
            CommandEvent::cursor_move(MovementDirection::WordEnd)
        );
    }

    // Tests for BeginningOfLineCommand (0)
    #[test]
    fn beginning_of_line_should_be_relevant_for_zero_in_normal_mode() {
        let context = create_test_context(EditorMode::Normal);
        let cmd = BeginningOfLineCommand;
        let event = create_test_key_event(KeyCode::Char('0'));

        assert!(cmd.is_relevant(&context, &event));
    }

    #[test]
    fn beginning_of_line_should_not_be_relevant_for_zero_in_insert_mode() {
        let context = create_test_context(EditorMode::Insert);
        let cmd = BeginningOfLineCommand;
        let event = create_test_key_event(KeyCode::Char('0'));

        assert!(!cmd.is_relevant(&context, &event));
    }

    #[test]
    fn beginning_of_line_should_produce_line_start_event() {
        let context = create_test_context(EditorMode::Normal);
        let cmd = BeginningOfLineCommand;
        let event = create_test_key_event(KeyCode::Char('0'));

        let events = cmd.execute(event, &context).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0],
            CommandEvent::cursor_move(MovementDirection::LineStart)
        );
    }

    // Tests for EndOfLineCommand ($)
    #[test]
    fn end_of_line_should_be_relevant_for_dollar_in_normal_mode() {
        let context = create_test_context(EditorMode::Normal);
        let cmd = EndOfLineCommand;
        let event = create_test_key_event(KeyCode::Char('$'));

        assert!(cmd.is_relevant(&context, &event));
    }

    #[test]
    fn end_of_line_should_not_be_relevant_for_dollar_in_insert_mode() {
        let context = create_test_context(EditorMode::Insert);
        let cmd = EndOfLineCommand;
        let event = create_test_key_event(KeyCode::Char('$'));

        assert!(!cmd.is_relevant(&context, &event));
    }

    #[test]
    fn end_of_line_should_produce_line_end_event() {
        let context = create_test_context(EditorMode::Normal);
        let cmd = EndOfLineCommand;
        let event = create_test_key_event(KeyCode::Char('$'));

        let events = cmd.execute(event, &context).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0],
            CommandEvent::cursor_move(MovementDirection::LineEnd)
        );
    }

    // Tests for HomeKeyCommand
    #[test]
    fn home_key_should_be_relevant_for_home_key() {
        let context = create_test_context(EditorMode::Normal);
        let cmd = HomeKeyCommand;
        let event = create_test_key_event(KeyCode::Home);

        assert!(cmd.is_relevant(&context, &event));
    }

    #[test]
    fn home_key_should_be_relevant_in_insert_mode() {
        let context = create_test_context(EditorMode::Insert);
        let cmd = HomeKeyCommand;
        let event = create_test_key_event(KeyCode::Home);

        assert!(cmd.is_relevant(&context, &event));
    }

    #[test]
    fn home_key_should_produce_line_start_event() {
        let context = create_test_context(EditorMode::Normal);
        let cmd = HomeKeyCommand;
        let event = create_test_key_event(KeyCode::Home);

        let events = cmd.execute(event, &context).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0],
            CommandEvent::cursor_move(MovementDirection::LineStart)
        );
    }

    // Tests for EndKeyCommand
    #[test]
    fn end_key_should_be_relevant_for_end_key() {
        let context = create_test_context(EditorMode::Normal);
        let cmd = EndKeyCommand;
        let event = create_test_key_event(KeyCode::End);

        assert!(cmd.is_relevant(&context, &event));
    }

    #[test]
    fn end_key_should_be_relevant_in_insert_mode() {
        let context = create_test_context(EditorMode::Insert);
        let cmd = EndKeyCommand;
        let event = create_test_key_event(KeyCode::End);

        assert!(cmd.is_relevant(&context, &event));
    }

    #[test]
    fn end_key_should_produce_line_end_event() {
        let context = create_test_context(EditorMode::Normal);
        let cmd = EndKeyCommand;
        let event = create_test_key_event(KeyCode::End);

        let events = cmd.execute(event, &context).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0],
            CommandEvent::cursor_move(MovementDirection::LineEnd)
        );
    }

    // Visual mode navigation tests
    #[test]
    fn move_cursor_left_should_be_relevant_for_h_in_visual_mode() {
        let context = create_test_context(EditorMode::Visual);
        let cmd = MoveCursorLeftCommand;
        let event = create_test_key_event(KeyCode::Char('h'));

        assert!(cmd.is_relevant(&context, &event));
    }

    #[test]
    fn move_cursor_right_should_be_relevant_for_l_in_visual_mode() {
        let context = create_test_context(EditorMode::Visual);
        let cmd = MoveCursorRightCommand;
        let event = create_test_key_event(KeyCode::Char('l'));

        assert!(cmd.is_relevant(&context, &event));
    }

    #[test]
    fn move_cursor_up_should_be_relevant_for_k_in_visual_mode() {
        let context = create_test_context(EditorMode::Visual);
        let cmd = MoveCursorUpCommand;
        let event = create_test_key_event(KeyCode::Char('k'));

        assert!(cmd.is_relevant(&context, &event));
    }

    #[test]
    fn move_cursor_down_should_be_relevant_for_j_in_visual_mode() {
        let context = create_test_context(EditorMode::Visual);
        let cmd = MoveCursorDownCommand;
        let event = create_test_key_event(KeyCode::Char('j'));

        assert!(cmd.is_relevant(&context, &event));
    }

    #[test]
    fn next_word_should_be_relevant_for_w_in_visual_mode() {
        let context = create_test_context(EditorMode::Visual);
        let cmd = NextWordCommand;
        let event = create_test_key_event(KeyCode::Char('w'));

        assert!(cmd.is_relevant(&context, &event));
    }

    #[test]
    fn previous_word_should_be_relevant_for_b_in_visual_mode() {
        let context = create_test_context(EditorMode::Visual);
        let cmd = PreviousWordCommand;
        let event = create_test_key_event(KeyCode::Char('b'));

        assert!(cmd.is_relevant(&context, &event));
    }

    #[test]
    fn end_of_word_should_be_relevant_for_e_in_visual_mode() {
        let context = create_test_context(EditorMode::Visual);
        let cmd = EndOfWordCommand;
        let event = create_test_key_event(KeyCode::Char('e'));

        assert!(cmd.is_relevant(&context, &event));
    }

    #[test]
    fn beginning_of_line_should_be_relevant_for_zero_in_visual_mode() {
        let context = create_test_context(EditorMode::Visual);
        let cmd = BeginningOfLineCommand;
        let event = create_test_key_event(KeyCode::Char('0'));

        assert!(cmd.is_relevant(&context, &event));
    }

    #[test]
    fn end_of_line_should_be_relevant_for_dollar_in_visual_mode() {
        let context = create_test_context(EditorMode::Visual);
        let cmd = EndOfLineCommand;
        let event = create_test_key_event(KeyCode::Char('$'));

        assert!(cmd.is_relevant(&context, &event));
    }

    #[test]
    fn go_to_bottom_should_be_relevant_for_uppercase_g_in_visual_mode() {
        let context = create_test_context(EditorMode::Visual);
        let cmd = GoToBottomCommand;
        let event = create_test_key_event(KeyCode::Char('G'));

        assert!(cmd.is_relevant(&context, &event));
    }

    #[test]
    fn enter_g_prefix_should_be_relevant_for_g_in_visual_mode() {
        let context = create_test_context(EditorMode::Visual);
        let cmd = EnterGPrefixCommand;
        let event = create_test_key_event(KeyCode::Char('g'));

        assert!(cmd.is_relevant(&context, &event));
    }

    // Tests for PageDownCommand (Ctrl+f)
    #[test]
    fn page_down_should_be_relevant_for_ctrl_f_in_normal_mode() {
        let context = create_test_context(EditorMode::Normal);
        let cmd = PageDownCommand;
        let event = KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL);

        assert!(cmd.is_relevant(&context, &event));
    }

    #[test]
    fn page_down_should_be_relevant_for_ctrl_f_in_visual_mode() {
        let context = create_test_context(EditorMode::Visual);
        let cmd = PageDownCommand;
        let event = KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL);

        assert!(cmd.is_relevant(&context, &event));
    }

    #[test]
    fn page_down_should_not_be_relevant_for_ctrl_f_in_insert_mode() {
        let context = create_test_context(EditorMode::Insert);
        let cmd = PageDownCommand;
        let event = KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL);

        assert!(!cmd.is_relevant(&context, &event));
    }

    #[test]
    fn page_down_should_not_be_relevant_for_ctrl_f_in_command_mode() {
        let context = create_test_context(EditorMode::Command);
        let cmd = PageDownCommand;
        let event = KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL);

        assert!(!cmd.is_relevant(&context, &event));
    }

    #[test]
    fn page_down_should_not_be_relevant_for_f_without_ctrl() {
        let context = create_test_context(EditorMode::Normal);
        let cmd = PageDownCommand;
        let event = create_test_key_event(KeyCode::Char('f'));

        assert!(!cmd.is_relevant(&context, &event));
    }

    #[test]
    fn page_down_should_not_be_relevant_for_ctrl_shift_f() {
        let context = create_test_context(EditorMode::Normal);
        let cmd = PageDownCommand;
        let event = KeyEvent::new(
            KeyCode::Char('f'),
            KeyModifiers::CONTROL | KeyModifiers::SHIFT,
        );

        assert!(!cmd.is_relevant(&context, &event));
    }

    #[test]
    fn page_down_should_not_be_relevant_for_ctrl_alt_f() {
        let context = create_test_context(EditorMode::Normal);
        let cmd = PageDownCommand;
        let event = KeyEvent::new(
            KeyCode::Char('f'),
            KeyModifiers::CONTROL | KeyModifiers::ALT,
        );

        assert!(!cmd.is_relevant(&context, &event));
    }

    #[test]
    fn page_down_should_produce_page_down_movement_event() {
        let context = create_test_context(EditorMode::Normal);
        let cmd = PageDownCommand;
        let event = KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL);

        let events = cmd.execute(event, &context).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0],
            CommandEvent::cursor_move(MovementDirection::PageDown)
        );
    }

    #[test]
    fn page_down_should_return_correct_command_name() {
        let cmd = PageDownCommand;
        assert_eq!(cmd.name(), "PageDown");
    }

    // Tests for PageUpCommand (Ctrl+b)
    #[test]
    fn page_up_should_be_relevant_for_ctrl_b_in_normal_mode() {
        let context = create_test_context(EditorMode::Normal);
        let cmd = PageUpCommand;
        let event = KeyEvent::new(KeyCode::Char('b'), KeyModifiers::CONTROL);

        assert!(cmd.is_relevant(&context, &event));
    }

    #[test]
    fn page_up_should_be_relevant_for_ctrl_b_in_visual_mode() {
        let context = create_test_context(EditorMode::Visual);
        let cmd = PageUpCommand;
        let event = KeyEvent::new(KeyCode::Char('b'), KeyModifiers::CONTROL);

        assert!(cmd.is_relevant(&context, &event));
    }

    #[test]
    fn page_up_should_not_be_relevant_for_ctrl_b_in_insert_mode() {
        let context = create_test_context(EditorMode::Insert);
        let cmd = PageUpCommand;
        let event = KeyEvent::new(KeyCode::Char('b'), KeyModifiers::CONTROL);

        assert!(!cmd.is_relevant(&context, &event));
    }

    #[test]
    fn page_up_should_not_be_relevant_for_ctrl_b_in_command_mode() {
        let context = create_test_context(EditorMode::Command);
        let cmd = PageUpCommand;
        let event = KeyEvent::new(KeyCode::Char('b'), KeyModifiers::CONTROL);

        assert!(!cmd.is_relevant(&context, &event));
    }

    #[test]
    fn page_up_should_not_be_relevant_for_b_without_ctrl() {
        let context = create_test_context(EditorMode::Normal);
        let cmd = PageUpCommand;
        let event = create_test_key_event(KeyCode::Char('b'));

        assert!(!cmd.is_relevant(&context, &event));
    }

    #[test]
    fn page_up_should_not_be_relevant_for_ctrl_shift_b() {
        let context = create_test_context(EditorMode::Normal);
        let cmd = PageUpCommand;
        let event = KeyEvent::new(
            KeyCode::Char('b'),
            KeyModifiers::CONTROL | KeyModifiers::SHIFT,
        );

        assert!(!cmd.is_relevant(&context, &event));
    }

    #[test]
    fn page_up_should_not_be_relevant_for_ctrl_alt_b() {
        let context = create_test_context(EditorMode::Normal);
        let cmd = PageUpCommand;
        let event = KeyEvent::new(
            KeyCode::Char('b'),
            KeyModifiers::CONTROL | KeyModifiers::ALT,
        );

        assert!(!cmd.is_relevant(&context, &event));
    }

    #[test]
    fn page_up_should_produce_page_up_movement_event() {
        let context = create_test_context(EditorMode::Normal);
        let cmd = PageUpCommand;
        let event = KeyEvent::new(KeyCode::Char('b'), KeyModifiers::CONTROL);

        let events = cmd.execute(event, &context).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0],
            CommandEvent::cursor_move(MovementDirection::PageUp)
        );
    }

    #[test]
    fn page_up_should_return_correct_command_name() {
        let cmd = PageUpCommand;
        assert_eq!(cmd.name(), "PageUp");
    }
}
