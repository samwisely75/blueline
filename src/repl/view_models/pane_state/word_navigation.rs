//! Word navigation functionality for PaneState
//!
//! This module contains methods for:
//! - Finding next/previous word start positions
//! - Finding next word end positions
//! - Cross-line word navigation
//! - Support for Japanese and multi-byte character word boundaries

use crate::repl::events::{EditorMode, LogicalPosition, PaneCapabilities, ViewEvent};
use crate::repl::models::geometry::Position;

use super::{OptionalPosition, PaneState};

impl PaneState {
    /// Find the position of the beginning of the next word from current position
    /// Returns None if no next word is found
    /// Now supports Japanese characters as word characters
    pub fn find_next_word_start_position(&self, current_pos: Position) -> OptionalPosition {
        let mut current_line = current_pos.row;
        let mut current_col = current_pos.col;
        // Loop through display lines to find next word
        while current_line < self.display_cache.display_line_count() {
            if let Some(line_info) = self.display_cache.get_display_line(current_line) {
                // Try to find next word on current line
                if let Some(new_col) = line_info.find_next_word_start(current_col) {
                    return Some(Position::new(current_line, new_col));
                }
                // Move to next line and start at beginning
                current_line += 1;
                current_col = 0;
                // If we moved to next line, look for first word on that line
                if current_line < self.display_cache.display_line_count() {
                    if let Some(next_line_info) = self.display_cache.get_display_line(current_line)
                    {
                        if let Some(new_col) = next_line_info.find_next_word_start(0) {
                            return Some(Position::new(current_line, new_col));
                        }
                    }
                }
            } else {
                break;
            }
        }
        None
    }

    /// Find the position of the beginning of the previous word from current position
    /// Returns None if no previous word is found
    /// Now supports Japanese characters as word characters
    pub fn find_previous_word_start_position(&self, current_pos: Position) -> OptionalPosition {
        let mut current_line = current_pos.row;
        let mut current_col = current_pos.col;
        tracing::debug!(
            "find_previous_word_start_position: starting at display_pos=({}, {})",
            current_line,
            current_col
        );
        // Loop through display lines backwards to find previous word
        while let Some(line_info) = self.display_cache.get_display_line(current_line) {
            tracing::debug!("find_previous_word_start_position: checking line {} with {} chars, display_width={}, current_col={}", 
                current_line, line_info.char_count(), line_info.display_width(), current_col);
            // Try to find previous word on current line
            if let Some(new_col) = line_info.find_previous_word_start(current_col) {
                tracing::debug!(
                    "find_previous_word_start_position: found word on line {} at col {}",
                    current_line,
                    new_col
                );
                return Some(Position::new(current_line, new_col));
            }
            tracing::debug!(
                "find_previous_word_start_position: no word found on line {}, moving to previous line",
                current_line
            );
            // If we can't find a previous word on this line, move to previous line
            if current_line > 0 {
                current_line -= 1;
                if let Some(prev_line_info) = self.display_cache.get_display_line(current_line) {
                    current_col = prev_line_info.display_width();
                    tracing::debug!("find_previous_word_start_position: moved to line {}, set current_col to display_width={}", 
                        current_line, current_col);
                    // Try to find previous word from the end of the previous line
                    if let Some(new_col) = prev_line_info.find_previous_word_start(current_col) {
                        tracing::debug!(
                            "find_previous_word_start_position: found word on prev line {} at col {}",
                            current_line,
                            new_col
                        );
                        return Some(Position::new(current_line, new_col));
                    }
                    tracing::debug!(
                        "find_previous_word_start_position: no word found on prev line {}",
                        current_line
                    );
                }
            } else {
                break; // Already at beginning of buffer
            }
        }
        None
    }

    /// Find the position of the end of the current or next word from current position
    /// Returns None if no word end is found
    /// Now supports Japanese characters as word characters
    pub fn find_next_word_end_position(&self, current_pos: Position) -> OptionalPosition {
        let mut current_line = current_pos.row;
        let mut current_col = current_pos.col;
        // Loop through display lines to find end of word
        while current_line < self.display_cache.display_line_count() {
            if let Some(line_info) = self.display_cache.get_display_line(current_line) {
                // Try to find end of word on current line
                if let Some(new_col) = line_info.find_next_word_end(current_col) {
                    return Some(Position::new(current_line, new_col));
                }
                // Move to next line
                current_line += 1;
                current_col = 0;
                // Try to find end of word on next line from beginning
                if current_line < self.display_cache.display_line_count() {
                    if let Some(next_line_info) = self.display_cache.get_display_line(current_line)
                    {
                        if let Some(new_col) = next_line_info.find_next_word_end(0) {
                            return Some(Position::new(current_line, new_col));
                        }
                    }
                }
            } else {
                break;
            }
        }
        None
    }

    // ========================================
    // Word Movement Methods
    // ========================================

