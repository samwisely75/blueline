//! # Movement Commands
//!
//! This module contains command implementations for cursor movement in both
//! the Request and Response panes. These commands follow vim-style navigation.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::repl::{
    command::{Command, CommandResult},
    model::{AppState, EditorMode, Pane, RequestBuffer, ResponseBuffer},
};

/// Helper trait to provide common movement operations for both buffer types
trait MovementBuffer {
    fn cursor_line(&self) -> usize;
    fn cursor_line_mut(&mut self) -> &mut usize;
    fn cursor_col_mut(&mut self) -> &mut usize;
    fn scroll_offset_mut(&mut self) -> &mut usize;
    fn lines(&self) -> &[String];
}

impl MovementBuffer for RequestBuffer {
    fn cursor_line(&self) -> usize {
        self.cursor_line
    }
    fn cursor_line_mut(&mut self) -> &mut usize {
        &mut self.cursor_line
    }
    fn cursor_col_mut(&mut self) -> &mut usize {
        &mut self.cursor_col
    }
    fn scroll_offset_mut(&mut self) -> &mut usize {
        &mut self.scroll_offset
    }
    fn lines(&self) -> &[String] {
        &self.lines
    }
}

impl MovementBuffer for ResponseBuffer {
    fn cursor_line(&self) -> usize {
        self.cursor_line
    }
    fn cursor_line_mut(&mut self) -> &mut usize {
        &mut self.cursor_line
    }
    fn cursor_col_mut(&mut self) -> &mut usize {
        &mut self.cursor_col
    }
    fn scroll_offset_mut(&mut self) -> &mut usize {
        &mut self.scroll_offset
    }
    fn lines(&self) -> &[String] {
        &self.lines
    }
}

/// Move cursor up by one line, handling scroll and column adjustment
fn move_cursor_up<T: MovementBuffer>(buffer: &mut T) -> Result<CommandResult> {
    let cursor_line = buffer.cursor_line();
    if cursor_line > 0 {
        *buffer.cursor_line_mut() -= 1;
        let new_cursor_line = cursor_line - 1;
        let line_len = buffer.lines().get(new_cursor_line).map_or(0, |l| l.len());
        *buffer.cursor_col_mut() = (*buffer.cursor_col_mut()).min(line_len);

        // Auto-scroll up if cursor goes above visible area
        let mut scroll_occurred = false;
        if new_cursor_line < *buffer.scroll_offset_mut() {
            *buffer.scroll_offset_mut() = new_cursor_line;
            scroll_occurred = true;
        }

        let mut result = CommandResult::cursor_moved();
        if scroll_occurred {
            result = result.with_scroll();
        }
        Ok(result)
    } else {
        Ok(CommandResult::not_handled())
    }
}

/// Move cursor down by one line, handling scroll and column adjustment
fn move_cursor_down<T: MovementBuffer>(
    buffer: &mut T,
    visible_height: usize,
) -> Result<CommandResult> {
    let cursor_line = buffer.cursor_line();
    if cursor_line < buffer.lines().len().saturating_sub(1) {
        *buffer.cursor_line_mut() += 1;
        let new_cursor_line = cursor_line + 1;
        let line_len = buffer.lines().get(new_cursor_line).map_or(0, |l| l.len());
        *buffer.cursor_col_mut() = (*buffer.cursor_col_mut()).min(line_len);

        // Auto-scroll down if cursor goes below visible area
        let mut scroll_occurred = false;
        if new_cursor_line >= *buffer.scroll_offset_mut() + visible_height {
            *buffer.scroll_offset_mut() = new_cursor_line - visible_height + 1;
            scroll_occurred = true;
        }

        let mut result = CommandResult::cursor_moved();
        if scroll_occurred {
            result = result.with_scroll();
        }
        Ok(result)
    } else {
        Ok(CommandResult::not_handled())
    }
}

/// Move cursor left by one column
fn move_cursor_left<T: MovementBuffer>(buffer: &mut T) -> Result<CommandResult> {
    if *buffer.cursor_col_mut() > 0 {
        *buffer.cursor_col_mut() -= 1;
        Ok(CommandResult::cursor_moved())
    } else {
        Ok(CommandResult::not_handled())
    }
}

/// Move cursor right by one column
fn move_cursor_right<T: MovementBuffer>(buffer: &mut T) -> Result<CommandResult> {
    let cursor_line = buffer.cursor_line();
    let cursor_col = *buffer.cursor_col_mut();
    if let Some(line) = buffer.lines().get(cursor_line) {
        if cursor_col < line.len() {
            *buffer.cursor_col_mut() += 1;
            return Ok(CommandResult::cursor_moved());
        }
    }
    Ok(CommandResult::not_handled())
}

/// Move cursor to start of line
fn move_cursor_line_start<T: MovementBuffer>(buffer: &mut T) -> Result<CommandResult> {
    *buffer.cursor_col_mut() = 0;
    Ok(CommandResult::cursor_moved())
}

/// Move cursor to end of line
fn move_cursor_line_end<T: MovementBuffer>(buffer: &mut T) -> Result<CommandResult> {
    let cursor_line = buffer.cursor_line();
    if let Some(line) = buffer.lines().get(cursor_line) {
        *buffer.cursor_col_mut() = line.len();
    }
    Ok(CommandResult::cursor_moved())
}

/// Command for moving cursor left (h, Left arrow)
pub struct MoveCursorLeftCommand;

impl MoveCursorLeftCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Command for MoveCursorLeftCommand {
    fn is_relevant(&self, state: &AppState, event: &KeyEvent) -> bool {
        // Only relevant in Normal mode and for h/Left arrow keys
        matches!(state.mode, EditorMode::Normal)
            && match event.code {
                KeyCode::Char('h') => event.modifiers == KeyModifiers::NONE,
                KeyCode::Left => true,
                _ => false,
            }
    }

