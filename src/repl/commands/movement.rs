//! # Movement Commands
//!
//! This module contains command implementations for cursor movement in both
//! the Request and Response panes. These commands follow vim-style navigation.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::repl::{
    command::{CommandResult, CommandV2},
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

impl CommandV2 for MoveCursorLeftCommand {
    fn is_relevant(&self, event: KeyEvent, state: &AppState) -> bool {
        // Only relevant in Normal mode with h or Left arrow
        matches!(state.mode, EditorMode::Normal)
            && matches!(event.code, KeyCode::Char('h') | KeyCode::Left)
            && (matches!(event.code, KeyCode::Left) || event.modifiers == KeyModifiers::NONE)
    }

    fn process_detailed(&self, _event: KeyEvent, state: &mut AppState) -> Result<CommandResult> {
        match state.current_pane {
            Pane::Request => move_cursor_left(&mut state.request_buffer),
            Pane::Response => {
                if let Some(ref mut buffer) = state.response_buffer {
                    move_cursor_left(buffer)
                } else {
                    Ok(CommandResult::not_handled())
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

impl CommandV2 for MoveCursorRightCommand {
    fn is_relevant(&self, event: KeyEvent, state: &AppState) -> bool {
        // Only relevant in Normal mode with l or Right arrow
        matches!(state.mode, EditorMode::Normal)
            && matches!(event.code, KeyCode::Char('l') | KeyCode::Right)
            && (matches!(event.code, KeyCode::Right) || event.modifiers == KeyModifiers::NONE)
    }

    fn process_detailed(&self, _event: KeyEvent, state: &mut AppState) -> Result<CommandResult> {
        match state.current_pane {
            Pane::Request => move_cursor_right(&mut state.request_buffer),
            Pane::Response => {
                if let Some(ref mut buffer) = state.response_buffer {
                    move_cursor_right(buffer)
                } else {
                    Ok(CommandResult::not_handled())
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

impl CommandV2 for MoveCursorUpCommand {
    fn is_relevant(&self, event: KeyEvent, state: &AppState) -> bool {
        // Only relevant in Normal mode with k or Up arrow
        matches!(state.mode, EditorMode::Normal)
            && matches!(event.code, KeyCode::Char('k') | KeyCode::Up)
            && (matches!(event.code, KeyCode::Up) || event.modifiers == KeyModifiers::NONE)
    }

    fn process_detailed(&self, _event: KeyEvent, state: &mut AppState) -> Result<CommandResult> {
        match state.current_pane {
            Pane::Request => move_cursor_up(&mut state.request_buffer),
            Pane::Response => {
                if let Some(ref mut buffer) = state.response_buffer {
                    move_cursor_up(buffer)
                } else {
                    Ok(CommandResult::not_handled())
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

impl CommandV2 for MoveCursorDownCommand {
    fn is_relevant(&self, event: KeyEvent, state: &AppState) -> bool {
        // Only relevant in Normal mode with j or Down arrow
        matches!(state.mode, EditorMode::Normal)
            && matches!(event.code, KeyCode::Char('j') | KeyCode::Down)
            && (matches!(event.code, KeyCode::Down) || event.modifiers == KeyModifiers::NONE)
    }

    fn process_detailed(&self, _event: KeyEvent, state: &mut AppState) -> Result<CommandResult> {
        // Get visible heights before mutable borrows
        let request_visible_height = state.get_request_pane_height();
        let response_visible_height = state.get_response_pane_height();

        match state.current_pane {
            Pane::Request => move_cursor_down(&mut state.request_buffer, request_visible_height),
            Pane::Response => {
                if let Some(ref mut buffer) = state.response_buffer {
                    move_cursor_down(buffer, response_visible_height)
                } else {
                    Ok(CommandResult::not_handled())
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

impl CommandV2 for MoveCursorLineStartCommand {
    fn is_relevant(&self, event: KeyEvent, state: &AppState) -> bool {
        // Only relevant in Normal mode with '0' key and no modifiers
        matches!(state.mode, EditorMode::Normal)
            && matches!(event.code, KeyCode::Char('0'))
            && event.modifiers == KeyModifiers::NONE
    }

    fn process_detailed(&self, _event: KeyEvent, state: &mut AppState) -> Result<CommandResult> {
        match state.current_pane {
            Pane::Request => move_cursor_line_start(&mut state.request_buffer),
            Pane::Response => {
                if let Some(ref mut buffer) = state.response_buffer {
                    move_cursor_line_start(buffer)
                } else {
                    Ok(CommandResult::not_handled())
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

impl CommandV2 for MoveCursorLineEndCommand {
    fn is_relevant(&self, event: KeyEvent, state: &AppState) -> bool {
        // Only relevant in Normal mode with '$' key and no modifiers
        matches!(state.mode, EditorMode::Normal)
            && matches!(event.code, KeyCode::Char('$'))
            && event.modifiers == KeyModifiers::NONE
    }

    fn process_detailed(&self, _event: KeyEvent, state: &mut AppState) -> Result<CommandResult> {
        match state.current_pane {
            Pane::Request => move_cursor_line_end(&mut state.request_buffer),
            Pane::Response => {
                if let Some(ref mut buffer) = state.response_buffer {
                    move_cursor_line_end(buffer)
                } else {
                    Ok(CommandResult::not_handled())
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

impl CommandV2 for SwitchPaneCommand {
    fn is_relevant(&self, event: KeyEvent, state: &AppState) -> bool {
        // Only relevant in Normal mode for Ctrl+W sequences
        if !matches!(state.mode, EditorMode::Normal) {
            return false;
        }

        // First step: Ctrl+W
        if event.modifiers.contains(KeyModifiers::CONTROL)
            && matches!(event.code, KeyCode::Char('w'))
        {
            return true;
        }

        // Second step: commands after Ctrl+W
        if state.pending_ctrl_w && matches!(event.code, KeyCode::Char('w') | KeyCode::Esc) {
            return true;
        }

        // Invalid second step (for error handling)
        state.pending_ctrl_w
    }

    fn process_detailed(&self, event: KeyEvent, state: &mut AppState) -> Result<CommandResult> {
        // Handle Ctrl+W (first step)
        if event.modifiers.contains(KeyModifiers::CONTROL)
            && matches!(event.code, KeyCode::Char('w'))
        {
            state.pending_ctrl_w = true;
            return Ok(CommandResult {
                handled: true,
                content_changed: false,
                cursor_moved: false,
                mode_changed: false,
                pane_changed: false,
                scroll_occurred: false,
                status_message: None,
            });
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
                    return Ok(CommandResult::cursor_moved().with_pane_change());
                }
                KeyCode::Esc => {
                    // Cancel Ctrl+W command
                    state.pending_ctrl_w = false;
                    return Ok(CommandResult {
                        handled: true,
                        content_changed: false,
                        cursor_moved: false,
                        mode_changed: false,
                        pane_changed: false,
                        scroll_occurred: false,
                        status_message: None,
                    });
                }
                _ => {
                    // Invalid Ctrl+W command
                    state.pending_ctrl_w = false;
                    return Ok(CommandResult {
                        handled: true,
                        content_changed: false,
                        cursor_moved: false,
                        mode_changed: false,
                        pane_changed: false,
                        scroll_occurred: false,
                        status_message: Some("Invalid window command".to_string()),
                    });
                }
            }
        }

        Ok(CommandResult::not_handled())
    }

    fn name(&self) -> &'static str {
        "SwitchPane"
    }
}
