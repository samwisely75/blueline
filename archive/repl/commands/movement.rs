/// Move cursor to the start of the next word in the current line or next line
/// Handles auto-scrolling when moving to a line outside the visible area
fn move_to_next_word<T: MovementBuffer>(
    buffer: &mut T,
    visible_height: usize,
) -> Result<CommandResult> {
    let mut line_idx = buffer.cursor_line();
    let mut col_idx = *buffer.cursor_col_mut();
    let lines = buffer.lines();
    let mut first = true;
    let mut scroll_occurred = false;

    loop {
        if line_idx >= lines.len() {
            break;
        }
        let line = &lines[line_idx];
        let chars: Vec<char> = line.chars().collect();
        let len = chars.len();
        let mut i = col_idx;

        // On first iteration, always move at least one character forward if possible
        if first && i < len {
            i += 1;
        }

        // If we're in a word, skip to the end of it
        while i < len && chars[i].is_alphanumeric() {
            i += 1;
        }

        // Skip non-word characters
        while i < len && !chars[i].is_alphanumeric() {
            i += 1;
        }

        // If we found a word character, we're at the start of the next word
        if i < len && chars[i].is_alphanumeric() {
            *buffer.cursor_line_mut() = line_idx;
            *buffer.cursor_col_mut() = i;

            // Auto-scroll if cursor moved outside visible area
            if line_idx < *buffer.scroll_offset_mut() {
                *buffer.scroll_offset_mut() = line_idx;
                scroll_occurred = true;
            } else if line_idx >= *buffer.scroll_offset_mut() + visible_height {
                *buffer.scroll_offset_mut() = line_idx - visible_height + 1;
                scroll_occurred = true;
            }

            let mut result = CommandResult::cursor_moved();
            if scroll_occurred {
                result = result.with_scroll();
            }
            return Ok(result);
        } else {
            // Go to next line and start from beginning
            line_idx += 1;
            col_idx = 0;

            // Check if next line exists and starts with a word
            if line_idx < lines.len() {
                let next_line = &lines[line_idx];
                let next_chars: Vec<char> = next_line.chars().collect();
                if !next_chars.is_empty() && next_chars[0].is_alphanumeric() {
                    // Found a word at the start of the next line
                    *buffer.cursor_line_mut() = line_idx;
                    *buffer.cursor_col_mut() = 0;

                    // Auto-scroll if cursor moved outside visible area
                    if line_idx < *buffer.scroll_offset_mut() {
                        *buffer.scroll_offset_mut() = line_idx;
                        scroll_occurred = true;
                    } else if line_idx >= *buffer.scroll_offset_mut() + visible_height {
                        *buffer.scroll_offset_mut() = line_idx - visible_height + 1;
                        scroll_occurred = true;
                    }

                    let mut result = CommandResult::cursor_moved();
                    if scroll_occurred {
                        result = result.with_scroll();
                    }
                    return Ok(result);
                }
            }
            first = false; // Don't move forward on subsequent lines
        }
    }

    Ok(CommandResult::cursor_moved()) // Always return cursor_moved for feedback
}
/// Command for moving cursor to next word (w)
pub struct MoveToNextWordCommand;

impl MoveToNextWordCommand {}

impl Command for MoveToNextWordCommand {
    fn is_relevant(&self, state: &AppState, event: &KeyEvent) -> bool {
        matches!(state.mode, EditorMode::Normal)
            && matches!(event.code, KeyCode::Char('w'))
            && event.modifiers == KeyModifiers::NONE
            && !state.pending_ctrl_w // Don't intercept 'w' when Ctrl+W is pending
    }

