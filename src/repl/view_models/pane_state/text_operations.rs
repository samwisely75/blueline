//! Text editing operations for PaneState
//!
//! This module contains methods for:
//! - Character insertion and deletion
//! - Text selection and extraction
//! - Line joining operations
//! - Visual mode text manipulation

use crate::repl::events::{
    EditorMode, LogicalPosition, LogicalRange, ModelEvent, PaneCapabilities, ViewEvent,
};
use crate::repl::geometry::Position;

use super::PaneState;

// Type alias for deletion operation results
type DeletionResult = Option<(String, ModelEvent)>;

impl PaneState {
    // Helper method to save last visual selection before clearing
    fn save_last_visual_selection_before_clear(&mut self) {
        if self.visual_selection_start.is_some() && self.visual_selection_end.is_some() {
            self.last_visual_selection_start = self.visual_selection_start;
            self.last_visual_selection_end = self.visual_selection_end;
            // Save which visual mode was active
            self.last_visual_mode = match self.editor_mode {
                EditorMode::Visual | EditorMode::VisualLine | EditorMode::VisualBlock => {
                    Some(self.editor_mode)
                }
                _ => None,
            };

            tracing::info!(
                "🎯 PaneState::save_last_visual_selection_before_clear - saved selection {:?} to {:?} in mode {:?}",
                self.last_visual_selection_start,
                self.last_visual_selection_end,
                self.last_visual_mode
            );
        }
    }

    /// Get the currently selected text in visual mode
    pub fn get_selected_text(&self) -> Option<String> {
        // Check if we have a selection
        let (Some(start), Some(end)) = (self.visual_selection_start, self.visual_selection_end)
        else {
            return None;
        };

        // Normalize selection (ensure start <= end)
        let (selection_start, selection_end) =
            if start.line < end.line || (start.line == end.line && start.column <= end.column) {
                (start, end)
            } else {
                (end, start)
            };

        let content = self.buffer.content();
        let mut selected_text = String::new();

        match self.editor_mode {
            EditorMode::VisualLine => {
                // Visual Line mode: always select entire lines from beginning to end
                let first_line = selection_start.line;
                let last_line = selection_end.line;

                for line_num in first_line..=last_line {
                    if let Some(line) = content.get_line(line_num) {
                        selected_text.push_str(&line);

                        // Add newline after each line except the last one
                        if line_num < last_line {
                            selected_text.push('\n');
                        }
                    }
                }
            }
            EditorMode::VisualBlock => {
                // Visual Block mode: select rectangular region
                // VISUAL BLOCK FIX: Use selection_start as the anchor column and selection_end as current cursor
                // This ensures selection always goes from the starting column to the current cursor column
                let top_line = selection_start.line;
                let bottom_line = selection_end.line;
                let start_col = selection_start.column; // Column where Visual Block mode started
                let end_col = selection_end.column; // Current cursor column

                // Determine selection direction and boundaries
                let (left_col, right_col) = if start_col <= end_col {
                    (start_col, end_col)
                } else {
                    (end_col, start_col)
                };

                for line_num in top_line..=bottom_line {
                    if let Some(line) = content.get_line(line_num) {
                        let chars: Vec<char> = line.chars().collect();
                        let line_char_length = chars.len();

                        // Skip lines that are too short to have content in the block region
                        if left_col < line_char_length {
                            let actual_right_col = (right_col + 1).min(line_char_length); // +1 to include character at end position
                            let selected_chars: String =
                                chars[left_col..actual_right_col].iter().collect();
                            selected_text.push_str(&selected_chars);
                        }

                        // Add newline after each line except the last one
                        if line_num < bottom_line {
                            selected_text.push('\n');
                        }
                    }
                }
            }
            _ => {
                // Visual mode (character-wise): original behavior
                if selection_start.line == selection_end.line {
                    // Single line selection
                    if let Some(line) = content.get_line(selection_start.line) {
                        let chars: Vec<char> = line.chars().collect();
                        let start_col = selection_start.column.min(chars.len());
                        let end_col = (selection_end.column + 1).min(chars.len()); // +1 to include character at end position
                        let selected_chars: String = chars[start_col..end_col].iter().collect();
                        selected_text.push_str(&selected_chars);
                    }
                } else {
                    // Multi-line selection
                    for line_num in selection_start.line..=selection_end.line {
                        if let Some(line) = content.get_line(line_num) {
                            if line_num == selection_start.line {
                                // First line: from start column to end
                                let chars: Vec<char> = line.chars().collect();
                                let start_col = selection_start.column.min(chars.len());
                                let selected_chars: String = chars[start_col..].iter().collect();
                                selected_text.push_str(&selected_chars);
                            } else if line_num == selection_end.line {
                                // Last line: from beginning to end column
                                let chars: Vec<char> = line.chars().collect();
                                let end_col = (selection_end.column + 1).min(chars.len());
                                let selected_chars: String = chars[..end_col].iter().collect();
                                selected_text.push_str(&selected_chars);
                            } else {
                                // Middle lines: entire line
                                selected_text.push_str(&line);
                            }

                            // Add newline between lines (but not after the last line)
                            if line_num < selection_end.line {
                                selected_text.push('\n');
                            }
                        }
                    }
                }
            }
        }

        if selected_text.is_empty() {
            None
        } else {
            Some(selected_text)
        }
    }

