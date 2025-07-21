//! # Model - Application State and Data Structures
//!
//! This module contains all data structures for the REPL including buffers,
//! application state, and enums for modes and configuration.

#![allow(dead_code)] // Allow unused code during refactoring
#![allow(clippy::needless_lifetimes)] // Allow explicit lifetimes during refactoring
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
            mode: EditorMode::Insert, // Start in insert mode like original
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
            verbose,
        }
    }

    /// Get the height of the request pane in lines
    pub fn get_request_pane_height(&self) -> usize {
        self.request_pane_height
    }

    /// Get the height of the response pane in lines
    pub fn get_response_pane_height(&self) -> usize {
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

impl<'a> CurrentBuffer<'a> {
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