    fn process(&self, _event: KeyEvent, state: &mut AppState) -> Result<bool> {
        // Get visible heights before mutable borrows
        let request_visible_height = state.get_request_pane_height();
        let response_visible_height = state.get_response_pane_height();

        match state.current_pane {
            Pane::Request => {
                move_to_next_word(&mut state.request_buffer, request_visible_height)?;
                Ok(true)
            }
            Pane::Response => {
                if let Some(ref mut buffer) = state.response_buffer {
                    move_to_next_word(buffer, response_visible_height)?;
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
        }
    }

    fn name(&self) -> &'static str {
        "MoveToNextWord"
    }
}

/// # Movement Commands
///
/// This module contains command implementations for cursor movement in both
/// the Request and Response panes. These commands follow vim-style navigation
/// with display-line-aware movement for wrapped text.
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::repl::commands::MvvmCommand;
use crate::repl::{
    commands::{Command, CommandResult},
    model::{AppState, EditorMode, Pane, RequestBuffer, ResponseBuffer},
    view_model::ViewModel,
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

/// Move cursor up by one display line, handling scroll and column adjustment
/// Uses display cache for proper wrapped text navigation
fn move_cursor_up_display_aware(state: &mut AppState) -> Result<CommandResult> {
    let cache = match state.current_pane {
        Pane::Request => state.cache_manager.get_request_cache(),
        Pane::Response => state.cache_manager.get_response_cache(),
    };

    // Get current logical position
    let (current_logical_line, current_logical_col) = match state.current_pane {
        Pane::Request => (
            state.request_buffer.cursor_line,
            state.request_buffer.cursor_col,
        ),
        Pane::Response => {
            if let Some(ref buffer) = state.response_buffer {
                (buffer.cursor_line, buffer.cursor_col)
            } else {
                return Ok(CommandResult::not_handled());
            }
        }
    };

    // Convert to display position
    if let Some((current_display_line, current_display_col)) =
        cache.logical_to_display_position(current_logical_line, current_logical_col)
    {
        // Try to move up one display line
        if let Some((new_display_line, new_display_col)) =
            cache.move_up(current_display_line, current_display_col)
        {
            // Convert back to logical position
            if let Some((new_logical_line, new_logical_col)) =
                cache.display_to_logical_position(new_display_line, new_display_col)
            {
                // Update cursor position and handle display-line-aware scrolling
                match state.current_pane {
                    Pane::Request => {
                        state.request_buffer.cursor_line = new_logical_line;
                        state.request_buffer.cursor_col = new_logical_col;

                        // Auto-scroll up if new display line goes above visible area
                        let scroll_offset_display = if let Some((scroll_display_line, _)) =
                            cache.logical_to_display_position(state.request_buffer.scroll_offset, 0)
                        {
                            scroll_display_line
                        } else {
                            state.request_buffer.scroll_offset
                        };

                        if new_display_line < scroll_offset_display {
                            // Find logical line that contains the target display line
                            if let Some((target_logical_line, _)) =
                                cache.display_to_logical_position(new_display_line, 0)
                            {
                                state.request_buffer.scroll_offset = target_logical_line;
                            }
                            return Ok(CommandResult::cursor_moved().with_scroll());
                        }
                    }
                    Pane::Response => {
                        if let Some(ref mut buffer) = state.response_buffer {
                            buffer.cursor_line = new_logical_line;
                            buffer.cursor_col = new_logical_col;

                            // Auto-scroll up if new display line goes above visible area
                            let scroll_offset_display = if let Some((scroll_display_line, _)) =
                                cache.logical_to_display_position(buffer.scroll_offset, 0)
                            {
                                scroll_display_line
                            } else {
                                buffer.scroll_offset
                            };

                            if new_display_line < scroll_offset_display {
                                // Find logical line that contains the target display line
                                if let Some((target_logical_line, _)) =
                                    cache.display_to_logical_position(new_display_line, 0)
                                {
                                    buffer.scroll_offset = target_logical_line;
                                }
                                return Ok(CommandResult::cursor_moved().with_scroll());
                            }
                        }
                    }
                }
                return Ok(CommandResult::cursor_moved());
            }
        }
    }

    // Fallback to logical line movement if cache is not available
    move_cursor_up_fallback(state)
}

/// Fallback movement when display cache is not available
fn move_cursor_up_fallback(state: &mut AppState) -> Result<CommandResult> {
    match state.current_pane {
        Pane::Request => {
            let buffer = &mut state.request_buffer;
            if buffer.cursor_line > 0 {
                buffer.cursor_line -= 1;
                let line_len = buffer.lines.get(buffer.cursor_line).map_or(0, |l| l.len());
                buffer.cursor_col = buffer.cursor_col.min(line_len);

                if buffer.cursor_line < buffer.scroll_offset {
                    buffer.scroll_offset = buffer.cursor_line;
                    return Ok(CommandResult::cursor_moved().with_scroll());
                }
                Ok(CommandResult::cursor_moved())
            } else {
                Ok(CommandResult::not_handled())
            }
        }
        Pane::Response => {
            if let Some(ref mut buffer) = state.response_buffer {
                if buffer.cursor_line > 0 {
                    buffer.cursor_line -= 1;
                    let line_len = buffer.lines.get(buffer.cursor_line).map_or(0, |l| l.len());
                    buffer.cursor_col = buffer.cursor_col.min(line_len);

                    if buffer.cursor_line < buffer.scroll_offset {
                        buffer.scroll_offset = buffer.cursor_line;
                        return Ok(CommandResult::cursor_moved().with_scroll());
                    }
                    Ok(CommandResult::cursor_moved())
                } else {
                    Ok(CommandResult::not_handled())
                }
            } else {
                Ok(CommandResult::not_handled())
            }
        }
    }
}

/// Move cursor down by one display line, handling scroll and column adjustment
/// Works purely in display space - tells buffer about segments to display and cursor position
fn move_cursor_down_display_aware(
    state: &mut AppState,
    visible_height: usize,
) -> Result<CommandResult> {
    let cache = match state.current_pane {
        Pane::Request => state.cache_manager.get_request_cache(),
        Pane::Response => state.cache_manager.get_response_cache(),
    };

    // Work purely in display space
    let (current_display_cursor, current_display_scroll) = match state.current_pane {
        Pane::Request => (
            (
                state.request_buffer.display_cursor_line,
                state.request_buffer.display_cursor_col,
            ),
            state.request_buffer.display_scroll_offset,
        ),
        Pane::Response => {
            if let Some(ref buffer) = state.response_buffer {
                (
                    (buffer.display_cursor_line, buffer.display_cursor_col),
                    buffer.display_scroll_offset,
                )
            } else {
                return Ok(CommandResult::not_handled());
            }
        }
    };

    let (current_display_line, current_display_col) = current_display_cursor;

    // Try to move down one display line in display space
    if let Some((new_display_line, new_display_col)) =
        cache.move_down(current_display_line, current_display_col)
    {
        // Calculate where cursor would be positioned in terminal coordinates
        let cursor_terminal_position = new_display_line - current_display_scroll;

        // Determine if scrolling is needed
        let (final_display_scroll, scroll_occurred) = if cursor_terminal_position >= visible_height
        {
            // Cursor would go beyond visible area - scroll down to keep it visible
            let new_scroll = new_display_line - visible_height + 1;
            (new_scroll, true)
        } else {
            // No scrolling needed
            (current_display_scroll, false)
        };

        // Update buffer with new display positions (logical positions derived automatically)
        match state.current_pane {
            Pane::Request => {
                state
                    .request_buffer
                    .set_display_cursor(new_display_line, new_display_col, &cache);
                state
                    .request_buffer
                    .set_display_scroll(final_display_scroll, &cache);
            }
            Pane::Response => {
                if let Some(ref mut buffer) = state.response_buffer {
                    buffer.set_display_cursor(new_display_line, new_display_col, &cache);
                    buffer.set_display_scroll(final_display_scroll, &cache);
                }
            }
        }

        // Return result indicating movement with optional scroll
        if scroll_occurred {
            Ok(CommandResult::cursor_moved().with_scroll())
        } else {
            Ok(CommandResult::cursor_moved())
        }
    } else {
        // At last display line - check if we can scroll to reveal more content
        let total_display_lines = cache.display_lines.len();
        let last_visible_display = current_display_scroll + visible_height - 1;

        if last_visible_display < total_display_lines.saturating_sub(1) {
            // There are more display lines below - scroll down
            let new_display_scroll = current_display_scroll + 1;

            // Update buffer with new scroll position
            match state.current_pane {
                Pane::Request => {
                    state
                        .request_buffer
                        .set_display_scroll(new_display_scroll, &cache);
                }
                Pane::Response => {
                    if let Some(ref mut buffer) = state.response_buffer {
                        buffer.set_display_scroll(new_display_scroll, &cache);
                    }
                }
            }

            Ok(CommandResult::cursor_moved().with_scroll())
        } else {
            // No more content to scroll - no movement
            Ok(CommandResult::not_handled())
        }
    }
}

/// Fallback movement when display cache is not available
fn move_cursor_down_fallback(state: &mut AppState, visible_height: usize) -> Result<CommandResult> {
    match state.current_pane {
        Pane::Request => {
            let buffer = &mut state.request_buffer;
            if buffer.cursor_line < buffer.lines.len().saturating_sub(1) {
                buffer.cursor_line += 1;
                let line_len = buffer.lines.get(buffer.cursor_line).map_or(0, |l| l.len());
                buffer.cursor_col = buffer.cursor_col.min(line_len);

                if buffer.cursor_line >= buffer.scroll_offset + visible_height {
                    buffer.scroll_offset = buffer.cursor_line - visible_height + 1;

                    // Additional bounds check: ensure cursor position in terminal is valid
                    let cursor_pos_in_terminal = buffer.cursor_line - buffer.scroll_offset;
                    if cursor_pos_in_terminal >= visible_height {
                        // Cursor would be beyond terminal bounds - adjust scroll
                        buffer.scroll_offset =
                            buffer.cursor_line.saturating_sub(visible_height - 1);
                    }

                    return Ok(CommandResult::cursor_moved().with_scroll());
                }
                Ok(CommandResult::cursor_moved())
            } else {
                Ok(CommandResult::not_handled())
            }
        }
        Pane::Response => {
            if let Some(ref mut buffer) = state.response_buffer {
                if buffer.cursor_line < buffer.lines.len().saturating_sub(1) {
                    buffer.cursor_line += 1;
                    let line_len = buffer.lines.get(buffer.cursor_line).map_or(0, |l| l.len());
                    buffer.cursor_col = buffer.cursor_col.min(line_len);

                    if buffer.cursor_line >= buffer.scroll_offset + visible_height {
                        buffer.scroll_offset = buffer.cursor_line - visible_height + 1;

                        // Additional bounds check: ensure cursor position in terminal is valid
                        let cursor_pos_in_terminal = buffer.cursor_line - buffer.scroll_offset;
                        if cursor_pos_in_terminal >= visible_height {
                            // Cursor would be beyond terminal bounds - adjust scroll
                            buffer.scroll_offset =
                                buffer.cursor_line.saturating_sub(visible_height - 1);
                        }

                        return Ok(CommandResult::cursor_moved().with_scroll());
                    }
                    Ok(CommandResult::cursor_moved())
                } else {
                    Ok(CommandResult::not_handled())
                }
            } else {
                Ok(CommandResult::not_handled())
            }
        }
    }
}

/// Move cursor left by one column with display-line wrapping awareness
/// Moves to previous display line when at beginning of wrapped segment
fn move_cursor_left_display_aware(state: &mut AppState) -> Result<CommandResult> {
    match state.current_pane {
        Pane::Request => {
            let buffer = &mut state.request_buffer;
            if buffer.cursor_col > 0 {
                buffer.cursor_col -= 1;
                Ok(CommandResult::cursor_moved())
            } else {
                // At beginning of line - try to wrap to previous display line
                let cache = state.cache_manager.get_request_cache();
                if let Some((current_display_line, _)) =
                    cache.logical_to_display_position(buffer.cursor_line, buffer.cursor_col)
                {
                    if current_display_line > 0 {
                        // Move to end of previous display line
                        let prev_display_line = current_display_line - 1;
                        if let Some(display_info) = cache.get_display_line(prev_display_line) {
                            if let Some((new_logical_line, new_logical_col)) = cache
                                .display_to_logical_position(
                                    prev_display_line,
                                    display_info.content.chars().count(),
                                )
                            {
                                buffer.cursor_line = new_logical_line;
                                buffer.cursor_col = new_logical_col;
                                return Ok(CommandResult::cursor_moved());
                            }
                        }
                    }
                }
                Ok(CommandResult::not_handled())
            }
        }
        Pane::Response => {
            if let Some(ref mut buffer) = state.response_buffer {
                if buffer.cursor_col > 0 {
                    buffer.cursor_col -= 1;
                    Ok(CommandResult::cursor_moved())
                } else {
                    // At beginning of line - try to wrap to previous display line
                    let cache = state.cache_manager.get_response_cache();
                    if let Some((current_display_line, _)) =
                        cache.logical_to_display_position(buffer.cursor_line, buffer.cursor_col)
                    {
                        if current_display_line > 0 {
                            // Move to end of previous display line
                            let prev_display_line = current_display_line - 1;
                            if let Some(display_info) = cache.get_display_line(prev_display_line) {
                                if let Some((new_logical_line, new_logical_col)) = cache
                                    .display_to_logical_position(
                                        prev_display_line,
                                        display_info.content.chars().count(),
                                    )
                                {
                                    buffer.cursor_line = new_logical_line;
                                    buffer.cursor_col = new_logical_col;
                                    return Ok(CommandResult::cursor_moved());
                                }
                            }
                        }
                    }
                    Ok(CommandResult::not_handled())
                }
            } else {
                Ok(CommandResult::not_handled())
            }
        }
    }
}

/// Move cursor right by one column with display-line wrapping awareness
/// Moves to next display line when at end of wrapped segment
fn move_cursor_right_display_aware(state: &mut AppState) -> Result<CommandResult> {
    match state.current_pane {
        Pane::Request => {
            let buffer = &mut state.request_buffer;
            let current_line = buffer.lines.get(buffer.cursor_line);
            if let Some(line) = current_line {
                if buffer.cursor_col < line.len() {
                    buffer.cursor_col += 1;
                    Ok(CommandResult::cursor_moved())
                } else {
                    // At end of logical line - try to wrap to next display line
                    let cache = state.cache_manager.get_request_cache();
                    if let Some((current_display_line, _)) =
                        cache.logical_to_display_position(buffer.cursor_line, buffer.cursor_col)
                    {
                        if current_display_line < cache.display_lines.len().saturating_sub(1) {
                            // Move to beginning of next display line
                            let next_display_line = current_display_line + 1;
                            if let Some((new_logical_line, new_logical_col)) =
                                cache.display_to_logical_position(next_display_line, 0)
                            {
                                buffer.cursor_line = new_logical_line;
                                buffer.cursor_col = new_logical_col;
                                return Ok(CommandResult::cursor_moved());
                            }
                        }
                    }
                    Ok(CommandResult::not_handled())
                }
            } else {
                Ok(CommandResult::not_handled())
            }
        }
        Pane::Response => {
            if let Some(ref mut buffer) = state.response_buffer {
                let current_line = buffer.lines.get(buffer.cursor_line);
                if let Some(line) = current_line {
                    if buffer.cursor_col < line.len() {
                        buffer.cursor_col += 1;
                        Ok(CommandResult::cursor_moved())
                    } else {
                        // At end of logical line - try to wrap to next display line
                        let cache = state.cache_manager.get_response_cache();
                        if let Some((current_display_line, _)) =
                            cache.logical_to_display_position(buffer.cursor_line, buffer.cursor_col)
                        {
                            if current_display_line < cache.display_lines.len().saturating_sub(1) {
                                // Move to beginning of next display line
                                let next_display_line = current_display_line + 1;
                                if let Some((new_logical_line, new_logical_col)) =
                                    cache.display_to_logical_position(next_display_line, 0)
                                {
                                    buffer.cursor_line = new_logical_line;
                                    buffer.cursor_col = new_logical_col;
                                    return Ok(CommandResult::cursor_moved());
                                }
                            }
                        }
                        Ok(CommandResult::not_handled())
                    }
                } else {
                    Ok(CommandResult::not_handled())
                }
            } else {
                Ok(CommandResult::not_handled())
            }
        }
    }
}

/// Move cursor to start of current display line (vim-style 0)
fn move_cursor_line_start<T: MovementBuffer>(buffer: &mut T) -> Result<CommandResult> {
    *buffer.cursor_col_mut() = 0;
    Ok(CommandResult::cursor_moved())
}

/// Move cursor to end of current display line (vim-style $)
fn move_cursor_line_end<T: MovementBuffer>(buffer: &mut T) -> Result<CommandResult> {
    let cursor_line = buffer.cursor_line();
    if let Some(line) = buffer.lines().get(cursor_line) {
        *buffer.cursor_col_mut() = line.len();
    }
    Ok(CommandResult::cursor_moved())
}

/// Command for moving cursor left (h, Left arrow)
pub struct MoveCursorLeftCommand;

impl MoveCursorLeftCommand {}

impl Command for MoveCursorLeftCommand {
    fn is_relevant(&self, state: &AppState, event: &KeyEvent) -> bool {
        // Allow Left arrow in any mode, but 'h' only in Normal mode
        match event.code {
            KeyCode::Char('h') => {
                matches!(state.mode, EditorMode::Normal) && event.modifiers == KeyModifiers::NONE
            }
            KeyCode::Left => true,
            _ => false,
        }
    }

    fn process(&self, _event: KeyEvent, state: &mut AppState) -> Result<bool> {
        match state.current_pane {
            Pane::Request => {
                move_cursor_left_display_aware(state)?;
                Ok(true)
            }
            Pane::Response => {
                move_cursor_left_display_aware(state)?;
                Ok(true)
            }
        }
    }

    fn name(&self) -> &'static str {
        "MoveCursorLeft"
    }
}

// MVVM Implementation - NEW!
impl MvvmCommand for MoveCursorLeftCommand {
    fn is_relevant(&self, view_model: &ViewModel, event: &KeyEvent) -> bool {
        // Allow Left arrow in any mode, but 'h' only in Normal mode
        match event.code {
            KeyCode::Char('h') => {
                matches!(view_model.editor.mode, EditorMode::Normal) && event.modifiers == KeyModifiers::NONE
            }
            KeyCode::Left => true,
            _ => false,
        }
    }