    fn process(&self, _event: KeyEvent, state: &mut AppState) -> Result<bool> {
        match state.current_pane {
            Pane::Request => {
                move_cursor_left(&mut state.request_buffer)?;
                Ok(true)
            }
            Pane::Response => {
                if let Some(ref mut buffer) = state.response_buffer {
                    move_cursor_left(buffer)?;
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
        }
    }

    fn name(&self) -> &'static str {
        "MoveCursorLeft"
    }
}

/// Command for moving cursor right (l, Right arrow)
pub struct MoveCursorRightCommand;

impl MoveCursorRightCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Command for MoveCursorRightCommand {
    fn is_relevant(&self, state: &AppState, event: &KeyEvent) -> bool {
        // Only relevant in Normal mode and for l/Right arrow keys
        matches!(state.mode, EditorMode::Normal)
            && match event.code {
                KeyCode::Char('l') => event.modifiers == KeyModifiers::NONE,
                KeyCode::Right => true,
                _ => false,
            }
    }

    fn process(&self, _event: KeyEvent, state: &mut AppState) -> Result<bool> {
        match state.current_pane {
            Pane::Request => {
                move_cursor_right(&mut state.request_buffer)?;
                Ok(true)
            }
            Pane::Response => {
                if let Some(ref mut buffer) = state.response_buffer {
                    move_cursor_right(buffer)?;
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
        }
    }

    fn name(&self) -> &'static str {
        "MoveCursorRight"
    }
}

/// Command for moving cursor up (k, Up arrow)
pub struct MoveCursorUpCommand;

impl MoveCursorUpCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Command for MoveCursorUpCommand {
    fn is_relevant(&self, state: &AppState, event: &KeyEvent) -> bool {
        // Only relevant in Normal mode and for k/Up arrow keys
        matches!(state.mode, EditorMode::Normal)
            && match event.code {
                KeyCode::Char('k') => event.modifiers == KeyModifiers::NONE,
                KeyCode::Up => true,
                _ => false,
            }
    }

    fn process(&self, _event: KeyEvent, state: &mut AppState) -> Result<bool> {
        match state.current_pane {
            Pane::Request => {
                move_cursor_up(&mut state.request_buffer)?;
                Ok(true)
            }
            Pane::Response => {
                if let Some(ref mut buffer) = state.response_buffer {
                    move_cursor_up(buffer)?;
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
        }
    }

    fn name(&self) -> &'static str {
        "MoveCursorUp"
    }
}

/// Command for moving cursor down (j, Down arrow)
pub struct MoveCursorDownCommand;

impl MoveCursorDownCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Command for MoveCursorDownCommand {
    fn is_relevant(&self, state: &AppState, event: &KeyEvent) -> bool {
        // Only relevant in Normal mode and for j/Down arrow keys
        matches!(state.mode, EditorMode::Normal)
            && match event.code {
                KeyCode::Char('j') => event.modifiers == KeyModifiers::NONE,
                KeyCode::Down => true,
                _ => false,
            }
    }

    fn process(&self, _event: KeyEvent, state: &mut AppState) -> Result<bool> {
        // Get visible heights before mutable borrows
        let request_visible_height = state.get_request_pane_height();
        let response_visible_height = state.get_response_pane_height();

        match state.current_pane {
            Pane::Request => {
                move_cursor_down(&mut state.request_buffer, request_visible_height)?;
                Ok(true)
            }
            Pane::Response => {
                if let Some(ref mut buffer) = state.response_buffer {
                    move_cursor_down(buffer, response_visible_height)?;
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
        }
    }

    fn name(&self) -> &'static str {
        "MoveCursorDown"
    }
}

/// Command for moving cursor to line start (0)
pub struct MoveCursorLineStartCommand;

impl MoveCursorLineStartCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Command for MoveCursorLineStartCommand {
    fn is_relevant(&self, state: &AppState, event: &KeyEvent) -> bool {
        // Only relevant in Normal mode and for '0' key
        matches!(state.mode, EditorMode::Normal)
            && matches!(event.code, KeyCode::Char('0'))
            && event.modifiers == KeyModifiers::NONE
    }

    fn process(&self, _event: KeyEvent, state: &mut AppState) -> Result<bool> {
        match state.current_pane {
            Pane::Request => {
                move_cursor_line_start(&mut state.request_buffer)?;
                Ok(true)
            }
            Pane::Response => {
                if let Some(ref mut buffer) = state.response_buffer {
                    move_cursor_line_start(buffer)?;
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
        }
    }

    fn name(&self) -> &'static str {
        "MoveCursorLineStart"
    }
}

/// Command for moving cursor to line end ($)
pub struct MoveCursorLineEndCommand;

impl MoveCursorLineEndCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Command for MoveCursorLineEndCommand {
    fn is_relevant(&self, state: &AppState, event: &KeyEvent) -> bool {
        // Only relevant in Normal mode and for '$' key
        matches!(state.mode, EditorMode::Normal)
            && matches!(event.code, KeyCode::Char('$'))
            && event.modifiers == KeyModifiers::NONE
    }

    fn process(&self, _event: KeyEvent, state: &mut AppState) -> Result<bool> {
        match state.current_pane {
            Pane::Request => {
                move_cursor_line_end(&mut state.request_buffer)?;
                Ok(true)
            }
            Pane::Response => {
                if let Some(ref mut buffer) = state.response_buffer {
                    move_cursor_line_end(buffer)?;
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
        }
    }

    fn name(&self) -> &'static str {
        "MoveCursorLineEnd"
    }
}

/// Command for switching between panes (Ctrl+W w)
pub struct SwitchPaneCommand;

impl SwitchPaneCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Command for SwitchPaneCommand {
    fn is_relevant(&self, state: &AppState, event: &KeyEvent) -> bool {
        // Only relevant in Normal mode and for Ctrl+W sequences or pending Ctrl+W state
        matches!(state.mode, EditorMode::Normal)
            && ((event.modifiers.contains(KeyModifiers::CONTROL)
                && matches!(event.code, KeyCode::Char('w')))
                || state.pending_ctrl_w)
    }

