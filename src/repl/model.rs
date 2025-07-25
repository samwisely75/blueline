//! # Model - Application State and Data Structures
//!
//! This module contains all data structures for the REPL including buffers,
//! application state, and enums for modes and configuration.
//!
//! ## Design Principles
//!
//! - **Data Only**: Models store state but don't process commands
//! - **Observable**: Models can notify observers when they change  
//! - **Immutable Methods**: Prefer methods that return new states over mutation
//! - **Clear Ownership**: Each piece of state has a clear owner

use std::collections::HashMap;
use std::time::Instant;

/// Editor modes matching vim behavior
#[derive(Debug, Clone, PartialEq)]
pub enum EditorMode {
    Normal,
    Insert,
    Command,
    Visual,
    VisualLine,
}

/// Active pane in the dual-pane interface
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Pane {
    Request,
    Response,
}

/// Visual selection state for vim-style text selection
#[derive(Debug, Clone, PartialEq)]
pub struct VisualSelection {
    pub start_line: usize,
    pub start_col: usize,
    pub end_line: usize,
    pub end_col: usize,
}

/// Request buffer containing HTTP request content.
///
/// This is the editable buffer where users compose HTTP requests.
/// It only manages content and cursor position - no key processing logic.
#[derive(Debug, Clone)]
pub struct RequestBuffer {
    pub lines: Vec<String>,
    pub cursor_line: usize,
    pub cursor_col: usize,
    pub scroll_offset: usize,
}

impl RequestBuffer {
    /// Create a new empty request buffer
    pub fn new() -> Self {
        Self {
            lines: vec![String::new()],
            cursor_line: 0,
            cursor_col: 0,
            scroll_offset: 0,
        }
    }

    /// Get the complete text content as a single string
    pub fn get_text(&self) -> String {
        self.lines.join("\n")
    }

    /// Get the current line content
    pub fn current_line(&self) -> &str {
        self.lines.get(self.cursor_line).map_or("", |s| s.as_str())
    }

    /// Get the number of lines
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    /// Check if cursor is at valid position
    pub fn is_cursor_valid(&self) -> bool {
        self.cursor_line < self.lines.len()
            && self.cursor_col <= self.lines.get(self.cursor_line).map_or(0, |l| l.len())
    }

    /// Ensure cursor is within valid bounds
    pub fn clamp_cursor(&mut self) {
        if self.cursor_line >= self.lines.len() {
            self.cursor_line = self.lines.len().saturating_sub(1);
        }

        if let Some(line) = self.lines.get(self.cursor_line) {
            if self.cursor_col > line.len() {
                self.cursor_col = line.len();
            }
        }
    }

    /// Generic scroll function that handles both directions
    /// 
    /// Scrolls the buffer by the specified number of lines. Positive values scroll down,
    /// negative values scroll up. The cursor is positioned at the top of the newly visible
    /// area following vim behavior.
    pub fn scroll(&mut self, lines: i32, page_height: usize) {
        if lines == 0 {
            return;
        }

        if lines > 0 {
            // Scroll down - increase scroll_offset
            let max_scroll = self.lines.len().saturating_sub(page_height);
            let scroll_amount = (lines as usize).min(max_scroll.saturating_sub(self.scroll_offset));
            
            if scroll_amount == 0 {
                return;
            }
            
            self.scroll_offset += scroll_amount;
        } else {
            // Scroll up - decrease scroll_offset
            let scroll_amount = ((-lines) as usize).min(self.scroll_offset);
            
            if scroll_amount == 0 {
                return;
            }
            
            self.scroll_offset -= scroll_amount;
        }

        // Move cursor to top of newly visible area (vim behavior)
        self.cursor_line = self.scroll_offset;

        // Ensure cursor column is within line bounds
        self.clamp_cursor();
    }

    /// Scroll up by half a page, moving cursor to top of newly visible area (vim behavior)
    pub fn scroll_half_page_up(&mut self, half_page_size: usize) {
        self.scroll(-(half_page_size as i32), half_page_size * 2);
    }

    /// Scroll down by half a page, moving cursor to top of newly visible area (vim behavior)
    pub fn scroll_half_page_down(&mut self, half_page_size: usize) {
        self.scroll(half_page_size as i32, half_page_size * 2);
    }

    /// Get visible line range for given viewport height
    pub fn visible_range(&self, viewport_height: usize) -> (usize, usize) {
        let start = self.scroll_offset;
        let end = (start + viewport_height).min(self.lines.len());
        (start, end)
    }
}

impl Default for RequestBuffer {
    fn default() -> Self {
        Self::new()
    }
}

/// Response buffer containing HTTP response content (read-only).
///
/// This buffer displays HTTP response data and allows navigation
/// but no editing. It's essentially a read-only view of response content.
#[derive(Debug, Clone)]
pub struct ResponseBuffer {
    #[allow(dead_code)]
    content: String,
    pub lines: Vec<String>,
    pub scroll_offset: usize,
    pub cursor_line: usize,
    pub cursor_col: usize,
}

impl ResponseBuffer {
    /// Create a new response buffer from content string
    pub fn new(content: String) -> Self {
        let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        Self {
            content,
            lines,
            scroll_offset: 0,
            cursor_line: 0,
            cursor_col: 0,
        }
    }

    /// Get the current line content
    pub fn current_line(&self) -> &str {
        self.lines.get(self.cursor_line).map_or("", |s| s.as_str())
    }