    fn execute(&self, _event: KeyEvent, view_model: &mut ViewModel) -> Result<bool> {
        // Delegate to ViewModel - all the complex logic is already implemented!
        view_model.move_cursor_left()?;
        Ok(true)
    }

    fn name(&self) -> &'static str {
        "MoveCursorLeft"
    }
}

/// Command for moving cursor right (l, Right arrow)
pub struct MoveCursorRightCommand;

impl MoveCursorRightCommand {}

impl Command for MoveCursorRightCommand {
    fn is_relevant(&self, state: &AppState, event: &KeyEvent) -> bool {
        // Allow Right arrow in any mode, but 'l' only in Normal mode
        match event.code {
            KeyCode::Char('l') => {
                matches!(state.mode, EditorMode::Normal) && event.modifiers == KeyModifiers::NONE
            }
            KeyCode::Right => true,
            _ => false,
        }
    }

    fn process(&self, _event: KeyEvent, state: &mut AppState) -> Result<bool> {
        match state.current_pane {
            Pane::Request => {
                move_cursor_right_display_aware(state)?;
                Ok(true)
            }
            Pane::Response => {
                move_cursor_right_display_aware(state)?;
                Ok(true)
            }
        }
    }

    fn name(&self) -> &'static str {
        "MoveCursorRight"
    }
}

/// Command for moving cursor up (k, Up arrow)
pub struct MoveCursorUpCommand;

impl MoveCursorUpCommand {}

