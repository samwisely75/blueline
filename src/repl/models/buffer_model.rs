//! # Buffer Models
//!
//! Text buffer content and buffer model for MVVM architecture.
//! Handles text storage, cursor management, and basic editing operations.

use crate::repl::events::{LogicalPosition, LogicalRange, ModelEvent, Pane};

/// Content of a text buffer
#[derive(Debug, Clone, PartialEq)]
pub struct BufferContent {
    lines: Vec<String>,
}

impl BufferContent {
    /// Create new empty buffer
    pub fn new() -> Self {
        Self {
            lines: vec![String::new()],
        }
    }

    /// Create buffer from existing lines
    pub fn from_lines(lines: Vec<String>) -> Self {
        if lines.is_empty() {
            Self::new()
        } else {
            Self { lines }
        }
    }

    /// Get all lines as slice
    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    /// Get number of lines
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    /// Get specific line
    pub fn get_line(&self, index: usize) -> Option<&String> {
        self.lines.get(index)
    }

    /// Get line length (character count)
    pub fn line_length(&self, index: usize) -> usize {
        self.lines.get(index).map_or(0, |line| line.len())
    }

    /// Insert text at position, returning event
    pub fn insert_text(&mut self, pane: Pane, position: LogicalPosition, text: &str) -> ModelEvent {
        // Ensure we have enough lines
        while self.lines.len() <= position.line {
            self.lines.push(String::new());
        }

        if text.contains('\n') {
            // Multi-line insertion
            let text_lines: Vec<&str> = text.split('\n').collect();
            let current_line = &mut self.lines[position.line];

            // Split current line at insertion point
            let after_cursor = current_line.split_off(position.column);

            // Insert first part of new text
            current_line.push_str(text_lines[0]);

            // Insert middle lines
            for (i, line) in text_lines.iter().enumerate().skip(1) {
                if i == text_lines.len() - 1 {
                    // Last line - append the text that was after cursor
                    let mut last_line = line.to_string();
                    last_line.push_str(&after_cursor);
                    self.lines.insert(position.line + i, last_line);
                } else {
                    // Middle lines
                    self.lines.insert(position.line + i, line.to_string());
                }
            }
        } else {
            // Single line insertion
            let line = &mut self.lines[position.line];
            line.insert_str(position.column, text);
        }

        ModelEvent::TextInserted {
            pane,
            position,
            text: text.to_string(),
        }
    }

    /// Delete text in range, returning event if successful
    pub fn delete_range(&mut self, pane: Pane, range: LogicalRange) -> Option<ModelEvent> {
        if range.start.line >= self.lines.len() || range.end.line >= self.lines.len() {
            return None;
        }

        if range.start == range.end {
            return None; // Nothing to delete
        }

        if range.start.line == range.end.line {
            // Single line deletion
            let line = &mut self.lines[range.start.line];
            if range.end.column <= line.len() {
                line.drain(range.start.column..range.end.column);
            }
        } else {
            // Multi-line deletion
            let end_line_content = if range.end.line < self.lines.len() {
                self.lines[range.end.line][range.end.column..].to_string()
            } else {
                String::new()
            };

            // Truncate start line
            self.lines[range.start.line].truncate(range.start.column);

            // Remove lines in between
            for _ in range.start.line + 1..=range.end.line {
                if range.start.line + 1 < self.lines.len() {
                    self.lines.remove(range.start.line + 1);
                }
            }

            // Append remaining content from end line
            self.lines[range.start.line].push_str(&end_line_content);
        }

        Some(ModelEvent::TextDeleted { pane, range })
    }

    /// Check if position is valid within this buffer
    pub fn is_valid_position(&self, position: LogicalPosition) -> bool {
        position.line < self.lines.len() && position.column <= self.line_length(position.line)
    }

    /// Clamp position to valid bounds
    pub fn clamp_position(&self, position: LogicalPosition) -> LogicalPosition {
        let line = position.line.min(self.lines.len().saturating_sub(1));
        let column = position.column.min(self.line_length(line));
        LogicalPosition::new(line, column)
    }

    /// Get text content as single string
    pub fn get_text(&self) -> String {
        self.lines.join("\n")
    }

    /// Set entire content from string
    pub fn set_text(&mut self, text: &str) {
        self.lines = if text.is_empty() {
            vec![String::new()]
        } else {
            text.lines().map(|s| s.to_string()).collect()
        };
    }
}

impl Default for BufferContent {
    fn default() -> Self {
        Self::new()
    }
}

/// Text buffer model for a specific pane
#[derive(Debug, Clone)]
pub struct BufferModel {
    content: BufferContent,
    pane: Pane,
    cursor: LogicalPosition,
    scroll_offset: usize,
}

impl BufferModel {
    /// Create new buffer for pane
    pub fn new(pane: Pane) -> Self {
        Self {
            content: BufferContent::new(),
            pane,
            cursor: LogicalPosition::zero(),
            scroll_offset: 0,
        }
    }