    /// Insert character at current cursor position with capability checking
    ///
    /// This method checks EDITABLE capability before allowing text insertion.
    /// It handles all the complex logic for display cache rebuilding, cursor
    /// synchronization, and view event generation.
    ///
    /// # Parameters
    /// - `ch`: Character to insert
    /// - `content_width`: Available width for content display
    /// - `wrap_enabled`: Whether text wrapping is enabled
    /// - `tab_width`: Tab stop width for character display
    ///
    /// # Returns
    /// Vector of ViewEvents to update the display, or empty if operation not allowed
    pub fn insert_char(
        &mut self,
        ch: char,
        content_width: usize,
        wrap_enabled: bool,
        tab_width: usize,
    ) -> Vec<ViewEvent> {
        // Check if editing is allowed on this pane
        if !self.capabilities.contains(PaneCapabilities::EDITABLE) {
            return vec![]; // Editing not allowed on this pane
        }

        // Insert character into buffer
        let _event = self.buffer.insert_char(ch);

        // Rebuild display cache to ensure rendering sees the updated content
        self.build_display_cache(content_width, wrap_enabled, tab_width);

        // Sync display cursor after cache rebuild
        let logical = self.buffer.cursor();
        if let Some(display_pos) = self
            .display_cache
            .logical_to_display_position(logical.line, logical.column)
        {
            self.display_cursor = display_pos;
        } else {
            // BUGFIX Issue #89: If logical_to_display_position fails, ensure cursor tracking doesn't break
            // This can happen with empty lines or edge cases after multiple newlines in Insert mode
            tracing::warn!(
                "logical_to_display_position failed for cursor at {:?} - using fallback display position", 
                logical
            );
            // Fallback: Use logical position as display position (works for non-wrapped content)
            self.display_cursor = Position::new(logical.line, logical.column);
        }

        // Return events for view updates - caller will handle cursor visibility
        vec![
            ViewEvent::RequestContentChanged,
            ViewEvent::ActiveCursorUpdateRequired,
            ViewEvent::PositionIndicatorUpdateRequired,
        ]
    }

    /// Delete character before cursor with capability checking
    ///
    /// This method checks EDITABLE capability before allowing character deletion.
    /// It handles two scenarios:
    /// 1. Delete character within current line (move cursor left)
    /// 2. Join with previous line when at beginning of line (backspace line join)
    ///
    /// # Parameters
    /// - `content_width`: Available width for content display  
    /// - `wrap_enabled`: Whether text wrapping is enabled
    /// - `tab_width`: Tab stop width for formatting
    ///
    /// # Returns
    /// Vector of ViewEvents to update the display, or empty if operation not allowed
    pub fn delete_char_before_cursor(
        &mut self,
        content_width: usize,
        wrap_enabled: bool,
        tab_width: usize,
    ) -> Vec<ViewEvent> {
        // Check if editing is allowed on this pane
        if !self.capabilities.contains(PaneCapabilities::EDITABLE) {
            return vec![]; // Editing not allowed on this pane
        }

        let current_cursor = self.buffer.cursor();

        tracing::debug!(
            "🗑️  PaneState::delete_char_before_cursor at position {:?}",
            current_cursor
        );

        // Dispatch to appropriate deletion method
        if current_cursor.column > 0 {
            self.delete_char_in_line(current_cursor, content_width, wrap_enabled, tab_width)
        } else if current_cursor.line > 0 {
            self.join_with_previous_line(current_cursor, content_width, wrap_enabled, tab_width)
        } else {
            tracing::debug!("🗑️  No deletion performed - at start of buffer");
            vec![]
        }
    }