impl Command for MoveCursorUpCommand {
    fn is_relevant(&self, state: &AppState, event: &KeyEvent) -> bool {
        // Allow Up arrow in any mode, but 'k' only in Normal mode
        match event.code {
            KeyCode::Char('k') => {
                matches!(state.mode, EditorMode::Normal) && event.modifiers == KeyModifiers::NONE
            }
            KeyCode::Up => true,
            _ => false,
        }
    }

    fn process(&self, _event: KeyEvent, state: &mut AppState) -> Result<bool> {
        move_cursor_up_display_aware(state)?;
        Ok(true)
    }

    fn name(&self) -> &'static str {
        "MoveCursorUp"
    }
}

/// Command for moving cursor down (j, Down arrow)
pub struct MoveCursorDownCommand;

impl MoveCursorDownCommand {}

impl Command for MoveCursorDownCommand {
    fn is_relevant(&self, state: &AppState, event: &KeyEvent) -> bool {
        // Allow Down arrow in any mode, but 'j' only in Normal mode
        match event.code {
            KeyCode::Char('j') => {
                matches!(state.mode, EditorMode::Normal) && event.modifiers == KeyModifiers::NONE
            }
            KeyCode::Down => true,
            _ => false,
        }
    }

    fn process(&self, _event: KeyEvent, state: &mut AppState) -> Result<bool> {
        // Get visible heights before mutable borrows
        let request_visible_height = state.get_request_pane_height();
        let response_visible_height = state.get_response_pane_height();

        let visible_height = match state.current_pane {
            Pane::Request => request_visible_height,
            Pane::Response => response_visible_height,
        };

        let result = move_cursor_down_display_aware(state, visible_height)?;
        Ok(result.handled)
    }

    fn name(&self) -> &'static str {
        "MoveCursorDown"
    }
}

/// Command for moving cursor to line start (0)
pub struct MoveCursorLineStartCommand;

impl MoveCursorLineStartCommand {}

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

impl MoveCursorLineEndCommand {}

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

/// Command for scrolling up half a page (Ctrl+U)
pub struct ScrollHalfPageUpCommand;

impl ScrollHalfPageUpCommand {}

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

impl ScrollHalfPageDownCommand {}

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

impl ScrollFullPageDownCommand {}

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

impl ScrollFullPageUpCommand {}

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

/// Command for going to the first line (gg in vim)
/// This command handles the double 'g' sequence in Normal mode
pub struct GoToTopCommand;

impl GoToTopCommand {}

impl Command for GoToTopCommand {
    fn is_relevant(&self, state: &AppState, event: &KeyEvent) -> bool {
        // Only relevant in Normal mode for second 'g' when first 'g' is pending
        matches!(state.mode, EditorMode::Normal)
            && state.pending_g
            && event.modifiers == KeyModifiers::NONE
            && matches!(event.code, KeyCode::Char('g'))
    }

    fn process(&self, _event: KeyEvent, state: &mut AppState) -> Result<bool> {
        // Clear the pending state first
        state.pending_g = false;

        match state.current_pane {
            Pane::Request => {
                go_to_top(&mut state.request_buffer);
                Ok(true)
            }
            Pane::Response => {
                if let Some(ref mut buffer) = state.response_buffer {
                    go_to_top(buffer);
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
        }
    }

    fn name(&self) -> &'static str {
        "GoToTop"
    }
}

/// Command for going to the last line (G in vim)
pub struct GoToBottomCommand;

impl GoToBottomCommand {}

impl Command for GoToBottomCommand {
    fn is_relevant(&self, state: &AppState, event: &KeyEvent) -> bool {
        // Only relevant in Normal mode for 'G' key (accept both NONE and SHIFT modifiers)
        matches!(state.mode, EditorMode::Normal)
            && (event.modifiers == KeyModifiers::NONE || event.modifiers == KeyModifiers::SHIFT)
            && matches!(event.code, KeyCode::Char('G'))
    }

    fn process(&self, _event: KeyEvent, state: &mut AppState) -> Result<bool> {
        match state.current_pane {
            Pane::Request => {
                let pane_height = state.get_request_pane_height();
                go_to_bottom(&mut state.request_buffer, pane_height);
                Ok(true)
            }
            Pane::Response => {
                let pane_height = state.get_response_pane_height();
                if let Some(ref mut buffer) = state.response_buffer {
                    go_to_bottom(buffer, pane_height);
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
        }
    }

    fn name(&self) -> &'static str {
        "GoToBottom"
    }
}

/// Command for setting pending 'g' state (first 'g' in gg sequence)
pub struct SetPendingGCommand;

impl SetPendingGCommand {}

impl Command for SetPendingGCommand {
    fn is_relevant(&self, state: &AppState, event: &KeyEvent) -> bool {
        // Only relevant in Normal mode for first 'g' when no pending state
        matches!(state.mode, EditorMode::Normal)
            && !state.pending_g
            && event.modifiers == KeyModifiers::NONE
            && matches!(event.code, KeyCode::Char('g'))
    }

    fn process(&self, _event: KeyEvent, state: &mut AppState) -> Result<bool> {
        // Set pending state and wait for second 'g'
        state.pending_g = true;
        Ok(true)
    }

    fn name(&self) -> &'static str {
        "SetPendingG"
    }
}

/// Move cursor to the first line (line 0) and column 0
/// Adjusts scroll offset to ensure the first line is visible
fn go_to_top<T: MovementBuffer>(buffer: &mut T) {
    *buffer.cursor_line_mut() = 0;
    *buffer.cursor_col_mut() = 0;
    *buffer.scroll_offset_mut() = 0;
}

/// Move cursor to the last line and column 0
/// Adjusts scroll offset to ensure the last line is visible
fn go_to_bottom<T: MovementBuffer>(buffer: &mut T, pane_height: usize) {
    let line_count = buffer.lines().len();
    if line_count > 0 {
        let last_line_idx = line_count - 1;

        *buffer.cursor_line_mut() = last_line_idx;
        *buffer.cursor_col_mut() = 0;

        // Adjust scroll offset to show the last line at the bottom of the visible area
        if line_count > pane_height {
            *buffer.scroll_offset_mut() = line_count - pane_height;
        } else {
            *buffer.scroll_offset_mut() = 0;
        }
    }
}

// Legacy movement functions for backward compatibility with tests
#[cfg(test)]
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

#[cfg(test)]
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

#[cfg(test)]
fn move_cursor_left<T: MovementBuffer>(buffer: &mut T) -> Result<CommandResult> {
    if *buffer.cursor_col_mut() > 0 {
        *buffer.cursor_col_mut() -= 1;
        Ok(CommandResult::cursor_moved())
    } else {
        Ok(CommandResult::not_handled())
    }
}

#[cfg(test)]
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
            display_cursor_line: 0,
            display_cursor_col: 0,
            display_scroll_offset: 0,
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
    fn mvvm_move_cursor_left_command_should_work() {
        use crate::repl::commands::MvvmCommand;
        use crate::repl::view_model::ViewModel;
        
        let command = MoveCursorLeftCommand;
        let mut view_model = ViewModel::new();
        
        // Add some content to move within
        let _ = view_model.request_buffer.content_mut().insert_text(
            Pane::Request,
            crate::repl::events::LogicalPosition { line: 0, column: 0 },
            "test content",
        );
        
        // Move cursor to position 5
        let _ = view_model.editor.set_cursor(
            Pane::Request, 
            crate::repl::events::LogicalPosition { line: 0, column: 5 }
        );
        
        // Test that command is relevant for 'h' key in Normal mode
        let event = KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE);
        assert!(MvvmCommand::is_relevant(&command, &view_model, &event));
        
        // Execute the command
        let result = MvvmCommand::execute(&command, event, &mut view_model);
        assert!(result.is_ok());
        assert!(result.unwrap());
        