    fn process(&self, event: KeyEvent, state: &mut AppState) -> Result<bool> {
        // Handle Ctrl+W (first step)
        if event.modifiers.contains(KeyModifiers::CONTROL)
            && matches!(event.code, KeyCode::Char('w'))
        {
            state.pending_ctrl_w = true;
            return Ok(true);
        }

        // Handle second step of Ctrl+W commands
        if state.pending_ctrl_w {
            match event.code {
                KeyCode::Char('w') => {
                    // Ctrl+W w - switch to next window
                    state.current_pane = match state.current_pane {
                        Pane::Request => Pane::Response,
                        Pane::Response => Pane::Request,
                    };
                    state.pending_ctrl_w = false;
                    return Ok(true);
                }
                KeyCode::Esc => {
                    // Cancel Ctrl+W command
                    state.pending_ctrl_w = false;
                    return Ok(true);
                }
                _ => {
                    // Invalid Ctrl+W command
                    state.pending_ctrl_w = false;
                    state.status_message = "Invalid window command".to_string();
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    fn name(&self) -> &'static str {
        "SwitchPane"
    }
}

/// Command for scrolling up half a page (Ctrl+U)
pub struct ScrollHalfPageUpCommand;

impl ScrollHalfPageUpCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Command for ScrollHalfPageUpCommand {
    fn is_relevant(&self, state: &AppState, event: &KeyEvent) -> bool {
        // Only relevant in Normal mode and for Ctrl+U
        matches!(state.mode, EditorMode::Normal)
            && event.modifiers.contains(KeyModifiers::CONTROL)
            && matches!(event.code, KeyCode::Char('u'))
    }

    fn process(&self, _event: KeyEvent, state: &mut AppState) -> Result<bool> {
        match state.current_pane {
            Pane::Request => {
                let half_page_size = state.get_request_pane_height() / 2;
                state.request_buffer.scroll_half_page_up(half_page_size);
                Ok(true)
            }
            Pane::Response => {
                let half_page_size = state.get_response_pane_height() / 2;
                if let Some(ref mut buffer) = state.response_buffer {
                    buffer.scroll_half_page_up(half_page_size);
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
        }
    }

    fn name(&self) -> &'static str {
        "ScrollHalfPageUp"
    }
}

/// Command for scrolling down half a page (Ctrl+D)
pub struct ScrollHalfPageDownCommand;

impl ScrollHalfPageDownCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Command for ScrollHalfPageDownCommand {
    fn is_relevant(&self, state: &AppState, event: &KeyEvent) -> bool {
        // Only relevant in Normal mode and for Ctrl+D
        matches!(state.mode, EditorMode::Normal)
            && event.modifiers.contains(KeyModifiers::CONTROL)
            && matches!(event.code, KeyCode::Char('d'))
    }

    fn process(&self, _event: KeyEvent, state: &mut AppState) -> Result<bool> {
        match state.current_pane {
            Pane::Request => {
                let half_page_size = state.get_request_pane_height() / 2;
                state.request_buffer.scroll_half_page_down(half_page_size);
                Ok(true)
            }
            Pane::Response => {
                let half_page_size = state.get_response_pane_height() / 2;
                if let Some(ref mut buffer) = state.response_buffer {
                    buffer.scroll_half_page_down(half_page_size);
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
        }
    }

    fn name(&self) -> &'static str {
        "ScrollHalfPageDown"
    }
}

/// Command for scrolling down a full page (Ctrl+F)
pub struct ScrollFullPageDownCommand;

impl ScrollFullPageDownCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Command for ScrollFullPageDownCommand {
    fn is_relevant(&self, state: &AppState, event: &KeyEvent) -> bool {
        // Relevant in Normal mode for Ctrl+F, and in both Normal and Insert modes for PageDown
        (matches!(state.mode, EditorMode::Normal)
            && event.modifiers.contains(KeyModifiers::CONTROL)
            && matches!(event.code, KeyCode::Char('f')))
            || (matches!(state.mode, EditorMode::Normal | EditorMode::Insert)
                && matches!(event.code, KeyCode::PageDown))
    }

    fn process(&self, _event: KeyEvent, state: &mut AppState) -> Result<bool> {
        match state.current_pane {
            Pane::Request => {
                let page_size = state.get_request_pane_height();
                state.request_buffer.scroll_full_page_down(page_size);
                Ok(true)
            }
            Pane::Response => {
                let page_size = state.get_response_pane_height();
                if let Some(ref mut buffer) = state.response_buffer {
                    buffer.scroll_full_page_down(page_size);
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
        }
    }

    fn name(&self) -> &'static str {
        "ScrollFullPageDown"
    }
}

/// Command for scrolling up a full page (Ctrl+B)
pub struct ScrollFullPageUpCommand;

impl ScrollFullPageUpCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Command for ScrollFullPageUpCommand {
    fn is_relevant(&self, state: &AppState, event: &KeyEvent) -> bool {
        // Relevant in Normal mode for Ctrl+B, and in both Normal and Insert modes for PageUp
        (matches!(state.mode, EditorMode::Normal)
            && event.modifiers.contains(KeyModifiers::CONTROL)
            && matches!(event.code, KeyCode::Char('b')))
            || (matches!(state.mode, EditorMode::Normal | EditorMode::Insert)
                && matches!(event.code, KeyCode::PageUp))
    }

    fn process(&self, _event: KeyEvent, state: &mut AppState) -> Result<bool> {
        match state.current_pane {
            Pane::Request => {
                let page_size = state.get_request_pane_height();
                state.request_buffer.scroll_full_page_up(page_size);
                Ok(true)
            }
            Pane::Response => {
                let page_size = state.get_response_pane_height();
                if let Some(ref mut buffer) = state.response_buffer {
                    buffer.scroll_full_page_up(page_size);
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
        }
    }

    fn name(&self) -> &'static str {
        "ScrollFullPageUp"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::model::{RequestBuffer, ResponseBuffer};

    /// Create a test request buffer with sample content for testing movement operations
    fn create_test_request_buffer() -> RequestBuffer {
        RequestBuffer {
            lines: vec![
                "GET /api/users".to_string(),
                "Host: example.com".to_string(),
                "".to_string(),
                "{\"name\": \"test\"}".to_string(),
            ],
            cursor_line: 0,
            cursor_col: 0,
            scroll_offset: 0,
        }
    }

    /// Create a test response buffer with sample content for testing movement operations
    fn create_test_response_buffer() -> ResponseBuffer {
        ResponseBuffer::new(
            "HTTP/1.1 200 OK\nContent-Type: application/json\n\n{\"users\": []}".to_string(),
        )
    }

    /// Create a test AppState for command testing
    fn create_test_app_state() -> AppState {
        AppState::new((80, 24), false)
    }

    #[test]
    fn move_cursor_up_should_move_cursor_up_one_line_and_adjust_column() {
        let mut buffer = create_test_request_buffer();
        buffer.cursor_line = 1;
        buffer.cursor_col = 10;

        let result = move_cursor_up(&mut buffer).unwrap();

        assert_eq!(buffer.cursor_line, 0);
        assert_eq!(buffer.cursor_col, 10); // Column stays same if line is long enough
        assert!(result.cursor_moved);
    }

    #[test]
    fn move_cursor_up_should_adjust_column_when_new_line_is_shorter() {
        let mut buffer = create_test_request_buffer();
        buffer.cursor_line = 3; // On line with "{\"name\": \"test\"}" (16 chars)
        buffer.cursor_col = 15;

        let result = move_cursor_up(&mut buffer).unwrap();

        assert_eq!(buffer.cursor_line, 2);
        assert_eq!(buffer.cursor_col, 0); // Empty line, so cursor moves to column 0
        assert!(result.cursor_moved);
    }

    #[test]
    fn move_cursor_up_should_handle_scroll_when_moving_above_visible_area() {
        let mut buffer = create_test_request_buffer();
        buffer.cursor_line = 2;
        buffer.scroll_offset = 2; // Scrolled down

        let result = move_cursor_up(&mut buffer).unwrap();

        assert_eq!(buffer.cursor_line, 1);
        assert_eq!(buffer.scroll_offset, 1); // Should scroll up
        assert!(result.cursor_moved);
        assert!(result.scroll_occurred);
    }

    #[test]
    fn move_cursor_up_should_not_move_when_at_first_line() {
        let mut buffer = create_test_request_buffer();
        buffer.cursor_line = 0;

        let result = move_cursor_up(&mut buffer).unwrap();

        assert_eq!(buffer.cursor_line, 0);
        assert!(!result.cursor_moved);
    }

    #[test]
    fn move_cursor_down_should_move_cursor_down_one_line_and_adjust_column() {
        let mut buffer = create_test_request_buffer();
        buffer.cursor_line = 0;
        buffer.cursor_col = 5;

        let result = move_cursor_down(&mut buffer, 10).unwrap();

        assert_eq!(buffer.cursor_line, 1);
        assert_eq!(buffer.cursor_col, 5); // Column stays same if line is long enough
        assert!(result.cursor_moved);
    }

    #[test]
    fn move_cursor_down_should_adjust_column_when_new_line_is_shorter() {
        let mut buffer = create_test_request_buffer();
        buffer.cursor_line = 1; // On "Host: example.com" (17 chars)
        buffer.cursor_col = 15;

        let result = move_cursor_down(&mut buffer, 10).unwrap();

        assert_eq!(buffer.cursor_line, 2);
        assert_eq!(buffer.cursor_col, 0); // Empty line, so cursor moves to column 0
        assert!(result.cursor_moved);
    }

    #[test]
    fn move_cursor_down_should_handle_scroll_when_moving_below_visible_area() {
        let mut buffer = create_test_request_buffer();
        buffer.cursor_line = 0;
        let visible_height = 2;

        let _result = move_cursor_down(&mut buffer, visible_height).unwrap();
        assert_eq!(buffer.cursor_line, 1);
        assert_eq!(buffer.scroll_offset, 0); // No scroll yet

        let result = move_cursor_down(&mut buffer, visible_height).unwrap();
        assert_eq!(buffer.cursor_line, 2);
        assert_eq!(buffer.scroll_offset, 1); // Should scroll down
        assert!(result.cursor_moved);
        assert!(result.scroll_occurred);
    }

    #[test]
    fn move_cursor_down_should_not_move_when_at_last_line() {
        let mut buffer = create_test_request_buffer();
        buffer.cursor_line = 3; // Last line

        let result = move_cursor_down(&mut buffer, 10).unwrap();

        assert_eq!(buffer.cursor_line, 3);
        assert!(!result.cursor_moved);
    }

    #[test]
    fn move_cursor_left_should_move_cursor_left_one_column() {
        let mut buffer = create_test_request_buffer();
        buffer.cursor_col = 5;

        let result = move_cursor_left(&mut buffer).unwrap();

        assert_eq!(buffer.cursor_col, 4);
        assert!(result.cursor_moved);
    }

    #[test]
    fn move_cursor_left_should_not_move_when_at_start_of_line() {
        let mut buffer = create_test_request_buffer();
        buffer.cursor_col = 0;

        let result = move_cursor_left(&mut buffer).unwrap();

        assert_eq!(buffer.cursor_col, 0);
        assert!(!result.cursor_moved);
    }

    #[test]
    fn move_cursor_right_should_move_cursor_right_one_column() {
        let mut buffer = create_test_request_buffer();
        buffer.cursor_line = 0; // "GET /api/users" (14 chars)
        buffer.cursor_col = 5;

        let result = move_cursor_right(&mut buffer).unwrap();

        assert_eq!(buffer.cursor_col, 6);
        assert!(result.cursor_moved);
    }

    #[test]
    fn move_cursor_right_should_not_move_when_at_end_of_line() {
        let mut buffer = create_test_request_buffer();
        buffer.cursor_line = 0; // "GET /api/users" (14 chars)
        buffer.cursor_col = 14; // At end of line

        let result = move_cursor_right(&mut buffer).unwrap();

        assert_eq!(buffer.cursor_col, 14);
        assert!(!result.cursor_moved);
    }

    #[test]
    fn move_cursor_right_should_not_move_when_line_is_empty() {
        let mut buffer = create_test_request_buffer();
        buffer.cursor_line = 2; // Empty line
        buffer.cursor_col = 0;

        let result = move_cursor_right(&mut buffer).unwrap();

        assert_eq!(buffer.cursor_col, 0);
        assert!(!result.cursor_moved);
    }

    #[test]
    fn move_cursor_line_start_should_move_cursor_to_beginning_of_line() {
        let mut buffer = create_test_request_buffer();
        buffer.cursor_col = 10;

        let result = move_cursor_line_start(&mut buffer).unwrap();

        assert_eq!(buffer.cursor_col, 0);
        assert!(result.cursor_moved);
    }

    #[test]
    fn move_cursor_line_start_should_work_when_already_at_start() {
        let mut buffer = create_test_request_buffer();
        buffer.cursor_col = 0;

        let result = move_cursor_line_start(&mut buffer).unwrap();

        assert_eq!(buffer.cursor_col, 0);
        assert!(result.cursor_moved);
    }

    #[test]
    fn move_cursor_line_end_should_move_cursor_to_end_of_line() {
        let mut buffer = create_test_request_buffer();
        buffer.cursor_line = 0; // "GET /api/users" (14 chars)
        buffer.cursor_col = 5;

        let result = move_cursor_line_end(&mut buffer).unwrap();

        assert_eq!(buffer.cursor_col, 14);
        assert!(result.cursor_moved);
    }

    #[test]
    fn move_cursor_line_end_should_work_when_already_at_end() {
        let mut buffer = create_test_request_buffer();
        buffer.cursor_line = 0; // "GET /api/users" (14 chars)
        buffer.cursor_col = 14;

        let result = move_cursor_line_end(&mut buffer).unwrap();

        assert_eq!(buffer.cursor_col, 14);
        assert!(result.cursor_moved);
    }

    #[test]
    fn move_cursor_line_end_should_handle_empty_line() {
        let mut buffer = create_test_request_buffer();
        buffer.cursor_line = 2; // Empty line
        buffer.cursor_col = 0;

        let result = move_cursor_line_end(&mut buffer).unwrap();

        assert_eq!(buffer.cursor_col, 0);
        assert!(result.cursor_moved);
    }

    #[test]
    fn movement_buffer_trait_should_work_with_response_buffer() {
        let mut buffer = create_test_response_buffer();
        buffer.cursor_col = 5;

        let result = move_cursor_left(&mut buffer).unwrap();

        assert_eq!(buffer.cursor_col, 4);
        assert!(result.cursor_moved);
    }

    #[test]
    fn move_cursor_left_command_should_be_relevant_for_h_key_in_normal_mode() {
        let command = MoveCursorLeftCommand::new();
        let state = create_test_app_state();
        let event = KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE);

        assert!(command.is_relevant(&state, &event));
        assert_eq!(command.name(), "MoveCursorLeft");
    }

    #[test]
    fn move_cursor_left_command_should_be_relevant_for_left_arrow_in_normal_mode() {
        let command = MoveCursorLeftCommand::new();
        let state = create_test_app_state();
        let event = KeyEvent::new(KeyCode::Left, KeyModifiers::NONE);

        assert!(command.is_relevant(&state, &event));
    }

    #[test]
    fn move_cursor_left_command_should_not_be_relevant_in_insert_mode() {
        let command = MoveCursorLeftCommand::new();
        let mut state = create_test_app_state();
        state.mode = EditorMode::Insert;
        let event = KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE);

        assert!(!command.is_relevant(&state, &event));
    }

    #[test]
    fn move_cursor_right_command_should_be_relevant_for_l_key_in_normal_mode() {
        let command = MoveCursorRightCommand::new();
        let state = create_test_app_state();
        let event = KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE);

        assert!(command.is_relevant(&state, &event));
        assert_eq!(command.name(), "MoveCursorRight");
    }

    #[test]
    fn move_cursor_up_command_should_be_relevant_for_k_key_in_normal_mode() {
        let command = MoveCursorUpCommand::new();
        let state = create_test_app_state();
        let event = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);

        assert!(command.is_relevant(&state, &event));
        assert_eq!(command.name(), "MoveCursorUp");
    }

    #[test]
    fn move_cursor_down_command_should_be_relevant_for_j_key_in_normal_mode() {
        let command = MoveCursorDownCommand::new();
        let state = create_test_app_state();
        let event = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);

        assert!(command.is_relevant(&state, &event));
        assert_eq!(command.name(), "MoveCursorDown");
    }

    #[test]
    fn move_cursor_line_start_command_should_be_relevant_for_zero_key_in_normal_mode() {
        let command = MoveCursorLineStartCommand::new();
        let state = create_test_app_state();
        let event = KeyEvent::new(KeyCode::Char('0'), KeyModifiers::NONE);

        assert!(command.is_relevant(&state, &event));
        assert_eq!(command.name(), "MoveCursorLineStart");
    }

    #[test]
    fn move_cursor_line_end_command_should_be_relevant_for_dollar_key_in_normal_mode() {
        let command = MoveCursorLineEndCommand::new();
        let state = create_test_app_state();
        let event = KeyEvent::new(KeyCode::Char('$'), KeyModifiers::NONE);

        assert!(command.is_relevant(&state, &event));
        assert_eq!(command.name(), "MoveCursorLineEnd");
    }

    #[test]
    fn switch_pane_command_should_be_relevant_for_ctrl_w_in_normal_mode() {
        let command = SwitchPaneCommand::new();
        let state = create_test_app_state();
        let event = KeyEvent::new(KeyCode::Char('w'), KeyModifiers::CONTROL);

        assert!(command.is_relevant(&state, &event));
        assert_eq!(command.name(), "SwitchPane");
    }

    #[test]
    fn switch_pane_command_should_be_relevant_when_pending_ctrl_w() {
        let command = SwitchPaneCommand::new();
        let mut state = create_test_app_state();
        state.pending_ctrl_w = true;
        let event = KeyEvent::new(KeyCode::Char('w'), KeyModifiers::NONE);

        assert!(command.is_relevant(&state, &event));
    }

    #[test]
    fn scroll_half_page_up_command_should_be_relevant_for_ctrl_u_in_normal_mode() {
        let command = ScrollHalfPageUpCommand::new();
        let state = create_test_app_state();
        let event = KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL);

        assert!(command.is_relevant(&state, &event));
        assert_eq!(command.name(), "ScrollHalfPageUp");
    }

    #[test]
    fn scroll_half_page_up_command_should_not_be_relevant_in_insert_mode() {
        let command = ScrollHalfPageUpCommand::new();
        let mut state = create_test_app_state();
        state.mode = EditorMode::Insert;
        let event = KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL);

        assert!(!command.is_relevant(&state, &event));
    }

    #[test]
    fn scroll_half_page_up_command_should_not_be_relevant_without_ctrl() {
        let command = ScrollHalfPageUpCommand::new();
        let state = create_test_app_state();
        let event = KeyEvent::new(KeyCode::Char('u'), KeyModifiers::NONE);

        assert!(!command.is_relevant(&state, &event));
    }

    #[test]
    fn scroll_half_page_up_command_should_scroll_request_buffer() {
        let command = ScrollHalfPageUpCommand::new();
        let mut state = create_test_app_state();

        // Set up buffer with enough content to scroll
        state.request_buffer.lines = vec![
            "line 0".to_string(),
            "line 1".to_string(),
            "line 2".to_string(),
            "line 3".to_string(),
            "line 4".to_string(),
            "line 5".to_string(),
        ];
        state.request_buffer.cursor_line = 4;
        state.request_buffer.scroll_offset = 2;

        let event = KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL);
        let result = command.process(event, &mut state).unwrap();

        assert!(result);
        assert_eq!(state.request_buffer.scroll_offset, 0);
        assert_eq!(state.request_buffer.cursor_line, 0); // Cursor moves to top of visible area (vim behavior)
    }