    /// Delete character after cursor (Delete key)
    pub fn delete_char_after_cursor(
        &mut self,
        content_width: usize,
        wrap_enabled: bool,
        tab_width: usize,
    ) -> Vec<ViewEvent> {
        // Check if editing is allowed on this pane
        if !self.capabilities.contains(PaneCapabilities::EDITABLE) {
            return vec![]; // Editing not allowed on this pane
        }

        let current_cursor = self.buffer.cursor();

        tracing::debug!(
            "🗑️  PaneState::delete_char_after_cursor at position {:?}",
            current_cursor
        );

        // Get current line to check if we can delete within the line
        if let Some(current_line) = self.buffer.content().get_line(current_cursor.line) {
            if current_cursor.column < current_line.len() {
                // Delete character at cursor position (same line)
                self.delete_char_after_cursor_in_line(
                    current_cursor,
                    content_width,
                    wrap_enabled,
                    tab_width,
                )
            } else if current_cursor.line + 1 < self.buffer.content().line_count() {
                // At end of line, join with next line (delete key at line end)
                self.join_with_next_line(current_cursor, content_width, wrap_enabled, tab_width)
            } else {
                tracing::debug!("🗑️  No deletion performed - at end of buffer");
                vec![] // Nothing to delete (at end of buffer)
            }
        } else {
            tracing::debug!("🗑️  No deletion performed - invalid line");
            vec![]
        }
    }

    /// Delete character after cursor without joining lines (safe for Visual Block Insert)
    pub fn delete_char_after_cursor_no_join(
        &mut self,
        content_width: usize,
        wrap_enabled: bool,
        tab_width: usize,
    ) -> Vec<ViewEvent> {
        // Check if editing is allowed on this pane
        if !self.capabilities.contains(PaneCapabilities::EDITABLE) {
            return vec![]; // Editing not allowed on this pane
        }

        let current_cursor = self.buffer.cursor();

        tracing::debug!(
            "🗑️  PaneState::delete_char_after_cursor_no_join at position {:?}",
            current_cursor
        );

        // Get current line to check if we can delete within the line
        if let Some(current_line) = self.buffer.content().get_line(current_cursor.line) {
            if current_cursor.column < current_line.len() {
                // Delete character at cursor position (same line only)
                self.delete_char_after_cursor_in_line(
                    current_cursor,
                    content_width,
                    wrap_enabled,
                    tab_width,
                )
            } else {
                // At end of line - do NOT join with next line in Visual Block Insert mode
                tracing::debug!(
                    "🗑️  No deletion performed - at end of line (Visual Block Insert mode)"
                );
                vec![] // No line joining allowed
            }
        } else {
            tracing::debug!("🗑️  No deletion performed - invalid line");
            vec![]
        }
    }

    /// Delete the currently selected text and return the deleted content
    pub fn delete_selected_text(&mut self) -> DeletionResult {
        // Extract selection boundaries
        let (start, end) = (self.visual_selection_start?, self.visual_selection_end?);

        match self.editor_mode {
            EditorMode::VisualBlock => self.delete_visual_block_selection(),
            EditorMode::VisualLine => self.delete_visual_line_selection(),
            _ => {
                // Get the selected text before deleting it
                let selected_text = self.get_selected_text().unwrap_or_default();

                // Normalize selection (ensure start <= end)
                let (selection_start, selection_end) = if start.line < end.line
                    || (start.line == end.line && start.column <= end.column)
                {
                    (start, end)
                } else {
                    (end, start)
                };

                // Create deletion range - adjust end position for character-wise selection
                // to make it inclusive (matching how get_selected_text works)
                let content = self.buffer.content();
                let adjusted_end = if selection_start.line == selection_end.line {
                    // Single line: add 1 to end column to include the character at end position
                    // but don't go beyond the line length
                    let max_col = content.line_length(selection_end.line);
                    LogicalPosition::new(
                        selection_end.line,
                        (selection_end.column + 1).min(max_col),
                    )
                } else {
                    // Multi-line: add 1 to end column to include the character at end position
                    // but don't go beyond the line length
                    let max_col = content.line_length(selection_end.line);
                    LogicalPosition::new(
                        selection_end.line,
                        (selection_end.column + 1).min(max_col),
                    )
                };
                let delete_range = LogicalRange::new(selection_start, adjusted_end);

                tracing::debug!(
                    "🗑️  delete_selected_text: deleting range {:?} to {:?} (adjusted from {:?})",
                    selection_start,
                    adjusted_end,
                    selection_end
                );

                // Perform deletion
                let pane_type = self.buffer.pane();
                if let Some(event) = self
                    .buffer
                    .content_mut()
                    .delete_range(pane_type, delete_range)
                {
                    // Position cursor at start of deleted range
                    self.buffer.set_cursor(selection_start);

                    // Save and clear visual selection
                    self.save_last_visual_selection_before_clear();
                    self.visual_selection_start = None;
                    self.visual_selection_end = None;

                    Some((selected_text, event))
                } else {
                    None
                }
            }
        }
    }