        // Verify cursor moved left
        let cursor_pos = view_model.get_cursor_position();
        assert_eq!(cursor_pos.column, 4); // Should have moved from 5 to 4
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
        let command = MoveCursorLeftCommand;
        let state = create_test_app_state();
        let event = KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE);

        assert!(Command::is_relevant(&command, &state, &event));
        assert_eq!(Command::name(&command), "MoveCursorLeft");
    }

    #[test]
    fn move_cursor_left_command_should_be_relevant_for_left_arrow_in_normal_mode() {
        let command = MoveCursorLeftCommand;
        let state = create_test_app_state();
        let event = KeyEvent::new(KeyCode::Left, KeyModifiers::NONE);

        assert!(Command::is_relevant(&command, &state, &event));
    }

    #[test]
    fn move_cursor_left_command_should_not_be_relevant_in_insert_mode() {
        let command = MoveCursorLeftCommand;
        let mut state = create_test_app_state();
        state.mode = EditorMode::Insert;
        let event = KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE);

        assert!(!Command::is_relevant(&command, &state, &event));
    }

    #[test]
    fn move_cursor_right_command_should_be_relevant_for_l_key_in_normal_mode() {
        let command = MoveCursorRightCommand;
        let state = create_test_app_state();
        let event = KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE);

        assert!(Command::is_relevant(&command, &state, &event));
        assert_eq!(command.name(), "MoveCursorRight");
    }

    #[test]
    fn move_cursor_up_command_should_be_relevant_for_k_key_in_normal_mode() {
        let command = MoveCursorUpCommand;
        let state = create_test_app_state();
        let event = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);

        assert!(Command::is_relevant(&command, &state, &event));
        assert_eq!(command.name(), "MoveCursorUp");
    }

    #[test]
    fn move_cursor_down_command_should_be_relevant_for_j_key_in_normal_mode() {
        let command = MoveCursorDownCommand;
        let state = create_test_app_state();
        let event = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);

        assert!(Command::is_relevant(&command, &state, &event));
        assert_eq!(command.name(), "MoveCursorDown");
    }

    #[test]
    fn move_cursor_line_start_command_should_be_relevant_for_zero_key_in_normal_mode() {
        let command = MoveCursorLineStartCommand;
        let state = create_test_app_state();
        let event = KeyEvent::new(KeyCode::Char('0'), KeyModifiers::NONE);

        assert!(Command::is_relevant(&command, &state, &event));
        assert_eq!(command.name(), "MoveCursorLineStart");
    }

    #[test]
    fn move_cursor_line_end_command_should_be_relevant_for_dollar_key_in_normal_mode() {
        let command = MoveCursorLineEndCommand;
        let state = create_test_app_state();
        let event = KeyEvent::new(KeyCode::Char('$'), KeyModifiers::NONE);

        assert!(Command::is_relevant(&command, &state, &event));
        assert_eq!(command.name(), "MoveCursorLineEnd");
    }

    #[test]
    fn scroll_half_page_up_command_should_be_relevant_for_ctrl_u_in_normal_mode() {
        let command = ScrollHalfPageUpCommand;
        let state = create_test_app_state();
        let event = KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL);

        assert!(Command::is_relevant(&command, &state, &event));
        assert_eq!(command.name(), "ScrollHalfPageUp");
    }

    #[test]
    fn scroll_half_page_up_command_should_not_be_relevant_in_insert_mode() {
        let command = ScrollHalfPageUpCommand;
        let mut state = create_test_app_state();
        state.mode = EditorMode::Insert;
        let event = KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL);

        assert!(!Command::is_relevant(&command, &state, &event));
    }

    #[test]
    fn scroll_half_page_up_command_should_not_be_relevant_without_ctrl() {
        let command = ScrollHalfPageUpCommand;
        let state = create_test_app_state();
        let event = KeyEvent::new(KeyCode::Char('u'), KeyModifiers::NONE);

        assert!(!Command::is_relevant(&command, &state, &event));
    }

    #[test]
    fn scroll_half_page_up_command_should_scroll_request_buffer() {
        let command = ScrollHalfPageUpCommand;
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
        let command = ScrollHalfPageUpCommand;
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
        let command = ScrollHalfPageDownCommand;
        let state = AppState::new((80, 24), true);
        let event = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL);

        assert!(Command::is_relevant(&command, &state, &event));
    }

    #[test]
    fn scroll_half_page_down_command_should_not_be_relevant_in_insert_mode() {
        let command = ScrollHalfPageDownCommand;
        let mut state = AppState::new((80, 24), true);
        state.mode = EditorMode::Insert;
        let event = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL);

        assert!(!Command::is_relevant(&command, &state, &event));
    }

    #[test]
    fn scroll_half_page_down_command_should_not_be_relevant_without_ctrl() {
        let command = ScrollHalfPageDownCommand;
        let state = AppState::new((80, 24), true);
        let event = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE);

        assert!(!Command::is_relevant(&command, &state, &event));
    }

    #[test]
    fn scroll_half_page_down_command_should_scroll_request_buffer() {
        let command = ScrollHalfPageDownCommand;
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
        let command = ScrollHalfPageDownCommand;
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
        let command = ScrollFullPageDownCommand;
        let state = AppState::new((80, 24), true);
        let event = KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL);

        assert!(Command::is_relevant(&command, &state, &event));
        assert_eq!(command.name(), "ScrollFullPageDown");
    }

    #[test]
    fn scroll_full_page_down_command_should_not_be_relevant_in_insert_mode() {
        let command = ScrollFullPageDownCommand;
        let mut state = AppState::new((80, 24), true);
        state.mode = EditorMode::Insert;
        let event = KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL);

        assert!(!Command::is_relevant(&command, &state, &event));
    }

    #[test]
    fn scroll_full_page_down_command_should_not_be_relevant_without_ctrl() {
        let command = ScrollFullPageDownCommand;
        let state = AppState::new((80, 24), true);
        let event = KeyEvent::new(KeyCode::Char('f'), KeyModifiers::NONE);

        assert!(!Command::is_relevant(&command, &state, &event));
    }

    #[test]
    fn scroll_full_page_down_command_should_scroll_request_buffer() {
        let command = ScrollFullPageDownCommand;
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
        let command = ScrollFullPageDownCommand;
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
        let command = ScrollFullPageDownCommand;
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
        let command = ScrollFullPageUpCommand;
        let state = AppState::new((80, 24), true);
        let event = KeyEvent::new(KeyCode::Char('b'), KeyModifiers::CONTROL);

        assert!(Command::is_relevant(&command, &state, &event));
        assert_eq!(command.name(), "ScrollFullPageUp");
    }

    #[test]
    fn scroll_full_page_up_command_should_not_be_relevant_in_insert_mode() {
        let command = ScrollFullPageUpCommand;
        let mut state = AppState::new((80, 24), true);
        state.mode = EditorMode::Insert;
        let event = KeyEvent::new(KeyCode::Char('b'), KeyModifiers::CONTROL);

        assert!(!Command::is_relevant(&command, &state, &event));
    }

    #[test]
    fn scroll_full_page_up_command_should_not_be_relevant_without_ctrl() {
        let command = ScrollFullPageUpCommand;
        let state = AppState::new((80, 24), true);
        let event = KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE);

        assert!(!Command::is_relevant(&command, &state, &event));
    }

    #[test]
    fn scroll_full_page_up_command_should_scroll_request_buffer() {
        let command = ScrollFullPageUpCommand;
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
        let command = ScrollFullPageUpCommand;
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
        let command = ScrollFullPageUpCommand;
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
        let command = ScrollFullPageUpCommand;
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
        let command = ScrollFullPageDownCommand;
        let state = AppState::new((80, 24), true);
        let event = KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE);

        assert!(Command::is_relevant(&command, &state, &event));
    }

    #[test]
    fn scroll_full_page_down_command_should_be_relevant_for_page_down_in_insert_mode() {
        let command = ScrollFullPageDownCommand;
        let mut state = AppState::new((80, 24), true);
        state.mode = EditorMode::Insert;
        let event = KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE);

        assert!(Command::is_relevant(&command, &state, &event));
    }

    #[test]
    fn scroll_full_page_down_command_should_process_page_down_key() {
        let command = ScrollFullPageDownCommand;
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
        let command = ScrollFullPageUpCommand;
        let state = AppState::new((80, 24), true);
        let event = KeyEvent::new(KeyCode::PageUp, KeyModifiers::NONE);

        assert!(Command::is_relevant(&command, &state, &event));
    }

    #[test]
    fn scroll_full_page_up_command_should_be_relevant_for_page_up_in_insert_mode() {
        let command = ScrollFullPageUpCommand;
        let mut state = AppState::new((80, 24), true);
        state.mode = EditorMode::Insert;
        let event = KeyEvent::new(KeyCode::PageUp, KeyModifiers::NONE);

        assert!(Command::is_relevant(&command, &state, &event));
    }

    #[test]
    fn scroll_full_page_up_command_should_process_page_up_key() {
        let command = ScrollFullPageUpCommand;
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

    // Tests for SetPendingGCommand
    #[test]
    fn set_pending_g_command_should_be_relevant_for_first_g_in_normal_mode() {
        let command = SetPendingGCommand;
        let state = AppState::new((80, 24), true);
        let event = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE);

        assert!(Command::is_relevant(&command, &state, &event));
    }

    #[test]
    fn set_pending_g_command_should_not_be_relevant_when_pending_g_is_true() {
        let command = SetPendingGCommand;
        let mut state = AppState::new((80, 24), true);
        state.pending_g = true;
        let event = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE);

        assert!(!Command::is_relevant(&command, &state, &event));
    }

    #[test]
    fn set_pending_g_command_should_not_be_relevant_in_insert_mode() {
        let command = SetPendingGCommand;
        let mut state = AppState::new((80, 24), true);
        state.mode = EditorMode::Insert;
        let event = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE);

        assert!(!Command::is_relevant(&command, &state, &event));
    }

    #[test]
    fn set_pending_g_command_should_set_pending_state() {
        let command = SetPendingGCommand;
        let mut state = AppState::new((80, 24), true);
        let event = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE);

        assert!(!state.pending_g); // Initially false
        let result = command.process(event, &mut state).unwrap();
        assert!(result);
        assert!(state.pending_g); // Should be set to true
    }

    // Tests for GoToTopCommand
    #[test]
    fn go_to_top_command_should_be_relevant_for_second_g_when_pending() {
        let command = GoToTopCommand;
        let mut state = AppState::new((80, 24), true);
        state.pending_g = true;
        let event = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE);

        assert!(Command::is_relevant(&command, &state, &event));
    }

    #[test]
    fn go_to_top_command_should_not_be_relevant_when_not_pending() {
        let command = GoToTopCommand;
        let state = AppState::new((80, 24), true);
        let event = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE);

        assert!(!Command::is_relevant(&command, &state, &event));
    }

    #[test]
    fn go_to_top_command_should_not_be_relevant_in_insert_mode() {
        let command = GoToTopCommand;
        let mut state = AppState::new((80, 24), true);
        state.mode = EditorMode::Insert;
        state.pending_g = true;
        let event = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE);

        assert!(!Command::is_relevant(&command, &state, &event));
    }

    #[test]
    fn go_to_top_command_should_move_cursor_to_first_line_in_request_pane() {
        let command = GoToTopCommand;
        let mut state = AppState::new((80, 24), true);
        state.pending_g = true;
        state.current_pane = Pane::Request;

        // Set up request buffer with content and cursor not at top
        state.request_buffer.lines = vec![
            "GET /api/users HTTP/1.1".to_string(),
            "Host: example.com".to_string(),
            "Authorization: Bearer token".to_string(),
            "".to_string(),
            "".to_string(),
        ];
        state.request_buffer.cursor_line = 3;
        state.request_buffer.cursor_col = 5;
        state.request_buffer.scroll_offset = 1;

        let event = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE);
        let result = command.process(event, &mut state).unwrap();

        assert!(result);
        assert!(!state.pending_g); // Should clear pending state
        assert_eq!(state.request_buffer.cursor_line, 0); // Should go to first line
        assert_eq!(state.request_buffer.cursor_col, 0); // Should go to column 0
        assert_eq!(state.request_buffer.scroll_offset, 0); // Should scroll to top
    }

    #[test]
    fn go_to_top_command_should_move_cursor_to_first_line_in_response_pane() {
        use crate::repl::model::ResponseBuffer;

        let command = GoToTopCommand;
        let mut state = AppState::new((80, 24), true);
        state.pending_g = true;
        state.current_pane = Pane::Response;

        // Set up response buffer with content and cursor not at top
        let response_content =
            "HTTP/1.1 200 OK\nContent-Type: application/json\n\n{\"data\": \"test\"}".to_string();
        let mut response_buffer = ResponseBuffer::new(response_content);
        response_buffer.cursor_line = 2;
        response_buffer.cursor_col = 3;
        response_buffer.scroll_offset = 1;
        state.response_buffer = Some(response_buffer);

        let event = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE);
        let result = command.process(event, &mut state).unwrap();

        assert!(result);
        assert!(!state.pending_g); // Should clear pending state

        let response_buffer = state.response_buffer.as_ref().unwrap();
        assert_eq!(response_buffer.cursor_line, 0); // Should go to first line
        assert_eq!(response_buffer.cursor_col, 0); // Should go to column 0
        assert_eq!(response_buffer.scroll_offset, 0); // Should scroll to top
    }

    #[test]
    fn go_to_top_command_should_return_false_when_no_response_buffer() {
        let command = GoToTopCommand;
        let mut state = AppState::new((80, 24), true);
        state.pending_g = true;
        state.current_pane = Pane::Response;
        state.response_buffer = None;

        let event = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE);
        let result = command.process(event, &mut state).unwrap();

        assert!(!result);
        assert!(!state.pending_g); // Should still clear pending state
    }

    // Tests for GoToBottomCommand
    #[test]
    fn go_to_bottom_command_should_be_relevant_for_capital_g_in_normal_mode() {
        let command = GoToBottomCommand;
        let state = AppState::new((80, 24), true);
        let event = KeyEvent::new(KeyCode::Char('G'), KeyModifiers::NONE);

        assert!(Command::is_relevant(&command, &state, &event));
    }

    #[test]
    fn go_to_bottom_command_should_be_relevant_for_capital_g_with_shift_in_normal_mode() {
        let command = GoToBottomCommand;
        let state = AppState::new((80, 24), true);
        let event = KeyEvent::new(KeyCode::Char('G'), KeyModifiers::SHIFT);

        assert!(Command::is_relevant(&command, &state, &event));
    }

    #[test]
    fn go_to_bottom_command_should_not_be_relevant_in_insert_mode() {
        let command = GoToBottomCommand;
        let mut state = AppState::new((80, 24), true);
        state.mode = EditorMode::Insert;
        let event = KeyEvent::new(KeyCode::Char('G'), KeyModifiers::NONE);

        assert!(!Command::is_relevant(&command, &state, &event));
    }

    #[test]
    fn go_to_bottom_command_should_not_be_relevant_for_lowercase_g() {
        let command = GoToBottomCommand;
        let state = AppState::new((80, 24), true);
        let event = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE);

        assert!(!Command::is_relevant(&command, &state, &event));
    }

    #[test]
    fn go_to_bottom_command_should_move_cursor_to_last_line_in_request_pane() {
        let command = GoToBottomCommand;
        let mut state = AppState::new((80, 24), true);
        state.current_pane = Pane::Request;

        // Set up request buffer with content
        state.request_buffer.lines = vec![
            "GET /api/users HTTP/1.1".to_string(),
            "Host: example.com".to_string(),
            "Authorization: Bearer token".to_string(),
            "".to_string(),
            "Last line".to_string(),
        ];
        state.request_buffer.cursor_line = 0;
        state.request_buffer.cursor_col = 5;
        state.request_buffer.scroll_offset = 0;

        let event = KeyEvent::new(KeyCode::Char('G'), KeyModifiers::NONE);
        let result = command.process(event, &mut state).unwrap();

        assert!(result);
        assert_eq!(state.request_buffer.cursor_line, 4); // Should go to last line (0-indexed)
        assert_eq!(state.request_buffer.cursor_col, 0); // Should go to column 0
                                                        // Scroll offset should be adjusted if needed to show the last line
    }

    #[test]
    fn go_to_bottom_command_should_adjust_scroll_for_large_content() {
        let command = GoToBottomCommand;
        let mut state = AppState::new((80, 10), true); // Small terminal height
        state.current_pane = Pane::Request;

        // Create content larger than pane height
        state.request_buffer.lines = (0..50).map(|i| format!("line {}", i)).collect();
        state.request_buffer.cursor_line = 0;
        state.request_buffer.cursor_col = 0;
        state.request_buffer.scroll_offset = 0;

        let pane_height = state.get_request_pane_height();
        let line_count = state.request_buffer.lines.len();

        let event = KeyEvent::new(KeyCode::Char('G'), KeyModifiers::NONE);
        let result = command.process(event, &mut state).unwrap();

        assert!(result);
        assert_eq!(state.request_buffer.cursor_line, line_count - 1); // Should go to last line
        assert_eq!(state.request_buffer.cursor_col, 0); // Should go to column 0

        // Scroll offset should be adjusted to show the last line
        let expected_scroll = line_count - pane_height;
        assert_eq!(state.request_buffer.scroll_offset, expected_scroll);
    }

    #[test]
    fn go_to_bottom_command_should_work_with_response_pane() {
        use crate::repl::model::ResponseBuffer;

        let command = GoToBottomCommand;
        let mut state = AppState::new((80, 24), true);
        state.current_pane = Pane::Response;

        // Set up response buffer with multiple lines
        let response_content =
            "HTTP/1.1 200 OK\nContent-Type: application/json\n\n{\"data\": \"test\"}\nLast line"
                .to_string();
        let mut response_buffer = ResponseBuffer::new(response_content);
        response_buffer.cursor_line = 0;
        response_buffer.cursor_col = 3;
        response_buffer.scroll_offset = 0;
        state.response_buffer = Some(response_buffer);

        let event = KeyEvent::new(KeyCode::Char('G'), KeyModifiers::NONE);
        let result = command.process(event, &mut state).unwrap();

        assert!(result);

        let response_buffer = state.response_buffer.as_ref().unwrap();
        let expected_last_line = response_buffer.lines.len() - 1;
        assert_eq!(response_buffer.cursor_line, expected_last_line); // Should go to last line
        assert_eq!(response_buffer.cursor_col, 0); // Should go to column 0
    }

    #[test]
    fn go_to_bottom_command_should_return_false_when_no_response_buffer() {
        let command = GoToBottomCommand;
        let mut state = AppState::new((80, 24), true);
        state.current_pane = Pane::Response;
        state.response_buffer = None;

        let event = KeyEvent::new(KeyCode::Char('G'), KeyModifiers::NONE);
        let result = command.process(event, &mut state).unwrap();

        assert!(!result);
    }

    #[test]
    fn go_to_bottom_command_should_handle_empty_buffer() {
        let command = GoToBottomCommand;
        let mut state = AppState::new((80, 24), true);
        state.current_pane = Pane::Request;

        // Empty buffer (should still have one empty line from RequestBuffer::new())
        state.request_buffer.lines = vec!["".to_string()];

        let event = KeyEvent::new(KeyCode::Char('G'), KeyModifiers::NONE);
        let result = command.process(event, &mut state).unwrap();

        assert!(result);
        assert_eq!(state.request_buffer.cursor_line, 0); // Should stay at line 0
        assert_eq!(state.request_buffer.cursor_col, 0); // Should go to column 0
        assert_eq!(state.request_buffer.scroll_offset, 0); // Should not scroll
    }

    #[test]
    fn move_to_next_word_should_advance_from_start_of_word() {
        let mut buffer = create_test_request_buffer();
        buffer.lines = vec!["hello world test".to_string()];
        buffer.cursor_line = 0;
        buffer.cursor_col = 0; // At start of "hello"

        move_to_next_word(&mut buffer, 10).unwrap();

        assert_eq!(buffer.cursor_col, 6); // Should move to start of "world"
    }

    #[test]
    fn move_to_next_word_should_advance_from_middle_of_word() {
        let mut buffer = create_test_request_buffer();
        buffer.lines = vec!["hello world test".to_string()];
        buffer.cursor_line = 0;
        buffer.cursor_col = 2; // In middle of "hello"

        move_to_next_word(&mut buffer, 10).unwrap();

        assert_eq!(buffer.cursor_col, 6); // Should move to start of "world"
    }

    #[test]
    fn move_to_next_word_should_skip_multiple_spaces() {
        let mut buffer = create_test_request_buffer();
        buffer.lines = vec!["hello    world".to_string()];
        buffer.cursor_line = 0;
        buffer.cursor_col = 5; // At first space after "hello"

        move_to_next_word(&mut buffer, 10).unwrap();

        assert_eq!(buffer.cursor_col, 9); // Should move to start of "world"
    }

    #[test]
    fn move_to_next_word_should_move_to_next_line_when_at_end() {
        let mut buffer = create_test_request_buffer();
        buffer.lines = vec!["hello".to_string(), "world".to_string()];
        buffer.cursor_line = 0;
        buffer.cursor_col = 5; // At end of first line

        move_to_next_word(&mut buffer, 10).unwrap();

        assert_eq!(buffer.cursor_line, 1); // Should move to next line
        assert_eq!(buffer.cursor_col, 0); // Should be at start of "world"
    }

    #[test]
    fn move_to_next_word_should_handle_punctuation() {
        let mut buffer = create_test_request_buffer();
        buffer.lines = vec!["hello, world!".to_string()];
        buffer.cursor_line = 0;
        buffer.cursor_col = 0; // At start of "hello"

        move_to_next_word(&mut buffer, 10).unwrap();

        assert_eq!(buffer.cursor_col, 7); // Should move to start of "world"
    }

    #[test]
    fn move_to_next_word_should_stop_at_last_position_when_no_more_words() {
        let mut buffer = create_test_request_buffer();
        buffer.lines = vec!["hello".to_string()];
        buffer.cursor_line = 0;
        buffer.cursor_col = 5; // At end of line

        move_to_next_word(&mut buffer, 10).unwrap();

        assert_eq!(buffer.cursor_line, 0);
        assert_eq!(buffer.cursor_col, 5); // Should stay at end
    }

    #[test]
    fn move_to_next_word_should_auto_scroll_when_moving_beyond_visible_area() {
        let mut buffer = create_test_request_buffer();
        // Create enough content to require scrolling (more than visible height of 3)
        buffer.lines = vec![
            "line0 word".to_string(),
            "line1 word".to_string(),
            "line2 word".to_string(),
            "line3 word".to_string(),
            "line4 word".to_string(),
        ];
        buffer.cursor_line = 2;
        buffer.cursor_col = 0; // At start of "line2"
        buffer.scroll_offset = 0;

        // With visible height of 3, lines 0-2 are visible, line 3+ are not
        let result = move_to_next_word(&mut buffer, 3).unwrap();

        assert_eq!(buffer.cursor_line, 2); // Should move to "word" on same line
        assert_eq!(buffer.cursor_col, 6); // Should be at start of "word"
        assert_eq!(buffer.scroll_offset, 0); // No scroll needed for same line
        assert!(result.cursor_moved);
        assert!(!result.scroll_occurred);

        // Now move to next word which should be on line 3 (beyond visible area)
        let result = move_to_next_word(&mut buffer, 3).unwrap();

        assert_eq!(buffer.cursor_line, 3); // Should move to line 3
        assert_eq!(buffer.cursor_col, 0); // Should be at start of "line3"
        assert_eq!(buffer.scroll_offset, 1); // Should scroll down to show line 3
        assert!(result.cursor_moved);
        assert!(result.scroll_occurred);
    }
}