    /// Get the number of lines
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    /// Get visible line range for given viewport height
    pub fn visible_range(&self, viewport_height: usize) -> (usize, usize) {
        let start = self.scroll_offset;
        let end = (start + viewport_height).min(self.lines.len());
        (start, end)
    }

    /// Check if cursor is at valid position
    pub fn is_cursor_valid(&self) -> bool {
        self.cursor_line < self.lines.len()
            && self.cursor_col <= self.lines.get(self.cursor_line).map_or(0, |l| l.len())
    }

    /// Ensure cursor is within valid bounds
    pub fn clamp_cursor(&mut self) {
        if self.cursor_line >= self.lines.len() && !self.lines.is_empty() {
            self.cursor_line = self.lines.len() - 1;
        }

        if let Some(line) = self.lines.get(self.cursor_line) {
            if self.cursor_col > line.len() {
                self.cursor_col = line.len();
            }
        }
    }

    /// Generic scroll function that handles both directions
    /// 
    /// Scrolls the buffer by the specified number of lines. Positive values scroll down,
    /// negative values scroll up. The cursor is positioned at the top of the newly visible
    /// area following vim behavior.
    pub fn scroll(&mut self, lines: i32, page_height: usize) {
        if lines == 0 {
            return;
        }

        if lines > 0 {
            // Scroll down - increase scroll_offset
            let max_scroll = self.lines.len().saturating_sub(page_height);
            let scroll_amount = (lines as usize).min(max_scroll.saturating_sub(self.scroll_offset));
            
            if scroll_amount == 0 {
                return;
            }
            
            self.scroll_offset += scroll_amount;
        } else {
            // Scroll up - decrease scroll_offset
            let scroll_amount = ((-lines) as usize).min(self.scroll_offset);
            
            if scroll_amount == 0 {
                return;
            }
            
            self.scroll_offset -= scroll_amount;
        }

        // Move cursor to top of newly visible area (vim behavior)
        self.cursor_line = self.scroll_offset;

        // Ensure cursor column is within line bounds
        self.clamp_cursor();
    }

    /// Scroll up by half a page, moving cursor to top of newly visible area (vim behavior)
    pub fn scroll_half_page_up(&mut self, half_page_size: usize) {
        self.scroll(-(half_page_size as i32), half_page_size * 2);
    }

    /// Scroll down by half a page, moving cursor to top of newly visible area (vim behavior)
    pub fn scroll_half_page_down(&mut self, half_page_size: usize) {
        self.scroll(half_page_size as i32, half_page_size * 2);
    }
}

/// Complete application state.
///
/// This is the central state container that holds all mutable state
/// for the REPL application. Commands operate on this state, and
/// observers watch it for changes.
#[derive(Debug)]
pub struct AppState {
    // Editor state
    pub mode: EditorMode,
    pub current_pane: Pane,
    pub visual_selection: Option<VisualSelection>,

    // Buffers
    pub request_buffer: RequestBuffer,
    pub response_buffer: Option<ResponseBuffer>,

    // UI state
    pub terminal_size: (u16, u16),
    pub request_pane_height: usize,
    pub status_message: String,
    pub command_buffer: String,

    // Session state
    pub session_headers: HashMap<String, String>,
    pub clipboard: String,

    // Vim state tracking
    pub pending_g: bool,
    pub pending_ctrl_w: bool,

    // Request timing
    pub last_response_status: Option<String>,
    pub request_start_time: Option<Instant>,
    pub last_request_duration: Option<u64>, // in milliseconds

    // Request execution flag for async operations
    pub execute_request_flag: bool,

    // Application lifecycle
    pub should_quit: bool,

    // Configuration
    pub verbose: bool,
}

impl AppState {
    /// Create new application state with default values
    pub fn new(terminal_size: (u16, u16), verbose: bool) -> Self {
        let height = terminal_size.1 as usize;
        let total_content_height = height.saturating_sub(2); // Minus separator and status line
        let initial_request_pane_height = total_content_height / 2;

        Self {
            mode: EditorMode::Normal, // Start in normal mode for vim-like behavior
            current_pane: Pane::Request,
            visual_selection: None,
            request_buffer: RequestBuffer::new(),
            response_buffer: None,
            terminal_size,
            request_pane_height: initial_request_pane_height,
            status_message: "-- INSERT --".to_string(),
            command_buffer: String::new(),
            session_headers: HashMap::new(),
            clipboard: String::new(),
            pending_g: false,
            pending_ctrl_w: false,
            last_response_status: None,
            request_start_time: None,
            last_request_duration: None,
            execute_request_flag: false,
            should_quit: false,
            verbose,
        }
    }

    /// Get the height of the request pane in lines
    /// Uses full available space when there's no response buffer
    pub fn get_request_pane_height(&self) -> usize {
        // Use full available space when there's no response buffer
        if self.response_buffer.is_none() {
            let total_height = self.terminal_size.1 as usize;
            return total_height.saturating_sub(2); // Minus separator and status
        }

        self.request_pane_height
    }

    /// Get the height of the response pane in lines
    /// Returns 0 when there's no response to hide the pane initially
    pub fn get_response_pane_height(&self) -> usize {
        // Hide response pane when there's no response buffer
        if self.response_buffer.is_none() {
            return 0;
        }

        let total_height = self.terminal_size.1 as usize;
        let total_content_height = total_height.saturating_sub(2); // Minus separator and status
        total_content_height.saturating_sub(self.request_pane_height)
    }