    /// Delete character at cursor position and return the deleted character
    pub fn delete_char_at_cursor_with_return(
        &mut self,
        content_width: usize,
        wrap_enabled: bool,
        tab_width: usize,
    ) -> Option<String> {
        // Check if editing is allowed on this pane
        if !self.capabilities.contains(PaneCapabilities::EDITABLE) {
            return None; // Editing not allowed on this pane
        }

        let current_cursor = self.buffer.cursor();

        tracing::debug!(
            "✂️  PaneState::delete_char_at_cursor_with_return at position {:?}",
            current_cursor
        );

        // Get the current line
        let Some(current_line) = self.buffer.content().get_line(current_cursor.line) else {
            tracing::debug!("✂️  Invalid line for delete operation");
            return None;
        };

        // Check if cursor is within the line
        if current_cursor.column >= current_line.len() {
            tracing::debug!("✂️  Cursor at end of line, no character to delete");
            return None;
        }

        // Get the character at cursor position
        let Some(char_at_cursor) = current_line.chars().nth(current_cursor.column) else {
            tracing::debug!("✂️  No character at cursor position to delete");
            return None;
        };

        tracing::debug!("✂️  Will delete character '{}' at cursor", char_at_cursor);

        // Delete the character using delete_range
        let delete_start = current_cursor;
        let delete_end = LogicalPosition::new(current_cursor.line, current_cursor.column + 1);
        let delete_range = LogicalRange::new(delete_start, delete_end);

        let pane_type = self.buffer.pane();
        let Some(_event) = self
            .buffer
            .content_mut()
            .delete_range(pane_type, delete_range)
        else {
            tracing::warn!("✂️  Failed to delete character at cursor");
            return None;
        };

        // After deletion, check if we need to adjust cursor position
        if let Some(line) = self.buffer.content().get_line(current_cursor.line) {
            // Calculate character count (not byte count) for proper multi-byte character handling
            let char_count = line.chars().count();

            // If cursor is now beyond the end of the line, move it to the last valid position
            if current_cursor.column >= char_count {
                let new_cursor_column = if line.is_empty() {
                    // Line is empty, cursor goes to column 0
                    0
                } else {
                    // Cursor goes to the last character position (character count - 1)
                    char_count.saturating_sub(1)
                };
                let new_cursor = LogicalPosition::new(current_cursor.line, new_cursor_column);
                self.buffer.set_cursor(new_cursor);
                tracing::debug!(
                    "✂️  Adjusted cursor from {:?} to {:?} (char count: {}, byte length: {})",
                    current_cursor,
                    new_cursor,
                    char_count,
                    line.len()
                );
            }
        }

        // Rebuild display cache to ensure proper rendering
        self.build_display_cache(content_width, wrap_enabled, tab_width);

        // Sync display cursor with the logical cursor position after cache rebuild
        let logical_cursor = self.buffer.cursor();
        if let Some(display_pos) = self
            .display_cache
            .logical_to_display_position(logical_cursor.line, logical_cursor.column)
        {
            self.display_cursor = display_pos;
            tracing::debug!(
                "✂️  Synced display cursor to {:?} (logical: {:?})",
                display_pos,
                logical_cursor
            );
        } else {
            // Fallback: Use logical position as display position
            self.display_cursor = Position::new(logical_cursor.line, logical_cursor.column);
            tracing::warn!(
                "✂️  Failed to sync display cursor, using fallback for logical: {:?}",
                logical_cursor
            );
        }

        tracing::debug!("✂️  Successfully deleted character at cursor");

        Some(char_at_cursor.to_string())
    }