// DISPLAY LINE MOVEMENT COMMANDS USING CACHE
// These new commands provide visual line movement (vim-style)

/// Command for moving cursor up by display line (cache-based, handles word wrapping)
pub struct MoveCursorUpDisplayCommand;

impl Command for MoveCursorUpDisplayCommand {
    fn is_relevant(&self, state: &AppState, event: &KeyEvent) -> bool {
        // Allow gk in Normal mode for display line movement
        match event.code {
            KeyCode::Char('k') => {
                matches!(state.mode, EditorMode::Normal)
                    && event.modifiers == KeyModifiers::NONE
                    && state.pending_g // Only when 'g' was pressed first
            }
            _ => false,
        }
    }

    fn process(&self, _event: KeyEvent, state: &mut AppState) -> Result<bool> {
        // Clear pending g flag
        state.pending_g = false;

        // Get current pane width for cache update
        let content_width = match state.current_pane {
            Pane::Request => {
                let total_width = state.terminal_size.0 as usize;
                total_width.saturating_sub(4) // Account for borders and padding
            }
            Pane::Response => {
                let total_width = state.terminal_size.0 as usize;
                total_width.saturating_sub(4) // Account for borders and padding
            }
        };

        // Update cache for current pane
        if let Err(e) = state.update_display_cache(content_width) {
            eprintln!("Warning: Failed to update display cache: {}", e);
            return Ok(false);
        }

        // Get current cursor position and desired column
        let current_buffer = state.current_buffer();
        let desired_col = current_buffer.cursor_col();

        // Try to move up one display line
        if let Some((new_logical_line, new_logical_col)) =
            state.move_cursor_up_display_line(desired_col)
        {
            // Update cursor position in the buffer
            match state.current_pane {
                Pane::Request => {
                    state.request_buffer.cursor_line = new_logical_line;
                    state.request_buffer.cursor_col = new_logical_col;
                    state.request_buffer.clamp_cursor();
                }
                Pane::Response => {
                    if let Some(ref mut buffer) = state.response_buffer {
                        buffer.cursor_line = new_logical_line;
                        buffer.cursor_col = new_logical_col;
                        buffer.clamp_cursor();
                    }
                }
            }
            Ok(true)
        } else {
            Ok(false) // Movement not possible
        }
    }