    #[test]
    fn scroll_half_page_up_command_should_scroll_response_buffer() {
        let command = ScrollHalfPageUpCommand::new();
        let mut state = create_test_app_state();
        state.current_pane = Pane::Response;

        // Set up response buffer
        state.set_response("line 0\nline 1\nline 2\nline 3\nline 4\nline 5".to_string());
        if let Some(ref mut buffer) = state.response_buffer {
            buffer.cursor_line = 4;
            buffer.scroll_offset = 2;
        }

        let event = KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL);
        let result = command.process(event, &mut state).unwrap();

        assert!(result);
        if let Some(ref buffer) = state.response_buffer {
            assert_eq!(buffer.scroll_offset, 0);
            assert_eq!(buffer.cursor_line, 0); // Cursor moves to top of visible area (vim behavior)
        }
    }

    #[test]
    fn scroll_half_page_down_command_should_be_relevant_for_ctrl_d_in_normal_mode() {
        let command = ScrollHalfPageDownCommand::new();
        let state = AppState::new((80, 24), true);
        let event = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL);

        assert!(command.is_relevant(&state, &event));
    }

    #[test]
    fn scroll_half_page_down_command_should_not_be_relevant_in_insert_mode() {
        let command = ScrollHalfPageDownCommand::new();
        let mut state = AppState::new((80, 24), true);
        state.mode = EditorMode::Insert;
        let event = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL);