    /// Update terminal size and adjust pane heights proportionally
    pub fn update_terminal_size(&mut self, new_size: (u16, u16)) {
        let old_height = self.terminal_size.1 as usize;
        let old_total_content_height = old_height.saturating_sub(2);
        let new_height = new_size.1 as usize;
        let new_total_content_height = new_height.saturating_sub(2);

        // Maintain proportional split
        if old_total_content_height > 0 {
            let proportion = self.request_pane_height as f64 / old_total_content_height as f64;
            self.request_pane_height = ((new_total_content_height as f64 * proportion) as usize)
                .max(3) // Minimum input height
                .min(new_total_content_height.saturating_sub(3)); // Leave 3 for output
        }

        self.terminal_size = new_size;
    }

    /// Set response content and create response buffer
    pub fn set_response(&mut self, content: String) {
        self.response_buffer = Some(ResponseBuffer::new(content));
    }

    /// Clear response buffer
    pub fn clear_response(&mut self) {
        self.response_buffer = None;
    }

    /// Check if there's an active response
    pub fn has_response(&self) -> bool {
        self.response_buffer.is_some()
    }

    /// Get mutable reference to current buffer based on active pane
    pub fn current_buffer_mut(&mut self) -> CurrentBufferMut {
        match self.current_pane {
            Pane::Request => CurrentBufferMut::Request(&mut self.request_buffer),
            Pane::Response => {
                if let Some(ref mut response) = self.response_buffer {
                    CurrentBufferMut::Response(response)
                } else {
                    // If no response buffer, fall back to request buffer
                    CurrentBufferMut::Request(&mut self.request_buffer)
                }
            }
        }
    }

    /// Get immutable reference to current buffer based on active pane
    pub fn current_buffer(&self) -> CurrentBuffer {
        match self.current_pane {
            Pane::Request => CurrentBuffer::Request(&self.request_buffer),
            Pane::Response => {
                if let Some(ref response) = self.response_buffer {
                    CurrentBuffer::Response(response)
                } else {
                    // If no response buffer, fall back to request buffer
                    CurrentBuffer::Request(&self.request_buffer)
                }
            }
        }
    }
}

