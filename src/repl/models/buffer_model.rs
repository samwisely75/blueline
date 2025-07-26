//! Buffer model for MVVM architecture
//!
//! This model manages text buffer operations like content storage, 
//! text manipulation, and cursor movement within content bounds.

use crate::repl::events::{LogicalPosition, LogicalRange, ModelEvent};
use crate::repl::model::Pane;

/// Content of a text buffer as lines of text
#[derive(Debug, Clone, PartialEq)]
pub struct BufferContent {
    lines: Vec<String>,
}

impl BufferContent {
    /// Create new empty buffer content
    pub fn new() -> Self {
        Self {
            lines: vec![String::new()],
        }
    }
    
    /// Create buffer content from lines
    pub fn from_lines(lines: Vec<String>) -> Self {
        if lines.is_empty() {
            Self::new()
        } else {
            Self { lines }
        }
    }
    
    /// Get all lines
    pub fn lines(&self) -> &[String] {
        &self.lines
    }
    
    /// Get line count
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
    
    /// Insert text at a specific position
    /// Returns event describing the insertion
    pub fn insert_text(&mut self, pane: Pane, position: LogicalPosition, text: &str) -> ModelEvent {
        // Ensure we have enough lines
        while self.lines.len() <= position.line {
            self.lines.push(String::new());
        }
        
        if text.contains('\n') {
            // Multi-line insertion
            let lines: Vec<&str> = text.split('\n').collect();
            let current_line = &mut self.lines[position.line];
            
            // Split current line at insertion point
            let after_cursor = current_line.split_off(position.column);
            
            // Insert first part of new text to current line
            current_line.push_str(lines[0]);
            
            // Insert middle lines
            for (i, line) in lines.iter().enumerate().skip(1).take(lines.len() - 2) {
                self.lines.insert(position.line + i, line.to_string());
            }
            
            // Insert last line and append the text that was after cursor
            if lines.len() > 1 {
                let mut last_line = lines[lines.len() - 1].to_string();
                last_line.push_str(&after_cursor);
                self.lines.insert(position.line + lines.len() - 1, last_line);
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
    
    /// Delete text in a range
    /// Returns event describing the deletion
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
            // Multi-line deletion - handle borrowing carefully
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
    
    /// Insert a new line at the specified line index
    pub fn insert_line(&mut self, pane: Pane, line_index: usize, content: String) -> ModelEvent {
        if line_index <= self.lines.len() {
            self.lines.insert(line_index, content);
        } else {
            // Pad with empty lines if needed
            while self.lines.len() < line_index {
                self.lines.push(String::new());
            }
            self.lines.push(content);
        }
        
        ModelEvent::LineInserted { pane, line: line_index }
    }
    
    /// Delete a line at the specified index
    pub fn delete_line(&mut self, pane: Pane, line_index: usize) -> Option<ModelEvent> {
        if line_index < self.lines.len() && self.lines.len() > 1 {
            self.lines.remove(line_index);
            Some(ModelEvent::LineDeleted { pane, line: line_index })
        } else {
            None
        }
    }
    
    /// Check if a position is valid within this buffer
    pub fn is_valid_position(&self, position: LogicalPosition) -> bool {
        position.line < self.lines.len() && 
        position.column <= self.line_length(position.line)
    }
    
    /// Clamp a position to valid bounds within this buffer
    pub fn clamp_position(&self, position: LogicalPosition) -> LogicalPosition {
        let line = position.line.min(self.lines.len().saturating_sub(1));
        let column = position.column.min(self.line_length(line));
        LogicalPosition { line, column }
    }
}

impl Default for BufferContent {
    fn default() -> Self {
        Self::new()
    }
}

/// Buffer model that manages content for a specific pane
#[derive(Debug, Clone)]
pub struct BufferModel {
    content: BufferContent,
    pane: Pane,
}

impl BufferModel {
    /// Create a new buffer model for the specified pane
    pub fn new(pane: Pane) -> Self {
        Self {
            content: BufferContent::new(),
            pane,
        }
    }
    
    /// Create buffer model with initial content
    pub fn with_content(pane: Pane, content: BufferContent) -> Self {
        Self { content, pane }
    }
    
    /// Get the buffer content
    pub fn content(&self) -> &BufferContent {
        &self.content
    }
    
    /// Get mutable access to buffer content
    pub fn content_mut(&mut self) -> &mut BufferContent {
        &mut self.content
    }
    
    /// Set new content for this buffer
    pub fn set_content(&mut self, new_content: BufferContent) {
        self.content = new_content;
    }
    
    /// Move cursor left by one column, handling line wrapping
    /// Returns new cursor position and optional event
    pub fn move_cursor_left(&self, current_pos: LogicalPosition) -> (LogicalPosition, Option<ModelEvent>) {
        let old_pos = current_pos;
        
        if current_pos.column > 0 {
            // Move left within current line
            let new_pos = LogicalPosition {
                line: current_pos.line,
                column: current_pos.column - 1,
            };
            (new_pos, Some(ModelEvent::CursorMoved {
                pane: self.pane,
                old_pos,
                new_pos,
            }))
        } else if current_pos.line > 0 {
            // Move to end of previous line
            let new_pos = LogicalPosition {
                line: current_pos.line - 1,
                column: self.content.line_length(current_pos.line - 1),
            };
            (new_pos, Some(ModelEvent::CursorMoved {
                pane: self.pane,
                old_pos,
                new_pos,
            }))
        } else {
            // Already at start of content
            (current_pos, None)
        }
    }
    
    /// Move cursor right by one column, handling line wrapping
    /// Returns new cursor position and optional event
    pub fn move_cursor_right(&self, current_pos: LogicalPosition) -> (LogicalPosition, Option<ModelEvent>) {
        let old_pos = current_pos;
        let current_line_length = self.content.line_length(current_pos.line);
        
        if current_pos.column < current_line_length {
            // Move right within current line
            let new_pos = LogicalPosition {
                line: current_pos.line,
                column: current_pos.column + 1,
            };
            (new_pos, Some(ModelEvent::CursorMoved {
                pane: self.pane,
                old_pos,
                new_pos,
            }))
        } else if current_pos.line + 1 < self.content.line_count() {
            // Move to beginning of next line
            let new_pos = LogicalPosition {
                line: current_pos.line + 1,
                column: 0,
            };
            (new_pos, Some(ModelEvent::CursorMoved {
                pane: self.pane,
                old_pos,
                new_pos,
            }))
        } else {
            // Already at end of content
            (current_pos, None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_content() -> BufferContent {
        BufferContent::from_lines(vec![
            "line1".to_string(),
            "line2 with more text".to_string(),
            "line3".to_string(),
        ])
    }

    #[test]
    fn buffer_content_should_create_with_default_state() {
        let content = BufferContent::new();
        
        assert_eq!(content.line_count(), 1);
        assert_eq!(content.get_line(0), Some(&String::new()));
        assert_eq!(content.line_length(0), 0);
    }

    #[test]
    fn buffer_content_should_handle_empty_lines() {
        let content = BufferContent::from_lines(vec![]);
        
        assert_eq!(content.line_count(), 1);
        assert_eq!(content.get_line(0), Some(&String::new()));
    }

    #[test]
    fn buffer_content_should_calculate_line_lengths() {
        let content = create_test_content();
        
        assert_eq!(content.line_length(0), 5);  // "line1"
        assert_eq!(content.line_length(1), 20); // "line2 with more text"
        assert_eq!(content.line_length(2), 5);  // "line3"
        assert_eq!(content.line_length(10), 0); // Out of bounds
    }

    #[test]
    fn buffer_content_should_insert_single_line_text() {
        let mut content = create_test_content();
        let position = LogicalPosition { line: 1, column: 5 };
        
        let event = content.insert_text(Pane::Request, position, " inserted");
        
        assert_eq!(content.get_line(1).unwrap(), "line2 inserted with more text");
        match event {
            ModelEvent::TextInserted { pane, position: event_pos, text } => {
                assert_eq!(pane, Pane::Request);
                assert_eq!(event_pos, position);
                assert_eq!(text, " inserted");
            }
            _ => panic!("Expected TextInserted event"),
        }
    }

    #[test]
    fn buffer_model_should_move_cursor_left_within_line() {
        let mut model = BufferModel::new(Pane::Request);
        let _ = model.content_mut().insert_text(Pane::Request, LogicalPosition { line: 0, column: 0 }, "test");
        let current_pos = LogicalPosition { line: 0, column: 2 };
        
        let (new_pos, event) = model.move_cursor_left(current_pos);
        
        assert_eq!(new_pos, LogicalPosition { line: 0, column: 1 });
        assert!(event.is_some());
    }

    #[test]
    fn buffer_model_should_move_cursor_left_to_previous_line() {
        let model = BufferModel::with_content(Pane::Request, create_test_content());
        let current_pos = LogicalPosition { line: 1, column: 0 };
        
        let (new_pos, event) = model.move_cursor_left(current_pos);
        
        assert_eq!(new_pos, LogicalPosition { line: 0, column: 5 }); // End of "line1"
        assert!(event.is_some());
    }

    #[test]
    fn buffer_model_should_not_move_cursor_left_at_start() {
        let model = BufferModel::new(Pane::Request);
        let current_pos = LogicalPosition { line: 0, column: 0 };
        
        let (new_pos, event) = model.move_cursor_left(current_pos);
        
        assert_eq!(new_pos, current_pos);
        assert!(event.is_none());
    }

    #[test]
    fn buffer_model_should_move_cursor_right_within_line() {
        let model = BufferModel::with_content(Pane::Request, create_test_content());
        let current_pos = LogicalPosition { line: 1, column: 5 };
        
        let (new_pos, event) = model.move_cursor_right(current_pos);
        
        assert_eq!(new_pos, LogicalPosition { line: 1, column: 6 });
        assert!(event.is_some());
    }

    #[test]
    fn buffer_model_should_move_cursor_right_to_next_line() {
        let model = BufferModel::with_content(Pane::Request, create_test_content());
        let current_pos = LogicalPosition { line: 1, column: 20 }; // End of "line2 with more text"
        
        let (new_pos, event) = model.move_cursor_right(current_pos);
        
        assert_eq!(new_pos, LogicalPosition { line: 2, column: 0 });
        assert!(event.is_some());
    }

    #[test]
    fn buffer_model_should_not_move_cursor_right_at_end() {
        let model = BufferModel::with_content(Pane::Request, create_test_content());
        let current_pos = LogicalPosition { line: 2, column: 5 }; // End of "line3"
        
        let (new_pos, event) = model.move_cursor_right(current_pos);
        
        assert_eq!(new_pos, current_pos);
        assert!(event.is_none());
    }

    #[test]
    fn buffer_content_should_validate_positions() {
        let content = create_test_content();
        
        assert!(content.is_valid_position(LogicalPosition { line: 0, column: 5 }));
        assert!(content.is_valid_position(LogicalPosition { line: 1, column: 20 }));
        assert!(!content.is_valid_position(LogicalPosition { line: 3, column: 0 }));
        assert!(!content.is_valid_position(LogicalPosition { line: 0, column: 10 }));
    }

    #[test]
    fn buffer_content_should_clamp_positions() {
        let content = create_test_content();
        
        let clamped = content.clamp_position(LogicalPosition { line: 10, column: 50 });
        assert_eq!(clamped, LogicalPosition { line: 2, column: 5 }); // Last valid position
        
        let clamped = content.clamp_position(LogicalPosition { line: 1, column: 50 });
        assert_eq!(clamped, LogicalPosition { line: 1, column: 20 }); // End of line 1
    }
}