    fn name(&self) -> &'static str {
        "MoveCursorUpDisplay"
    }
}

/// Command for moving cursor down by display line (cache-based, handles word wrapping)
pub struct MoveCursorDownDisplayCommand;

impl Command for MoveCursorDownDisplayCommand {
    fn is_relevant(&self, state: &AppState, event: &KeyEvent) -> bool {
        // Allow gj in Normal mode for display line movement
        match event.code {
            KeyCode::Char('j') => {
                matches!(state.mode, EditorMode::Normal)
                    && event.modifiers == KeyModifiers::NONE
                    && state.pending_g // Only when 'g' was pressed first
            }
            _ => false,
        }
    }

    fn process(&self, _event: KeyEvent, state: &mut AppState) -> Result<bool> {
        // Clear pending g flag
        state.pending_g = false;

        // Get current pane width for cache update
        let content_width = match state.current_pane {
            Pane::Request => {
                let total_width = state.terminal_size.0 as usize;
                total_width.saturating_sub(4) // Account for borders and padding
            }
            Pane::Response => {
                let total_width = state.terminal_size.0 as usize;
                total_width.saturating_sub(4) // Account for borders and padding
            }
        };

        // Update cache for current pane
        if let Err(e) = state.update_display_cache(content_width) {
            eprintln!("Warning: Failed to update display cache: {}", e);
            return Ok(false);
        }

        // Get current cursor position and desired column
        let current_buffer = state.current_buffer();
        let desired_col = current_buffer.cursor_col();

        // Try to move down one display line
        if let Some((new_logical_line, new_logical_col)) =
            state.move_cursor_down_display_line(desired_col)
        {
            // Update cursor position in the buffer
            match state.current_pane {
                Pane::Request => {
                    state.request_buffer.cursor_line = new_logical_line;
                    state.request_buffer.cursor_col = new_logical_col;
                    state.request_buffer.clamp_cursor();
                }
                Pane::Response => {
                    if let Some(ref mut buffer) = state.response_buffer {
                        buffer.cursor_line = new_logical_line;
                        buffer.cursor_col = new_logical_col;
                        buffer.clamp_cursor();
                    }
                }
            }
            Ok(true)
        } else {
            Ok(false) // Movement not possible
        }
    }