    /// Cut from cursor position to end of line and return the deleted text
    pub fn cut_to_end_of_line_with_return(
        &mut self,
        content_width: usize,
        wrap_enabled: bool,
        tab_width: usize,
    ) -> Option<String> {
        // Check if editing is allowed on this pane
        if !self.capabilities.contains(PaneCapabilities::EDITABLE) {
            return None; // Editing not allowed on this pane
        }

        let current_cursor = self.buffer.cursor();

        tracing::debug!(
            "✂️  PaneState::cut_to_end_of_line_with_return at position {:?}",
            current_cursor
        );

        // Get the current line
        let Some(current_line) = self.buffer.content().get_line(current_cursor.line) else {
            tracing::debug!("✂️  Invalid line for cut to end of line operation");
            return None;
        };

        // Convert line to characters for proper multi-byte handling
        let chars: Vec<char> = current_line.chars().collect();
        let line_char_length = chars.len();

        // Check if cursor is at or beyond the end of line
        if current_cursor.column >= line_char_length {
            tracing::debug!("✂️  Cursor at or beyond end of line, nothing to cut");
            return None;
        }

        // Get text from cursor to end of line
        let cut_chars: String = chars[current_cursor.column..].iter().collect();

        tracing::debug!(
            "✂️  Will cut text '{}' from cursor to end of line",
            cut_chars
        );

        // Delete the text using delete_range
        let delete_start = current_cursor;
        let delete_end = LogicalPosition::new(current_cursor.line, line_char_length);
        let delete_range = LogicalRange::new(delete_start, delete_end);

        let pane_type = self.buffer.pane();
        let Some(_event) = self
            .buffer
            .content_mut()
            .delete_range(pane_type, delete_range)
        else {
            tracing::warn!("✂️  Failed to cut text to end of line");
            return None;
        };

        // Cursor stays at current position (no movement after cutting to end of line)
        tracing::debug!(
            "✂️  Cut text to end of line, cursor remains at: {:?}",
            current_cursor
        );

        // Rebuild display cache to ensure proper rendering
        self.build_display_cache(content_width, wrap_enabled, tab_width);

        // Sync display cursor with the logical cursor position after cache rebuild
        let logical_cursor = self.buffer.cursor();
        if let Some(display_pos) = self
            .display_cache
            .logical_to_display_position(logical_cursor.line, logical_cursor.column)
        {
            self.display_cursor = display_pos;
            tracing::debug!(
                "✂️  Synced display cursor to {:?} (logical: {:?})",
                display_pos,
                logical_cursor
            );
        } else {
            // Fallback: Use logical position as display position
            self.display_cursor = Position::new(logical_cursor.line, logical_cursor.column);
            tracing::warn!(
                "✂️  Failed to sync display cursor, using fallback for logical: {:?}",
                logical_cursor
            );
        }

        tracing::debug!("✂️  Successfully cut text to end of line");

        Some(cut_chars)
    }

    // ========================================
    // Private Helper Methods
    // ========================================

    /// Delete a character within the current line
    fn delete_char_in_line(
        &mut self,
        current_cursor: LogicalPosition,
        content_width: usize,
        wrap_enabled: bool,
        tab_width: usize,
    ) -> Vec<ViewEvent> {
        tracing::debug!("🗑️  Deleting character before cursor in same line");

        let delete_start = LogicalPosition::new(current_cursor.line, current_cursor.column - 1);
        let delete_end = LogicalPosition::new(current_cursor.line, current_cursor.column);
        let delete_range = LogicalRange::new(delete_start, delete_end);

        // Attempt deletion
        let pane_type = self.buffer.pane();
        let Some(_event) = self
            .buffer
            .content_mut()
            .delete_range(pane_type, delete_range)
        else {
            return vec![];
        };

        // Move cursor left after successful deletion
        let new_cursor = LogicalPosition::new(current_cursor.line, current_cursor.column - 1);
        self.buffer.set_cursor(new_cursor);

        tracing::debug!(
            "🗑️  Deleted character in line, new cursor: {:?}",
            new_cursor
        );

        // Rebuild display cache and sync cursor
        self.rebuild_display_and_sync_cursor(new_cursor, content_width, wrap_enabled, tab_width);

        vec![
            ViewEvent::RequestContentChanged,
            ViewEvent::ActiveCursorUpdateRequired,
            ViewEvent::CurrentAreaRedrawRequired,
        ]
    }

    /// Delete a character after cursor within the current line
    fn delete_char_after_cursor_in_line(
        &mut self,
        current_cursor: LogicalPosition,
        content_width: usize,
        wrap_enabled: bool,
        tab_width: usize,
    ) -> Vec<ViewEvent> {
        tracing::debug!("🗑️  Deleting character after cursor in same line");

        let delete_start = LogicalPosition::new(current_cursor.line, current_cursor.column);
        let delete_end = LogicalPosition::new(current_cursor.line, current_cursor.column + 1);
        let delete_range = LogicalRange::new(delete_start, delete_end);

        // Attempt deletion
        let pane_type = self.buffer.pane();
        let Some(_event) = self
            .buffer
            .content_mut()
            .delete_range(pane_type, delete_range)
        else {
            return vec![];
        };

        // Cursor stays at same position after forward deletion
        tracing::debug!(
            "🗑️  Deleted character after cursor, cursor remains at: {:?}",
            current_cursor
        );

        // Rebuild display cache and sync cursor
        self.rebuild_display_and_sync_cursor(
            current_cursor,
            content_width,
            wrap_enabled,
            tab_width,
        );

        vec![
            ViewEvent::RequestContentChanged,
            ViewEvent::ActiveCursorUpdateRequired,
            ViewEvent::CurrentAreaRedrawRequired,
        ]
    }