/// Enum for mutable access to current buffer
pub enum CurrentBufferMut<'a> {
    Request(&'a mut RequestBuffer),
    Response(&'a mut ResponseBuffer),
}

/// Enum for immutable access to current buffer  
pub enum CurrentBuffer<'a> {
    Request(&'a RequestBuffer),
    Response(&'a ResponseBuffer),
}

impl CurrentBuffer<'_> {
    pub fn cursor_line(&self) -> usize {
        match self {
            CurrentBuffer::Request(buf) => buf.cursor_line,
            CurrentBuffer::Response(buf) => buf.cursor_line,
        }
    }

    pub fn cursor_col(&self) -> usize {
        match self {
            CurrentBuffer::Request(buf) => buf.cursor_col,
            CurrentBuffer::Response(buf) => buf.cursor_col,
        }
    }

    pub fn line_count(&self) -> usize {
        match self {
            CurrentBuffer::Request(buf) => buf.line_count(),
            CurrentBuffer::Response(buf) => buf.line_count(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_buffer_new_should_create_empty_buffer_with_single_empty_line() {
        let buffer = RequestBuffer::new();

        assert_eq!(buffer.lines.len(), 1);
        assert_eq!(buffer.lines[0], "");
        assert_eq!(buffer.cursor_line, 0);
        assert_eq!(buffer.cursor_col, 0);
        assert_eq!(buffer.scroll_offset, 0);
    }

    #[test]
    fn request_buffer_default_should_work_like_new() {
        let buffer = RequestBuffer::default();
        let new_buffer = RequestBuffer::new();

        assert_eq!(buffer.lines, new_buffer.lines);
        assert_eq!(buffer.cursor_line, new_buffer.cursor_line);
        assert_eq!(buffer.cursor_col, new_buffer.cursor_col);
        assert_eq!(buffer.scroll_offset, new_buffer.scroll_offset);
    }

    #[test]
    fn request_buffer_get_text_should_join_lines_with_newlines() {
        let mut buffer = RequestBuffer::new();
        buffer.lines = vec![
            "GET /api/users".to_string(),
            "Host: example.com".to_string(),
            "".to_string(),
            "{\"test\": true}".to_string(),
        ];

        let text = buffer.get_text();
        assert_eq!(
            text,
            "GET /api/users\nHost: example.com\n\n{\"test\": true}"
        );
    }

    #[test]
    fn request_buffer_get_text_should_handle_single_line() {
        let buffer = RequestBuffer::new();
        assert_eq!(buffer.get_text(), "");
    }

    #[test]
    fn request_buffer_current_line_should_return_line_at_cursor_position() {
        let mut buffer = RequestBuffer::new();
        buffer.lines = vec!["first line".to_string(), "second line".to_string()];
        buffer.cursor_line = 1;

        assert_eq!(buffer.current_line(), "second line");
    }

    #[test]
    fn request_buffer_current_line_should_return_empty_for_invalid_position() {
        let mut buffer = RequestBuffer::new();
        buffer.cursor_line = 10; // Out of bounds

        assert_eq!(buffer.current_line(), "");
    }

    #[test]
    fn request_buffer_line_count_should_return_number_of_lines() {
        let mut buffer = RequestBuffer::new();
        buffer.lines = vec![
            "line1".to_string(),
            "line2".to_string(),
            "line3".to_string(),
        ];

        assert_eq!(buffer.line_count(), 3);
    }

    #[test]
    fn request_buffer_is_cursor_valid_should_return_true_for_valid_position() {
        let mut buffer = RequestBuffer::new();
        buffer.lines = vec!["hello world".to_string()];
        buffer.cursor_line = 0;
        buffer.cursor_col = 5;

        assert!(buffer.is_cursor_valid());
    }

    #[test]
    fn request_buffer_is_cursor_valid_should_return_false_for_invalid_line() {
        let mut buffer = RequestBuffer::new();
        buffer.cursor_line = 10;

        assert!(!buffer.is_cursor_valid());
    }

    #[test]
    fn request_buffer_is_cursor_valid_should_return_false_for_invalid_column() {
        let mut buffer = RequestBuffer::new();
        buffer.lines = vec!["hello".to_string()];
        buffer.cursor_line = 0;
        buffer.cursor_col = 10; // Beyond line length

        assert!(!buffer.is_cursor_valid());
    }

    #[test]
    fn request_buffer_clamp_cursor_should_fix_invalid_line_position() {
        let mut buffer = RequestBuffer::new();
        buffer.lines = vec!["line1".to_string(), "line2".to_string()];
        buffer.cursor_line = 10; // Out of bounds

        buffer.clamp_cursor();
        assert_eq!(buffer.cursor_line, 1); // Should be clamped to last line
    }

    #[test]
    fn request_buffer_clamp_cursor_should_fix_invalid_column_position() {
        let mut buffer = RequestBuffer::new();
        buffer.lines = vec!["hello".to_string()];
        buffer.cursor_line = 0;
        buffer.cursor_col = 10; // Beyond line length

        buffer.clamp_cursor();
        assert_eq!(buffer.cursor_col, 5); // Should be clamped to line end
    }

    #[test]
    fn request_buffer_visible_range_should_return_correct_range() {
        let mut buffer = RequestBuffer::new();
        buffer.lines = vec![
            "1".to_string(),
            "2".to_string(),
            "3".to_string(),
            "4".to_string(),
            "5".to_string(),
        ];
        buffer.scroll_offset = 1;

        let (start, end) = buffer.visible_range(3);
        assert_eq!(start, 1);
        assert_eq!(end, 4);
    }

    #[test]
    fn request_buffer_visible_range_should_not_exceed_line_count() {
        let mut buffer = RequestBuffer::new();
        buffer.lines = vec!["1".to_string(), "2".to_string()];
        buffer.scroll_offset = 0;

        let (start, end) = buffer.visible_range(10);
        assert_eq!(start, 0);
        assert_eq!(end, 2); // Clamped to actual line count
    }

    #[test]
    fn request_buffer_scroll_should_handle_zero_lines() {
        let mut buffer = RequestBuffer::new();
        buffer.lines = vec!["line 0".to_string(), "line 1".to_string()];
        buffer.cursor_line = 1;
        buffer.scroll_offset = 0;

        buffer.scroll(0, 10); // No scroll

        assert_eq!(buffer.scroll_offset, 0);
        assert_eq!(buffer.cursor_line, 1);
    }

    #[test]
    fn request_buffer_scroll_should_scroll_down_and_move_cursor_to_top() {
        let mut buffer = RequestBuffer::new();
        buffer.lines = vec![
            "line 0".to_string(),
            "line 1".to_string(),
            "line 2".to_string(),
            "line 3".to_string(),
            "line 4".to_string(),
            "line 5".to_string(),
        ];
        buffer.cursor_line = 1;
        buffer.scroll_offset = 0;

        buffer.scroll(2, 4); // Scroll down 2 lines with page height 4

        assert_eq!(buffer.scroll_offset, 2);
        assert_eq!(buffer.cursor_line, 2); // Cursor moves to top of visible area
    }

    #[test]
    fn request_buffer_scroll_should_scroll_up_and_move_cursor_to_top() {
        let mut buffer = RequestBuffer::new();
        buffer.lines = vec![
            "line 0".to_string(),
            "line 1".to_string(),
            "line 2".to_string(),
            "line 3".to_string(),
            "line 4".to_string(),
            "line 5".to_string(),
        ];
        buffer.cursor_line = 4;
        buffer.scroll_offset = 3;

        buffer.scroll(-2, 4); // Scroll up 2 lines with page height 4

        assert_eq!(buffer.scroll_offset, 1);
        assert_eq!(buffer.cursor_line, 1); // Cursor moves to top of visible area
    }

    #[test]
    fn request_buffer_scroll_should_limit_scroll_down_to_available_space() {
        let mut buffer = RequestBuffer::new();
        buffer.lines = vec![
            "line 0".to_string(),
            "line 1".to_string(),
            "line 2".to_string(),
        ];
        buffer.cursor_line = 0;
        buffer.scroll_offset = 0;

        buffer.scroll(10, 2); // Request more than available (max scroll would be 1)

        assert_eq!(buffer.scroll_offset, 1); // Limited to max available
        assert_eq!(buffer.cursor_line, 1);
    }

    #[test]
    fn request_buffer_scroll_should_limit_scroll_up_to_available_offset() {
        let mut buffer = RequestBuffer::new();
        buffer.lines = vec![
            "line 0".to_string(),
            "line 1".to_string(),
            "line 2".to_string(),
        ];
        buffer.cursor_line = 2;
        buffer.scroll_offset = 1;

        buffer.scroll(-10, 2); // Request more than available

        assert_eq!(buffer.scroll_offset, 0); // Limited to available offset
        assert_eq!(buffer.cursor_line, 0);
    }

    #[test]
    fn request_buffer_scroll_should_handle_no_scroll_down_when_at_bottom() {
        let mut buffer = RequestBuffer::new();
        buffer.lines = vec!["line 0".to_string(), "line 1".to_string()];
        buffer.cursor_line = 1;
        buffer.scroll_offset = 0; // Already showing all content

        buffer.scroll(5, 2); // Try to scroll down

        assert_eq!(buffer.scroll_offset, 0); // No change
        assert_eq!(buffer.cursor_line, 1); // No change
    }

    #[test]
    fn request_buffer_scroll_should_handle_no_scroll_up_when_at_top() {
        let mut buffer = RequestBuffer::new();
        buffer.lines = vec!["line 0".to_string(), "line 1".to_string()];
        buffer.cursor_line = 0;
        buffer.scroll_offset = 0;

        buffer.scroll(-5, 2); // Try to scroll up

        assert_eq!(buffer.scroll_offset, 0); // No change
        assert_eq!(buffer.cursor_line, 0); // No change
    }

    #[test]
    fn request_buffer_scroll_should_clamp_cursor_column_after_scroll() {
        let mut buffer = RequestBuffer::new();
        buffer.lines = vec![
            "long line here".to_string(),
            "short".to_string(),
            "another long line here".to_string(),
        ];
        buffer.cursor_line = 0;
        buffer.cursor_col = 10; // Valid for first line
        buffer.scroll_offset = 0;

        buffer.scroll(1, 2); // Scroll down 1 line

        assert_eq!(buffer.scroll_offset, 1);
        assert_eq!(buffer.cursor_line, 1); // Move to top of visible area
        assert_eq!(buffer.cursor_col, 5); // Clamped to "short".len()
    }

    #[test]
    fn request_buffer_scroll_half_page_up_should_scroll_and_move_cursor() {
        let mut buffer = RequestBuffer::new();
        buffer.lines = vec![
            "line 0".to_string(),
            "line 1".to_string(),
            "line 2".to_string(),
            "line 3".to_string(),
            "line 4".to_string(),
            "line 5".to_string(),
        ];
        buffer.cursor_line = 4;
        buffer.cursor_col = 2;
        buffer.scroll_offset = 2;

        buffer.scroll_half_page_up(2);

        assert_eq!(buffer.scroll_offset, 0);
        assert_eq!(buffer.cursor_line, 0); // Cursor moves to top of visible area (vim behavior)
        assert_eq!(buffer.cursor_col, 2);
    }

    #[test]
    fn request_buffer_scroll_half_page_up_should_handle_zero_scroll_offset() {
        let mut buffer = RequestBuffer::new();
        buffer.lines = vec!["line 0".to_string(), "line 1".to_string()];
        buffer.cursor_line = 1;
        buffer.scroll_offset = 0;

        buffer.scroll_half_page_up(5);

        // Should not change anything when already at top
        assert_eq!(buffer.scroll_offset, 0);
        assert_eq!(buffer.cursor_line, 1); // Cursor should remain unchanged
    }

    #[test]
    fn request_buffer_scroll_half_page_up_should_clamp_cursor_to_scroll_offset_when_needed() {
        let mut buffer = RequestBuffer::new();
        buffer.lines = vec![
            "line 0".to_string(),
            "line 1".to_string(),
            "line 2".to_string(),
            "line 3".to_string(),
        ];
        buffer.cursor_line = 1; // Close to top
        buffer.cursor_col = 3;
        buffer.scroll_offset = 3;

        buffer.scroll_half_page_up(2);

        assert_eq!(buffer.scroll_offset, 1);
        assert_eq!(buffer.cursor_line, 1); // Should move to top of visible area (scroll_offset)
        assert_eq!(buffer.cursor_col, 3);
    }

    #[test]
    fn request_buffer_scroll_half_page_up_should_limit_scroll_to_available_offset() {
        let mut buffer = RequestBuffer::new();
        buffer.lines = vec!["line 0".to_string(), "line 1".to_string()];
        buffer.cursor_line = 1;
        buffer.scroll_offset = 1;

        buffer.scroll_half_page_up(10); // Request more than available

        assert_eq!(buffer.scroll_offset, 0); // Should only scroll available amount
        assert_eq!(buffer.cursor_line, 0);
    }

    #[test]
    fn request_buffer_scroll_half_page_up_should_clamp_cursor_column_to_line_bounds() {
        let mut buffer = RequestBuffer::new();
        buffer.lines = vec!["short".to_string(), "longer line".to_string()];
        buffer.cursor_line = 1;
        buffer.cursor_col = 10; // Beyond first line's length
        buffer.scroll_offset = 1;

        buffer.scroll_half_page_up(1);

        assert_eq!(buffer.cursor_line, 0);
        assert_eq!(buffer.cursor_col, 5); // Clamped to "short".len()
    }

    #[test]
    fn request_buffer_scroll_half_page_down_should_scroll_and_move_cursor() {
        let mut buffer = RequestBuffer::new();
        buffer.lines = vec![
            "line 0".to_string(),
            "line 1".to_string(),
            "line 2".to_string(),
            "line 3".to_string(),
            "line 4".to_string(),
            "line 5".to_string(),
        ];
        buffer.cursor_line = 1;
        buffer.cursor_col = 2;
        buffer.scroll_offset = 0;

        buffer.scroll_half_page_down(2);

        assert_eq!(buffer.scroll_offset, 2);
        assert_eq!(buffer.cursor_line, 2); // Cursor moves to top of newly visible area
        assert_eq!(buffer.cursor_col, 2);
    }

    #[test]
    fn request_buffer_scroll_half_page_down_should_handle_insufficient_content() {
        let mut buffer = RequestBuffer::new();
        buffer.lines = vec!["line 0".to_string(), "line 1".to_string()];
        buffer.cursor_line = 0;
        buffer.scroll_offset = 0;

        buffer.scroll_half_page_down(5); // Request more than available

        assert_eq!(buffer.scroll_offset, 0); // No scroll possible with short content
        assert_eq!(buffer.cursor_line, 0);
    }

    #[test]
    fn request_buffer_scroll_half_page_down_should_limit_scroll_to_available_space() {
        let mut buffer = RequestBuffer::new();
        buffer.lines = vec![
            "line 0".to_string(),
            "line 1".to_string(),
            "line 2".to_string(),
            "line 3".to_string(),
        ];
        buffer.cursor_line = 0;
        buffer.scroll_offset = 0;

        buffer.scroll_half_page_down(3); // Page height would be 6, request scroll 3

        assert_eq!(buffer.scroll_offset, 0); // No scroll possible: 4 lines - 6 page height = max scroll of 0
        assert_eq!(buffer.cursor_line, 0);
    }

    #[test]
    fn request_buffer_scroll_half_page_down_should_clamp_cursor_column_to_line_bounds() {
        let mut buffer = RequestBuffer::new();
        buffer.lines = vec![
            "long line here".to_string(),
            "short".to_string(),
            "another long line here".to_string(),
        ];
        buffer.cursor_line = 0;
        buffer.cursor_col = 10; // Valid for first line
        buffer.scroll_offset = 0;

        buffer.scroll_half_page_down(1);

        assert_eq!(buffer.scroll_offset, 1);
        assert_eq!(buffer.cursor_line, 1); // Move to top of visible area
        assert_eq!(buffer.cursor_col, 5); // Clamped to "short".len()
    }

    #[test]
    fn response_buffer_new_should_create_buffer_from_content() {
        let content = "HTTP/1.1 200 OK\nContent-Type: application/json\n\n{\"result\": true}";
        let buffer = ResponseBuffer::new(content.to_string());

        assert_eq!(buffer.lines.len(), 4);
        assert_eq!(buffer.lines[0], "HTTP/1.1 200 OK");
        assert_eq!(buffer.lines[1], "Content-Type: application/json");
        assert_eq!(buffer.lines[2], "");
        assert_eq!(buffer.lines[3], "{\"result\": true}");
        assert_eq!(buffer.cursor_line, 0);
        assert_eq!(buffer.cursor_col, 0);
        assert_eq!(buffer.scroll_offset, 0);
    }

    #[test]
    fn response_buffer_current_line_should_return_line_at_cursor_position() {
        let buffer = ResponseBuffer::new("line1\nline2\nline3".to_string());

        assert_eq!(buffer.current_line(), "line1");
    }

    #[test]
    fn response_buffer_line_count_should_return_number_of_lines() {
        let buffer = ResponseBuffer::new("line1\nline2\nline3".to_string());

        assert_eq!(buffer.line_count(), 3);
    }

    #[test]
    fn response_buffer_visible_range_should_work_like_request_buffer() {
        let mut buffer = ResponseBuffer::new("1\n2\n3\n4\n5".to_string());
        buffer.scroll_offset = 1;

        let (start, end) = buffer.visible_range(3);
        assert_eq!(start, 1);
        assert_eq!(end, 4);
    }

    #[test]
    fn response_buffer_is_cursor_valid_should_work_correctly() {
        let mut buffer = ResponseBuffer::new("hello world".to_string());
        buffer.cursor_line = 0;
        buffer.cursor_col = 5;

        assert!(buffer.is_cursor_valid());

        buffer.cursor_col = 50;
        assert!(!buffer.is_cursor_valid());
    }

    #[test]
    fn response_buffer_clamp_cursor_should_fix_invalid_positions() {
        let mut buffer = ResponseBuffer::new("hello".to_string());
        buffer.cursor_line = 10;
        buffer.cursor_col = 10;

        buffer.clamp_cursor();
        assert_eq!(buffer.cursor_line, 0);
        assert_eq!(buffer.cursor_col, 5);
    }

    #[test]
    fn response_buffer_scroll_should_handle_zero_lines() {
        let mut buffer = ResponseBuffer::new("line 0\nline 1".to_string());
        buffer.cursor_line = 1;
        buffer.scroll_offset = 0;

        buffer.scroll(0, 10); // No scroll

        assert_eq!(buffer.scroll_offset, 0);
        assert_eq!(buffer.cursor_line, 1);
    }

    #[test]
    fn response_buffer_scroll_should_scroll_down_and_move_cursor_to_top() {
        let mut buffer = ResponseBuffer::new("line 0\nline 1\nline 2\nline 3\nline 4\nline 5".to_string());
        buffer.cursor_line = 1;
        buffer.scroll_offset = 0;

        buffer.scroll(2, 4); // Scroll down 2 lines with page height 4

        assert_eq!(buffer.scroll_offset, 2);
        assert_eq!(buffer.cursor_line, 2); // Cursor moves to top of visible area
    }

    #[test]
    fn response_buffer_scroll_should_scroll_up_and_move_cursor_to_top() {
        let mut buffer = ResponseBuffer::new("line 0\nline 1\nline 2\nline 3\nline 4\nline 5".to_string());
        buffer.cursor_line = 4;
        buffer.scroll_offset = 3;

        buffer.scroll(-2, 4); // Scroll up 2 lines with page height 4

        assert_eq!(buffer.scroll_offset, 1);
        assert_eq!(buffer.cursor_line, 1); // Cursor moves to top of visible area
    }

    #[test]
    fn response_buffer_scroll_should_limit_scroll_down_to_available_space() {
        let mut buffer = ResponseBuffer::new("line 0\nline 1\nline 2".to_string());
        buffer.cursor_line = 0;
        buffer.scroll_offset = 0;

        buffer.scroll(10, 2); // Request more than available

        assert_eq!(buffer.scroll_offset, 1); // Limited to max available (3 lines - 2 page height)
        assert_eq!(buffer.cursor_line, 1);
    }

    #[test]
    fn response_buffer_scroll_should_limit_scroll_up_to_available_offset() {
        let mut buffer = ResponseBuffer::new("line 0\nline 1\nline 2".to_string());
        buffer.cursor_line = 2;
        buffer.scroll_offset = 1;

        buffer.scroll(-10, 2); // Request more than available

        assert_eq!(buffer.scroll_offset, 0); // Limited to available offset
        assert_eq!(buffer.cursor_line, 0);
    }

    #[test]
    fn response_buffer_scroll_half_page_up_should_scroll_and_move_cursor() {
        let mut buffer =
            ResponseBuffer::new("line 0\nline 1\nline 2\nline 3\nline 4\nline 5".to_string());
        buffer.cursor_line = 4;
        buffer.cursor_col = 2;
        buffer.scroll_offset = 2;

        buffer.scroll_half_page_up(2);

        assert_eq!(buffer.scroll_offset, 0);
        assert_eq!(buffer.cursor_line, 0); // Cursor moves to top of visible area (vim behavior)
        assert_eq!(buffer.cursor_col, 2);
    }

    #[test]
    fn response_buffer_scroll_half_page_up_should_handle_zero_scroll_offset() {
        let mut buffer = ResponseBuffer::new("line 0\nline 1".to_string());
        buffer.cursor_line = 1;
        buffer.scroll_offset = 0;

        buffer.scroll_half_page_up(5);

        // Should not change anything when already at top
        assert_eq!(buffer.scroll_offset, 0);
        assert_eq!(buffer.cursor_line, 1); // Cursor should remain unchanged
    }

    #[test]
    fn response_buffer_scroll_half_page_down_should_scroll_and_move_cursor() {
        let mut buffer = ResponseBuffer::new("line 0\nline 1\nline 2\nline 3\nline 4\nline 5".to_string());
        buffer.cursor_line = 1;
        buffer.cursor_col = 2;
        buffer.scroll_offset = 0;

        buffer.scroll_half_page_down(2);

        assert_eq!(buffer.scroll_offset, 2);
        assert_eq!(buffer.cursor_line, 2); // Cursor moves to top of newly visible area
        assert_eq!(buffer.cursor_col, 2);
    }

    #[test]
    fn response_buffer_scroll_half_page_down_should_handle_insufficient_content() {
        let mut buffer = ResponseBuffer::new("line 0\nline 1".to_string());
        buffer.cursor_line = 0;
        buffer.scroll_offset = 0;

        buffer.scroll_half_page_down(5); // Request more than available

        assert_eq!(buffer.scroll_offset, 0); // No scroll possible with short content
        assert_eq!(buffer.cursor_line, 0);
    }

    #[test]
    fn app_state_new_should_create_state_with_correct_defaults() {
        let state = AppState::new((80, 24), true);

        assert_eq!(state.mode, EditorMode::Normal);
        assert_eq!(state.current_pane, Pane::Request);
        assert!(state.visual_selection.is_none());
        assert_eq!(state.request_buffer.lines.len(), 1);
        assert!(state.response_buffer.is_none());
        assert_eq!(state.terminal_size, (80, 24));
        assert_eq!(state.status_message, "-- INSERT --");
        assert_eq!(state.command_buffer, "");
        assert!(state.session_headers.is_empty());
        assert_eq!(state.clipboard, "");
        assert!(!state.pending_g);
        assert!(!state.pending_ctrl_w);
        assert!(state.last_response_status.is_none());
        assert!(state.request_start_time.is_none());
        assert!(state.last_request_duration.is_none());
        assert!(!state.execute_request_flag);
        assert!(!state.should_quit);
        assert!(state.verbose);
    }

    #[test]
    fn app_state_new_should_calculate_initial_pane_height() {
        let state = AppState::new((80, 24), false);

        // Terminal height 24, minus separator and status = 22, divided by 2 = 11
        assert_eq!(state.request_pane_height, 11);
    }

    #[test]
    fn app_state_get_request_pane_height_should_return_full_space_when_no_response() {
        let state = AppState::new((80, 24), false);

        // Should get full available space (24 - 2 = 22) when no response buffer
        assert_eq!(state.get_request_pane_height(), 22);
    }

    #[test]
    fn app_state_get_request_pane_height_should_return_configured_height_when_response_exists() {
        let mut state = AppState::new((80, 24), false);
        state.set_response("test response".to_string());

        // Should return the configured split height when response exists
        assert_eq!(state.get_request_pane_height(), state.request_pane_height);
    }

    #[test]
    fn app_state_get_response_pane_height_should_return_zero_when_no_response() {
        let state = AppState::new((80, 24), false);

        assert_eq!(state.get_response_pane_height(), 0);
    }

    #[test]
    fn app_state_get_response_pane_height_should_return_remaining_space_when_response_exists() {
        let mut state = AppState::new((80, 24), false);
        state.set_response("test response".to_string());

        // Total content height = 24 - 2 = 22
        // Request pane height = 11 (from initial calculation)
        // Response pane height = 22 - 11 = 11
        assert_eq!(state.get_response_pane_height(), 11);
    }

    #[test]
    fn app_state_update_terminal_size_should_maintain_proportions() {
        let mut state = AppState::new((80, 20), false);
        let initial_request_height = state.request_pane_height;

        state.update_terminal_size((80, 40));

        // Should roughly double the pane height when terminal doubles in height
        let new_request_height = state.get_request_pane_height();
        assert!(new_request_height > initial_request_height);
        assert_eq!(state.terminal_size, (80, 40));
    }

    #[test]
    fn app_state_set_response_should_create_response_buffer() {
        let mut state = AppState::new((80, 24), false);

        state.set_response("HTTP/1.1 200 OK\nContent: test".to_string());

        assert!(state.response_buffer.is_some());
        let response = state.response_buffer.as_ref().unwrap();
        assert_eq!(response.lines.len(), 2);
        assert_eq!(response.lines[0], "HTTP/1.1 200 OK");
    }

    #[test]
    fn app_state_clear_response_should_remove_response_buffer() {
        let mut state = AppState::new((80, 24), false);
        state.set_response("test".to_string());

        state.clear_response();

        assert!(state.response_buffer.is_none());
    }

    #[test]
    fn app_state_has_response_should_return_correct_status() {
        let mut state = AppState::new((80, 24), false);

        assert!(!state.has_response());

        state.set_response("test".to_string());
        assert!(state.has_response());

        state.clear_response();
        assert!(!state.has_response());
    }

    #[test]
    fn app_state_current_buffer_should_return_request_when_in_request_pane() {
        let state = AppState::new((80, 24), false);

        match state.current_buffer() {
            CurrentBuffer::Request(_) => (), // Expected
            CurrentBuffer::Response(_) => panic!("Should return request buffer"),
        }
    }

    #[test]
    fn app_state_current_buffer_should_return_response_when_in_response_pane_with_response() {
        let mut state = AppState::new((80, 24), false);
        state.set_response("test".to_string());
        state.current_pane = Pane::Response;

        match state.current_buffer() {
            CurrentBuffer::Response(_) => (), // Expected
            CurrentBuffer::Request(_) => panic!("Should return response buffer"),
        }
    }

    #[test]
    fn app_state_current_buffer_should_fallback_to_request_when_no_response() {
        let mut state = AppState::new((80, 24), false);
        state.current_pane = Pane::Response;

        match state.current_buffer() {
            CurrentBuffer::Request(_) => (), // Expected fallback
            CurrentBuffer::Response(_) => panic!("Should fallback to request buffer"),
        }
    }

    #[test]
    fn current_buffer_cursor_line_should_return_correct_position() {
        let mut state = AppState::new((80, 24), false);
        state.request_buffer.cursor_line = 5;

        let buffer = state.current_buffer();
        assert_eq!(buffer.cursor_line(), 5);
    }

    #[test]
    fn current_buffer_cursor_col_should_return_correct_position() {
        let mut state = AppState::new((80, 24), false);
        state.request_buffer.cursor_col = 10;

        let buffer = state.current_buffer();
        assert_eq!(buffer.cursor_col(), 10);
    }

    #[test]
    fn current_buffer_line_count_should_return_correct_count() {
        let mut state = AppState::new((80, 24), false);
        state.request_buffer.lines = vec!["1".to_string(), "2".to_string(), "3".to_string()];

        let buffer = state.current_buffer();
        assert_eq!(buffer.line_count(), 3);
    }

    #[test]
    fn visual_selection_should_support_debug_trait() {
        let selection = VisualSelection {
            start_line: 0,
            start_col: 0,
            end_line: 1,
            end_col: 5,
        };

        let debug_str = format!("{:?}", selection);
        assert!(debug_str.contains("VisualSelection"));
    }

    #[test]
    fn editor_mode_should_support_debug_and_clone_traits() {
        let mode = EditorMode::Normal;
        let cloned_mode = mode.clone();

        assert_eq!(mode, cloned_mode);

        let debug_str = format!("{:?}", mode);
        assert!(debug_str.contains("Normal"));
    }

    #[test]
    fn pane_should_support_copy_and_debug_traits() {
        let pane = Pane::Request;
        let copied_pane = pane;

        assert_eq!(pane, copied_pane);

        let debug_str = format!("{:?}", pane);
        assert!(debug_str.contains("Request"));
    }
}