    fn name(&self) -> &'static str {
        "MoveCursorDownDisplay"
    }
}

#[cfg(test)]
mod display_line_movement_tests {
    use super::*;

    #[test]
    fn move_cursor_up_display_command_should_work_with_wrapped_lines() {
        let mut state = AppState::new((80, 24), false);

        // Create wrapped content in response buffer (synchronous cache updates)
        let wrapped_content = "This is a very long line that should wrap across multiple display lines when displayed\nShort line".to_string();
        state.set_response(wrapped_content);
        state.current_pane = crate::repl::model::Pane::Response;

        // Position cursor at second line in response buffer
        if let Some(ref mut response) = state.response_buffer {
            response.cursor_line = 1;
            response.cursor_col = 0;
        }

        // Set up pending g flag
        state.pending_g = true;

        // Pre-populate the cache by calling update_display_cache
        let content_width = 76; // 80 - 4 for borders
        state.update_display_cache(content_width).unwrap();

        let command = MoveCursorUpDisplayCommand;
        let event = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);

        // Should move to the last display line of the wrapped first line
        let result = command.process(event, &mut state).unwrap();
        assert!(result);
        assert!(!state.pending_g); // Flag should be cleared

        // Check cursor moved to wrapped first line
        if let Some(ref response) = state.response_buffer {
            assert_eq!(response.cursor_line, 0);
            // Column should be positioned in the wrapped segment
            assert!(response.cursor_col > 0);
        }
    }

    #[test]
    fn move_cursor_down_display_command_should_work_with_wrapped_lines() {
        let mut state = AppState::new((80, 24), false);

        // Create wrapped content in response buffer (synchronous cache updates)
        let wrapped_content = "This is a very long line that should wrap across multiple display lines when displayed\nSecond line".to_string();
        state.set_response(wrapped_content);
        state.current_pane = crate::repl::model::Pane::Response;

        // Position cursor at beginning of first line in response buffer
        if let Some(ref mut response) = state.response_buffer {
            response.cursor_line = 0;
            response.cursor_col = 0;
        }

        // Set up pending g flag
        state.pending_g = true;

        // Pre-populate the cache by calling update_display_cache
        let content_width = 76; // 80 - 4 for borders
        state.update_display_cache(content_width).unwrap();

        let command = MoveCursorDownDisplayCommand;
        let event = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);

        // Should move to the next display line within the wrapped first line
        let result = command.process(event, &mut state).unwrap();
        assert!(result);
        assert!(!state.pending_g); // Flag should be cleared

        // Check cursor is still on first logical line but in wrapped segment
        if let Some(ref response) = state.response_buffer {
            assert_eq!(response.cursor_line, 0);
            // Column should be positioned in the wrapped segment
            assert!(response.cursor_col > 0);
        }
    }

    #[test]
    fn display_commands_should_not_be_relevant_without_pending_g() {
        let state = AppState::new((80, 24), false);

        let up_command = MoveCursorUpDisplayCommand;
        let down_command = MoveCursorDownDisplayCommand;

        let k_event = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
        let j_event = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);

        // Without pending_g, these commands should not be relevant
        assert!(!up_command.is_relevant(&state, &k_event));
        assert!(!down_command.is_relevant(&state, &j_event));
    }
}