    /// Move cursor to next word with capability checking and Visual Block restrictions
    pub fn move_cursor_to_next_word(&mut self, content_width: usize) -> Vec<ViewEvent> {
        // Check if navigation is allowed on this pane
        if !self.capabilities.contains(PaneCapabilities::NAVIGABLE) {
            return vec![]; // Navigation not allowed on this pane
        }

        let current_display_pos = self.display_cursor;
        let current_mode = self.editor_mode;

        if let Some(new_pos) = self.find_next_word_start_position(current_display_pos) {
            // VISUAL BLOCK FIX: In Visual Block mode, prevent moving to different lines
            if current_mode == EditorMode::VisualBlock && new_pos.row != current_display_pos.row {
                return vec![]; // Don't move if it would cross lines
            }

            // Update display cursor
            self.display_cursor = new_pos;
            // Update virtual column for horizontal movement
            self.update_virtual_column();

            // Sync logical cursor with new display position
            if let Some(logical_pos) = self
                .display_cache
                .display_to_logical_position(new_pos.row, new_pos.col)
            {
                let new_logical_pos = LogicalPosition::new(logical_pos.row, logical_pos.col);
                self.buffer.set_cursor(new_logical_pos);

                // Update visual selection if active
                self.update_visual_selection_on_cursor_move(new_logical_pos);
            }

            let mut events = vec![
                ViewEvent::ActiveCursorUpdateRequired,
                ViewEvent::PositionIndicatorUpdateRequired,
                ViewEvent::CurrentAreaRedrawRequired,
            ];

            // Ensure cursor is visible and add visibility events
            let visibility_events = self.ensure_cursor_visible_with_events(content_width);
            events.extend(visibility_events);

            events
        } else {
            vec![]
        }
    }

    /// Move cursor to previous word with capability checking and Visual Block restrictions
    pub fn move_cursor_to_previous_word(&mut self, content_width: usize) -> Vec<ViewEvent> {
        // Check if navigation is allowed on this pane
        if !self.capabilities.contains(PaneCapabilities::NAVIGABLE) {
            return vec![]; // Navigation not allowed on this pane
        }

        let current_display_pos = self.display_cursor;
        let current_mode = self.editor_mode;

        if let Some(new_pos) = self.find_previous_word_start_position(current_display_pos) {
            // VISUAL BLOCK FIX: In Visual Block mode, prevent moving to different lines
            if current_mode == EditorMode::VisualBlock && new_pos.row != current_display_pos.row {
                return vec![]; // Don't move if it would cross lines
            }

            // Update display cursor
            self.display_cursor = new_pos;
            // Update virtual column for horizontal movement
            self.update_virtual_column();

            // Sync logical cursor with new display position
            if let Some(logical_pos) = self
                .display_cache
                .display_to_logical_position(new_pos.row, new_pos.col)
            {
                let new_logical_pos = LogicalPosition::new(logical_pos.row, logical_pos.col);
                self.buffer.set_cursor(new_logical_pos);

                // Update visual selection if active
                self.update_visual_selection_on_cursor_move(new_logical_pos);
            }

            let mut events = vec![
                ViewEvent::ActiveCursorUpdateRequired,
                ViewEvent::PositionIndicatorUpdateRequired,
                ViewEvent::CurrentAreaRedrawRequired,
            ];

            // Ensure cursor is visible and add visibility events
            let visibility_events = self.ensure_cursor_visible_with_events(content_width);
            events.extend(visibility_events);

            events
        } else {
            vec![]
        }
    }

    /// Move cursor to end of word with capability checking and Visual Block restrictions
    pub fn move_cursor_to_end_of_word(&mut self, content_width: usize) -> Vec<ViewEvent> {
        // Check if navigation is allowed on this pane
        if !self.capabilities.contains(PaneCapabilities::NAVIGABLE) {
            return vec![]; // Navigation not allowed on this pane
        }

        let current_display_pos = self.display_cursor;
        let current_mode = self.editor_mode;

        if let Some(new_pos) = self.find_next_word_end_position(current_display_pos) {
            // VISUAL BLOCK FIX: In Visual Block mode, prevent moving to different lines
            if current_mode == EditorMode::VisualBlock && new_pos.row != current_display_pos.row {
                return vec![]; // Don't move if it would cross lines
            }

            // Update display cursor
            self.display_cursor = new_pos;
            // Update virtual column for horizontal movement
            self.update_virtual_column();

            // Sync logical cursor with new display position
            if let Some(logical_pos) = self
                .display_cache
                .display_to_logical_position(new_pos.row, new_pos.col)
            {
                let new_logical_pos = LogicalPosition::new(logical_pos.row, logical_pos.col);
                self.buffer.set_cursor(new_logical_pos);

                // Update visual selection if active
                self.update_visual_selection_on_cursor_move(new_logical_pos);
            }

            let mut events = vec![
                ViewEvent::ActiveCursorUpdateRequired,
                ViewEvent::PositionIndicatorUpdateRequired,
                ViewEvent::CurrentAreaRedrawRequired,
            ];

            // Ensure cursor is visible and add visibility events
            let visibility_events = self.ensure_cursor_visible_with_events(content_width);
            events.extend(visibility_events);

            events
        } else {
            vec![]
        }
    }
}