    /// Join current line with previous line (backspace at beginning of line)
    fn join_with_previous_line(
        &mut self,
        current_cursor: LogicalPosition,
        content_width: usize,
        wrap_enabled: bool,
        tab_width: usize,
    ) -> Vec<ViewEvent> {
        tracing::debug!("🗑️  Joining current line with previous line");

        // Get length of previous line to position cursor correctly
        let prev_line_length =
            if let Some(prev_line) = self.buffer.content().get_line(current_cursor.line - 1) {
                prev_line.len()
            } else {
                0
            };

        // Delete the newline character between previous and current line
        let delete_start = LogicalPosition::new(current_cursor.line - 1, prev_line_length);
        let delete_end = LogicalPosition::new(current_cursor.line, 0);
        let delete_range = LogicalRange::new(delete_start, delete_end);

        // Attempt deletion
        let pane_type = self.buffer.pane();
        let Some(_event) = self
            .buffer
            .content_mut()
            .delete_range(pane_type, delete_range)
        else {
            return vec![];
        };

        // Position cursor at end of previous line (where lines joined)
        let new_cursor = LogicalPosition::new(current_cursor.line - 1, prev_line_length);
        self.buffer.set_cursor(new_cursor);

        tracing::debug!("🗑️  Joined lines, new cursor: {:?}", new_cursor);

        // Rebuild display cache and sync cursor
        self.rebuild_display_and_sync_cursor(new_cursor, content_width, wrap_enabled, tab_width);

        vec![
            ViewEvent::RequestContentChanged,
            ViewEvent::ActiveCursorUpdateRequired,
            ViewEvent::CurrentAreaRedrawRequired,
        ]
    }

    /// Join current line with next line (delete at end of line)
    fn join_with_next_line(
        &mut self,
        current_cursor: LogicalPosition,
        content_width: usize,
        wrap_enabled: bool,
        tab_width: usize,
    ) -> Vec<ViewEvent> {
        tracing::debug!("🗑️  Joining current line with next line");

        // Delete the newline character between current and next line
        let delete_start = LogicalPosition::new(current_cursor.line, current_cursor.column);
        let delete_end = LogicalPosition::new(current_cursor.line + 1, 0);
        let delete_range = LogicalRange::new(delete_start, delete_end);

        // Attempt deletion
        let pane_type = self.buffer.pane();
        let Some(_event) = self
            .buffer
            .content_mut()
            .delete_range(pane_type, delete_range)
        else {
            return vec![];
        };

        // Cursor stays at same position
        tracing::debug!("🗑️  Joined lines, cursor remains at: {:?}", current_cursor);

        // Rebuild display cache and sync cursor
        self.rebuild_display_and_sync_cursor(
            current_cursor,
            content_width,
            wrap_enabled,
            tab_width,
        );

        vec![
            ViewEvent::RequestContentChanged,
            ViewEvent::ActiveCursorUpdateRequired,
            ViewEvent::CurrentAreaRedrawRequired,
        ]
    }

    /// Helper to rebuild display cache and sync cursor position
    fn rebuild_display_and_sync_cursor(
        &mut self,
        new_cursor: LogicalPosition,
        content_width: usize,
        wrap_enabled: bool,
        tab_width: usize,
    ) {
        // Rebuild display cache since content changed
        self.build_display_cache(content_width, wrap_enabled, tab_width);

        // Sync display cursor with new logical position after cache rebuild
        match self
            .display_cache
            .logical_to_display_position(new_cursor.line, new_cursor.column)
        {
            Some(display_pos) => {
                self.display_cursor = display_pos;
            }
            None => {
                // BUGFIX Issue #89: If logical_to_display_position fails, ensure cursor tracking doesn't break
                tracing::warn!(
                    "delete_char_before_cursor: logical_to_display_position failed at {:?} - using fallback", 
                    new_cursor
                );
                // Fallback: Use logical position as display position (works for non-wrapped content)
                self.display_cursor = Position::new(new_cursor.line, new_cursor.column);
            }
        }
    }