        assert!(!command.is_relevant(&state, &event));
    }

    #[test]
    fn scroll_half_page_down_command_should_not_be_relevant_without_ctrl() {
        let command = ScrollHalfPageDownCommand::new();
        let state = AppState::new((80, 24), true);
        let event = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE);

        assert!(!command.is_relevant(&state, &event));
    }

    #[test]
    fn scroll_half_page_down_command_should_scroll_request_buffer() {
        let command = ScrollHalfPageDownCommand::new();
        let mut state = AppState::new((80, 24), true);
        // Create enough content to allow scrolling (more than page height of 22)
        state.request_buffer.lines = (0..30).map(|i| format!("line {}", i)).collect();
        state.request_buffer.cursor_line = 1;
        state.request_buffer.scroll_offset = 0;

        let event = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL);
        let result = command.process(event, &mut state).unwrap();

        assert!(result);
        // With 30 lines and page height 22, max scroll is 30-22=8
        // Requested scroll is 11 but limited to available space of 8
        assert_eq!(state.request_buffer.scroll_offset, 8);
        assert_eq!(state.request_buffer.cursor_line, 8);
    }

    #[test]
    fn scroll_half_page_down_command_should_scroll_response_buffer() {
        let command = ScrollHalfPageDownCommand::new();
        let mut state = AppState::new((80, 24), true);
        state.current_pane = Pane::Response;
        // Create enough content to allow scrolling (more than response pane height)
        let content = (0..30)
            .map(|i| format!("line {}", i))
            .collect::<Vec<_>>()
            .join("\n");
        state.response_buffer = Some(ResponseBuffer::new(content));

        let event = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL);
        let result = command.process(event, &mut state).unwrap();