    /// Get buffer content
    pub fn content(&self) -> &BufferContent {
        &self.content
    }

    /// Get mutable buffer content
    pub fn content_mut(&mut self) -> &mut BufferContent {
        &mut self.content
    }

    /// Get current cursor position
    pub fn cursor(&self) -> LogicalPosition {
        self.cursor
    }

    /// Set cursor position (clamped to valid bounds)
    pub fn set_cursor(&mut self, position: LogicalPosition) -> Option<ModelEvent> {
        let old_pos = self.cursor;
        let new_pos = self.content.clamp_position(position);

        if old_pos != new_pos {
            self.cursor = new_pos;
            Some(ModelEvent::CursorMoved {
                pane: self.pane,
                old_pos,
                new_pos,
            })
        } else {
            None
        }
    }

    /// Get scroll offset
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// Set scroll offset
    pub fn set_scroll_offset(&mut self, offset: usize) {
        self.scroll_offset = offset;
    }

    /// Move cursor left, returning new position and event
    pub fn move_cursor_left(&mut self) -> Option<ModelEvent> {
        let current = self.cursor;

        if current.column > 0 {
            // Move left within current line
            self.set_cursor(LogicalPosition::new(current.line, current.column - 1))
        } else if current.line > 0 {
            // Move to end of previous line
            let prev_line = current.line - 1;
            let line_length = self.content.line_length(prev_line);
            self.set_cursor(LogicalPosition::new(prev_line, line_length))
        } else {
            None // Already at start of buffer
        }
    }

    /// Move cursor right, returning new position and event
    pub fn move_cursor_right(&mut self) -> Option<ModelEvent> {
        let current = self.cursor;
        let line_length = self.content.line_length(current.line);

        if current.column < line_length {
            // Move right within current line
            self.set_cursor(LogicalPosition::new(current.line, current.column + 1))
        } else if current.line + 1 < self.content.line_count() {
            // Move to start of next line
            self.set_cursor(LogicalPosition::new(current.line + 1, 0))
        } else {
            None // Already at end of buffer
        }
    }

    /// Insert character at cursor, returning event
    pub fn insert_char(&mut self, ch: char) -> ModelEvent {
        let event = self
            .content
            .insert_text(self.pane, self.cursor, &ch.to_string());

        // Move cursor forward
        self.cursor = LogicalPosition::new(self.cursor.line, self.cursor.column + 1);

        event
    }

    /// Insert text at cursor, returning event
    pub fn insert_text(&mut self, text: &str) -> ModelEvent {
        let event = self.content.insert_text(self.pane, self.cursor, text);

        // Update cursor position based on inserted text
        if text.contains('\n') {
            let lines: Vec<&str> = text.split('\n').collect();
            let new_line = self.cursor.line + lines.len() - 1;
            let new_column = if lines.len() > 1 {
                lines.last().unwrap().len()
            } else {
                self.cursor.column + text.len()
            };
            self.cursor = LogicalPosition::new(new_line, new_column);
        } else {
            self.cursor = LogicalPosition::new(self.cursor.line, self.cursor.column + text.len());
        }

        event
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buffer_content_should_create_with_empty_line() {
        let content = BufferContent::new();
        assert_eq!(content.line_count(), 1);
        assert_eq!(content.get_line(0), Some(&String::new()));
    }

    #[test]
    fn buffer_content_should_insert_single_char() {
        let mut content = BufferContent::new();
        let pos = LogicalPosition::new(0, 0);

        content.insert_text(Pane::Request, pos, "a");

        assert_eq!(content.get_text(), "a");
    }

    #[test]
    fn buffer_content_should_insert_multiline_text() {
        let mut content = BufferContent::new();
        let pos = LogicalPosition::new(0, 0);

        content.insert_text(Pane::Request, pos, "hello\nworld");

        assert_eq!(content.line_count(), 2);
        assert_eq!(content.get_line(0), Some(&"hello".to_string()));
        assert_eq!(content.get_line(1), Some(&"world".to_string()));
    }

    #[test]
    fn buffer_model_should_move_cursor_left() {
        let mut buffer = BufferModel::new(Pane::Request);
        buffer.insert_text("hello");
        // Now cursor is at (0, 5), move it left

        let event = buffer.move_cursor_left();

        assert!(event.is_some());
        assert_eq!(buffer.cursor(), LogicalPosition::new(0, 4));
    }

    #[test]
    fn buffer_model_should_move_cursor_right() {
        let mut buffer = BufferModel::new(Pane::Request);
        buffer.insert_text("hello");
        buffer.cursor = LogicalPosition::new(0, 2);

        let event = buffer.move_cursor_right();

        assert!(event.is_some());
        assert_eq!(buffer.cursor(), LogicalPosition::new(0, 3));
    }
}