    /// Delete the selected text in Visual Block mode
    fn delete_visual_block_selection(&mut self) -> DeletionResult {
        let (start, end) = (self.visual_selection_start?, self.visual_selection_end?);

        // Extract the text before deleting it
        let selected_text = self.get_selected_text().unwrap_or_default();

        // For Visual Block mode, we need to delete from each line individually
        // Working backwards to maintain line number validity
        let top_line = start.line.min(end.line);
        let bottom_line = start.line.max(end.line);
        let left_col = start.column.min(end.column);
        let right_col = start.column.max(end.column);

        let pane_type = self.buffer.pane();
        let mut any_deletion = false;

        // Delete from bottom to top to maintain line indices
        for line_num in (top_line..=bottom_line).rev() {
            if let Some(line) = self.buffer.content().get_line(line_num) {
                let line_length = line.len();

                // Only delete if the line is long enough to have content in the block region
                if left_col < line_length {
                    let actual_right_col = (right_col + 1).min(line_length);
                    let delete_start = LogicalPosition::new(line_num, left_col);
                    let delete_end = LogicalPosition::new(line_num, actual_right_col);
                    let delete_range = LogicalRange::new(delete_start, delete_end);

                    if self
                        .buffer
                        .content_mut()
                        .delete_range(pane_type, delete_range)
                        .is_some()
                    {
                        any_deletion = true;
                    }
                }
            }
        }

        if any_deletion {
            // Position cursor at top-left of the deleted block
            let new_cursor = LogicalPosition::new(top_line, left_col);
            self.buffer.set_cursor(new_cursor);

            // Save and clear visual selection
            self.save_last_visual_selection_before_clear();
            self.visual_selection_start = None;
            self.visual_selection_end = None;

            // Create a synthetic event for the block deletion
            let event = ModelEvent::TextDeleted {
                pane: pane_type,
                range: LogicalRange::new(
                    LogicalPosition::new(top_line, left_col),
                    LogicalPosition::new(bottom_line, right_col + 1),
                ),
            };

            Some((selected_text, event))
        } else {
            None
        }
    }

    /// Delete the selected text in Visual Line mode (complete lines)
    fn delete_visual_line_selection(&mut self) -> DeletionResult {
        let (start, end) = (self.visual_selection_start?, self.visual_selection_end?);

        // Extract the text before deleting it
        let selected_text = self.get_selected_text().unwrap_or_default();

        // For Visual Line mode, we delete complete lines from first to last
        let first_line = start.line.min(end.line);
        let last_line = start.line.max(end.line);

        // Visual Line deletion: from beginning of first line to end of last line (including newline)
        let delete_start = LogicalPosition::new(first_line, 0);

        // For the end position, we need to include the newline after the last line
        // Check if there's content after the last line
        let content = self.buffer.content();
        let last_line_length = content
            .get_line(last_line)
            .map(|line| line.len())
            .unwrap_or(0);

        let delete_end = if last_line + 1 < content.line_count() {
            // There are lines after the last selected line, so delete up to start of next line
            LogicalPosition::new(last_line + 1, 0)
        } else {
            // This is the last line in the buffer, delete to end of line
            LogicalPosition::new(last_line, last_line_length)
        };

        // Create deletion range
        let delete_range = LogicalRange::new(delete_start, delete_end);

        // Perform deletion
        let pane_type = self.buffer.pane();
        if let Some(event) = self
            .buffer
            .content_mut()
            .delete_range(pane_type, delete_range)
        {
            // Position cursor at start of deleted range (beginning of first line)
            self.buffer.set_cursor(delete_start);

            // Save and clear visual selection
            self.save_last_visual_selection_before_clear();
            self.visual_selection_start = None;
            self.visual_selection_end = None;

            Some((selected_text, event))
        } else {
            None
        }
    }