        assert!(result);
        if let Some(ref buffer) = state.response_buffer {
            let expected_scroll = state.get_response_pane_height() / 2;
            assert_eq!(buffer.scroll_offset, expected_scroll);
            assert_eq!(buffer.cursor_line, expected_scroll);
        }
    }

    #[test]
    fn scroll_full_page_down_command_should_be_relevant_for_ctrl_f_in_normal_mode() {
        let command = ScrollFullPageDownCommand::new();
        let state = AppState::new((80, 24), true);
        let event = KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL);

        assert!(command.is_relevant(&state, &event));
        assert_eq!(command.name(), "ScrollFullPageDown");
    }

    #[test]
    fn scroll_full_page_down_command_should_not_be_relevant_in_insert_mode() {
        let command = ScrollFullPageDownCommand::new();
        let mut state = AppState::new((80, 24), true);
        state.mode = EditorMode::Insert;
        let event = KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL);

        assert!(!command.is_relevant(&state, &event));
    }

    #[test]
    fn scroll_full_page_down_command_should_not_be_relevant_without_ctrl() {
        let command = ScrollFullPageDownCommand::new();
        let state = AppState::new((80, 24), true);
        let event = KeyEvent::new(KeyCode::Char('f'), KeyModifiers::NONE);

        assert!(!command.is_relevant(&state, &event));
    }

    #[test]
    fn scroll_full_page_down_command_should_scroll_request_buffer() {
        let command = ScrollFullPageDownCommand::new();
        let mut state = AppState::new((80, 24), true);
        // Create enough content to allow scrolling (more than page height of 22)
        state.request_buffer.lines = (0..50).map(|i| format!("line {}", i)).collect();
        state.request_buffer.cursor_line = 5;
        state.request_buffer.scroll_offset = 0;

        let event = KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL);
        let result = command.process(event, &mut state).unwrap();

        assert!(result);
        let page_size = state.get_request_pane_height();
        assert_eq!(state.request_buffer.scroll_offset, page_size);
        assert_eq!(state.request_buffer.cursor_line, page_size);
    }

    #[test]
    fn scroll_full_page_down_command_should_scroll_response_buffer() {
        let command = ScrollFullPageDownCommand::new();
        let mut state = AppState::new((80, 24), true);
        state.current_pane = Pane::Response;
        // Create enough content to allow scrolling (more than response pane height)
        let content = (0..50)
            .map(|i| format!("line {}", i))
            .collect::<Vec<_>>()
            .join("\n");
        state.response_buffer = Some(ResponseBuffer::new(content));

        let event = KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL);
        let result = command.process(event, &mut state).unwrap();

        assert!(result);
        if let Some(ref buffer) = state.response_buffer {
            let page_size = state.get_response_pane_height();
            assert_eq!(buffer.scroll_offset, page_size);
            assert_eq!(buffer.cursor_line, page_size);
        }
    }

    #[test]
    fn scroll_full_page_down_command_should_handle_scroll_bounds() {
        let command = ScrollFullPageDownCommand::new();
        let mut state = AppState::new((80, 24), true);
        // Create content with limited lines (less than 2 pages)
        state.request_buffer.lines = (0..30).map(|i| format!("line {}", i)).collect();
        state.request_buffer.cursor_line = 0;
        state.request_buffer.scroll_offset = 0;

        let event = KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL);
        let result = command.process(event, &mut state).unwrap();

        assert!(result);
        // With 30 lines and page height 22, max scroll is 30-22=8
        // Requested scroll is 22 but limited to available space of 8
        assert_eq!(state.request_buffer.scroll_offset, 8);
        assert_eq!(state.request_buffer.cursor_line, 8);
    }

    #[test]
    fn scroll_full_page_up_command_should_be_relevant_for_ctrl_b_in_normal_mode() {
        let command = ScrollFullPageUpCommand::new();
        let state = AppState::new((80, 24), true);
        let event = KeyEvent::new(KeyCode::Char('b'), KeyModifiers::CONTROL);

        assert!(command.is_relevant(&state, &event));
        assert_eq!(command.name(), "ScrollFullPageUp");
    }

    #[test]
    fn scroll_full_page_up_command_should_not_be_relevant_in_insert_mode() {
        let command = ScrollFullPageUpCommand::new();
        let mut state = AppState::new((80, 24), true);
        state.mode = EditorMode::Insert;
        let event = KeyEvent::new(KeyCode::Char('b'), KeyModifiers::CONTROL);

        assert!(!command.is_relevant(&state, &event));
    }

    #[test]
    fn scroll_full_page_up_command_should_not_be_relevant_without_ctrl() {
        let command = ScrollFullPageUpCommand::new();
        let state = AppState::new((80, 24), true);
        let event = KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE);

        assert!(!command.is_relevant(&state, &event));
    }

    #[test]
    fn scroll_full_page_up_command_should_scroll_request_buffer() {
        let command = ScrollFullPageUpCommand::new();
        let mut state = AppState::new((80, 24), true);
        // Create enough content to allow scrolling and set initial scroll position
        state.request_buffer.lines = (0..50).map(|i| format!("line {}", i)).collect();
        let page_size = state.get_request_pane_height();
        state.request_buffer.cursor_line = page_size + 5;
        state.request_buffer.scroll_offset = page_size; // Start scrolled down one page

        let event = KeyEvent::new(KeyCode::Char('b'), KeyModifiers::CONTROL);
        let result = command.process(event, &mut state).unwrap();

        assert!(result);
        assert_eq!(state.request_buffer.scroll_offset, 0); // Should scroll back to top
        assert_eq!(state.request_buffer.cursor_line, 0); // Cursor moves to top of visible area
    }

    #[test]
    fn scroll_full_page_up_command_should_scroll_response_buffer() {
        let command = ScrollFullPageUpCommand::new();
        let mut state = AppState::new((80, 24), true);
        state.current_pane = Pane::Response;
        // Create enough content to allow scrolling
        let content = (0..50)
            .map(|i| format!("line {}", i))
            .collect::<Vec<_>>()
            .join("\n");
        state.response_buffer = Some(ResponseBuffer::new(content));

        // Set initial scroll position
        let page_size = state.get_response_pane_height();
        if let Some(ref mut buffer) = state.response_buffer {
            buffer.cursor_line = page_size + 5;
            buffer.scroll_offset = page_size; // Start scrolled down one page
        }

        let event = KeyEvent::new(KeyCode::Char('b'), KeyModifiers::CONTROL);
        let result = command.process(event, &mut state).unwrap();

        assert!(result);
        if let Some(ref buffer) = state.response_buffer {
            assert_eq!(buffer.scroll_offset, 0); // Should scroll back to top
            assert_eq!(buffer.cursor_line, 0); // Cursor moves to top of visible area
        }
    }

    #[test]
    fn scroll_full_page_up_command_should_handle_scroll_bounds() {
        let command = ScrollFullPageUpCommand::new();
        let mut state = AppState::new((80, 24), true);
        // Create content and set initial scroll position
        state.request_buffer.lines = (0..50).map(|i| format!("line {}", i)).collect();
        state.request_buffer.cursor_line = 5;
        state.request_buffer.scroll_offset = 5; // Start with small scroll offset

        let event = KeyEvent::new(KeyCode::Char('b'), KeyModifiers::CONTROL);
        let result = command.process(event, &mut state).unwrap();

        assert!(result);
        // Should scroll to top since requested page size is larger than current offset
        assert_eq!(state.request_buffer.scroll_offset, 0);
        assert_eq!(state.request_buffer.cursor_line, 0);
    }

    #[test]
    fn scroll_full_page_up_command_should_handle_no_scroll_when_at_top() {
        let command = ScrollFullPageUpCommand::new();
        let mut state = AppState::new((80, 24), true);
        // Create content but start at top
        state.request_buffer.lines = (0..50).map(|i| format!("line {}", i)).collect();
        state.request_buffer.cursor_line = 3;
        state.request_buffer.scroll_offset = 0; // Already at top

        let event = KeyEvent::new(KeyCode::Char('b'), KeyModifiers::CONTROL);
        let result = command.process(event, &mut state).unwrap();

        assert!(result);
        // Should remain at top since we can't scroll up further
        assert_eq!(state.request_buffer.scroll_offset, 0);
        assert_eq!(state.request_buffer.cursor_line, 3); // Cursor should remain unchanged
    }

    #[test]
    fn scroll_full_page_down_command_should_be_relevant_for_page_down_in_normal_mode() {
        let command = ScrollFullPageDownCommand::new();
        let state = AppState::new((80, 24), true);
        let event = KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE);

        assert!(command.is_relevant(&state, &event));
    }

    #[test]
    fn scroll_full_page_down_command_should_be_relevant_for_page_down_in_insert_mode() {
        let command = ScrollFullPageDownCommand::new();
        let mut state = AppState::new((80, 24), true);
        state.mode = EditorMode::Insert;
        let event = KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE);

        assert!(command.is_relevant(&state, &event));
    }

    #[test]
    fn scroll_full_page_down_command_should_process_page_down_key() {
        let command = ScrollFullPageDownCommand::new();
        let mut state = AppState::new((80, 24), true);
        // Create enough content to allow scrolling
        state.request_buffer.lines = (0..50).map(|i| format!("line {}", i)).collect();
        state.request_buffer.cursor_line = 5;
        state.request_buffer.scroll_offset = 0;

        let event = KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE);
        let result = command.process(event, &mut state).unwrap();

        assert!(result);
        let page_size = state.get_request_pane_height();
        assert_eq!(state.request_buffer.scroll_offset, page_size);
        assert_eq!(state.request_buffer.cursor_line, page_size);
    }

    #[test]
    fn scroll_full_page_up_command_should_be_relevant_for_page_up_in_normal_mode() {
        let command = ScrollFullPageUpCommand::new();
        let state = AppState::new((80, 24), true);
        let event = KeyEvent::new(KeyCode::PageUp, KeyModifiers::NONE);

        assert!(command.is_relevant(&state, &event));
    }

    #[test]
    fn scroll_full_page_up_command_should_be_relevant_for_page_up_in_insert_mode() {
        let command = ScrollFullPageUpCommand::new();
        let mut state = AppState::new((80, 24), true);
        state.mode = EditorMode::Insert;
        let event = KeyEvent::new(KeyCode::PageUp, KeyModifiers::NONE);

        assert!(command.is_relevant(&state, &event));
    }

    #[test]
    fn scroll_full_page_up_command_should_process_page_up_key() {
        let command = ScrollFullPageUpCommand::new();
        let mut state = AppState::new((80, 24), true);
        // Create enough content to allow scrolling and set initial scroll position
        state.request_buffer.lines = (0..50).map(|i| format!("line {}", i)).collect();
        let page_size = state.get_request_pane_height();
        state.request_buffer.cursor_line = page_size + 5;
        state.request_buffer.scroll_offset = page_size; // Start scrolled down one page

        let event = KeyEvent::new(KeyCode::PageUp, KeyModifiers::NONE);
        let result = command.process(event, &mut state).unwrap();

        assert!(result);
        assert_eq!(state.request_buffer.scroll_offset, 0); // Should scroll back to top
        assert_eq!(state.request_buffer.cursor_line, 0); // Cursor moves to top of visible area
    }
}