    /// Insert text block-wise at specific positions (for block paste operations)
    /// This inserts each line at the same column on successive lines without affecting cursor
    pub fn insert_block_wise(
        &mut self,
        start_position: LogicalPosition,
        block_lines: &[&str],
    ) -> Vec<ViewEvent> {
        if block_lines.is_empty() {
            return vec![];
        }

        let pane_type = self.buffer.pane();

        // Store original cursor position
        let original_cursor = self.buffer.cursor();

        // Process each line in the block
        for (line_offset, line_content) in block_lines.iter().enumerate() {
            let target_position = LogicalPosition {
                line: start_position.line + line_offset,
                column: start_position.column,
            };

            // Ensure target line exists
            while target_position.line >= self.buffer.content().line_count() {
                // Add a new line at the end of the buffer
                let current_line_count = self.buffer.content().line_count();
                let last_line = current_line_count.saturating_sub(1);
                let last_line_length = self.buffer.content().line_length(last_line);
                let end_of_buffer = LogicalPosition {
                    line: last_line,
                    column: last_line_length,
                };
                self.buffer
                    .content_mut()
                    .insert_text(pane_type, end_of_buffer, "\n");
            }

            // Ensure the target line is long enough by padding with spaces
            let current_line_length = self.buffer.content().line_length(target_position.line);
            if target_position.column > current_line_length {
                let padding_needed = target_position.column - current_line_length;
                let padding = " ".repeat(padding_needed);
                let line_end = LogicalPosition {
                    line: target_position.line,
                    column: current_line_length,
                };
                self.buffer
                    .content_mut()
                    .insert_text(pane_type, line_end, &padding);
            }

            // Insert the line content at the target position
            self.buffer
                .content_mut()
                .insert_text(pane_type, target_position, line_content);
        }

        // Restore original cursor position (block paste shouldn't move cursor)
        self.buffer.set_cursor(original_cursor);

        // Get display parameters from the current display cache
        let content_width = self.display_cache.content_width;
        let wrap_enabled = self.display_cache.wrap_enabled;
        // Use a default tab width (should be passed in or stored somewhere)
        let tab_width = 4; // Default tab width

        // Rebuild the display cache since we've modified the buffer significantly
        self.build_display_cache(content_width, wrap_enabled, tab_width);

        // Sync the display cursor with the logical cursor position
        self.sync_display_cursor_with_logical();

        // Return view events for full redraw to ensure the block paste is visible
        vec![
            ViewEvent::CurrentAreaRedrawRequired,
            ViewEvent::ActiveCursorUpdateRequired,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::events::{EditorMode, LogicalPosition, Pane, PaneCapabilities};
    use crate::repl::geometry::{Dimensions, Position};
    use crate::repl::models::{BufferModel, DisplayCache};

    fn create_test_pane_state_with_content(content: &str) -> PaneState {
        let mut buffer = BufferModel::new(Pane::Request);
        buffer.content_mut().set_text(content);

        let display_cache = DisplayCache::new();

        PaneState {
            buffer,
            display_cache,
            display_cursor: Position::new(0, 0),
            scroll_offset: Position::new(0, 0),
            visual_selection_start: None,
            visual_selection_end: None,
            last_visual_selection_start: None,
            last_visual_selection_end: None,
            last_visual_mode: None,
            pane_dimensions: Dimensions::new(80, 25),
            editor_mode: EditorMode::Visual,
            capabilities: PaneCapabilities::EDITABLE
                | PaneCapabilities::NAVIGABLE
                | PaneCapabilities::SELECTABLE,
            line_number_width: 3,
            virtual_column: 0,
        }
    }

    #[test]
    fn test_get_selected_text_single_line_multibyte() {
        let mut pane_state = create_test_pane_state_with_content("あいうえおかきくけこ");

        // Select "かきくけこ" (characters 5-9)
        pane_state.visual_selection_start = Some(LogicalPosition::new(0, 5));
        pane_state.visual_selection_end = Some(LogicalPosition::new(0, 9));

        let result = pane_state.get_selected_text();
        assert_eq!(result, Some("かきくけこ".to_string()));
    }

    #[test]
    fn test_get_selected_text_single_line_mixed_chars() {
        let mut pane_state = create_test_pane_state_with_content("abc漢字def");

        // Select "漢字" (characters 3-4)
        pane_state.visual_selection_start = Some(LogicalPosition::new(0, 3));
        pane_state.visual_selection_end = Some(LogicalPosition::new(0, 4));

        let result = pane_state.get_selected_text();
        assert_eq!(result, Some("漢字".to_string()));
    }

    #[test]
    fn test_get_selected_text_visual_block_multibyte() {
        let mut pane_state =
            create_test_pane_state_with_content("あいうえお\nかきくけこ\nさしすせそ");
        pane_state.editor_mode = EditorMode::VisualBlock;

        // Select a 2x2 block starting at column 1: "いう" + "きく"
        pane_state.visual_selection_start = Some(LogicalPosition::new(0, 1));
        pane_state.visual_selection_end = Some(LogicalPosition::new(1, 2));

        let result = pane_state.get_selected_text();
        assert_eq!(result, Some("いう\nきく".to_string()));
    }

    #[test]
    fn test_get_selected_text_multiline_multibyte() {
        let mut pane_state = create_test_pane_state_with_content("あいうえお\nかきくけこ");

        // Select from middle of first line to middle of second line
        pane_state.visual_selection_start = Some(LogicalPosition::new(0, 2)); // "う"
        pane_state.visual_selection_end = Some(LogicalPosition::new(1, 2)); // "く"

        let result = pane_state.get_selected_text();
        assert_eq!(result, Some("うえお\nかきく".to_string()));
    }

    #[test]
    fn test_get_selected_text_edge_case_end_of_multibyte_line() {
        let mut pane_state = create_test_pane_state_with_content("あいうえお");

        // Select last character "お" (character 4)
        pane_state.visual_selection_start = Some(LogicalPosition::new(0, 4));
        pane_state.visual_selection_end = Some(LogicalPosition::new(0, 4));

        let result = pane_state.get_selected_text();
        assert_eq!(result, Some("お".to_string()));
    }

    #[test]
    fn test_get_selected_text_beyond_line_boundary() {
        let mut pane_state = create_test_pane_state_with_content("あい");

        // Try to select beyond the line (characters 0-5, but line only has 2 chars)
        pane_state.visual_selection_start = Some(LogicalPosition::new(0, 0));
        pane_state.visual_selection_end = Some(LogicalPosition::new(0, 5));

        let result = pane_state.get_selected_text();
        assert_eq!(result, Some("あい".to_string())); // Should clamp to line length
    }
}
