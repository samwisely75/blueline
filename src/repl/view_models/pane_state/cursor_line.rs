//! Line-based cursor movement operations for PaneState
//!
//! This module contains methods for:
//! - Moving cursor to start/end of lines
//! - Document-wide navigation (start/end of document)
//! - Line number-based navigation
//! - Append mode positioning

use crate::repl::events::{EditorMode, LogicalPosition, PaneCapabilities, ViewEvent};
use crate::repl::models::geometry::Position;

use super::PaneState;

impl PaneState {
    /// Move cursor to start of current line with capability checking
    pub fn move_cursor_to_start_of_line(&mut self, content_width: usize) -> Vec<ViewEvent> {
        // Check if navigation is allowed on this pane
        if !self.capabilities.contains(PaneCapabilities::NAVIGABLE) {
            return vec![]; // Navigation not allowed on this pane
        }

        // Get current logical position
        let current_logical = self.buffer.cursor();

        // Create new logical position at start of current line (column 0)
        let new_logical = LogicalPosition::new(current_logical.line, 0);

        // Update logical cursor first
        self.buffer.set_cursor(new_logical);

        // Sync display cursor with logical cursor
        self.sync_display_cursor_with_logical();

        // Update visual selection if active
        self.update_visual_selection_on_cursor_move(new_logical);

        let mut events = vec![
            ViewEvent::ActiveCursorUpdateRequired,
            ViewEvent::PositionIndicatorUpdateRequired,
        ];

        // Add redraw event for visual selection if active
        if self.visual_selection_start.is_some() {
            events.push(ViewEvent::CurrentAreaRedrawRequired);
        }

        // Ensure cursor is visible and add visibility events
        let visibility_events = self.ensure_cursor_visible_with_events(content_width);
        events.extend(visibility_events);

        events
    }

    /// Move cursor to end of current line for append (A command) with capability checking
    /// This positions the cursor AFTER the last character for insert mode
    pub fn move_cursor_to_line_end_for_append(&mut self, content_width: usize) -> Vec<ViewEvent> {
        // Check if navigation is allowed on this pane
        if !self.capabilities.contains(PaneCapabilities::NAVIGABLE) {
            return vec![]; // Navigation not allowed on this pane
        }

        // Get current logical position
        let current_logical = self.buffer.cursor();

        let mut events = vec![
            ViewEvent::ActiveCursorUpdateRequired,
            ViewEvent::PositionIndicatorUpdateRequired,
        ];

        // Get the current line content to find its length
        if let Some(line) = self.buffer.content().get_line(current_logical.line) {
            let line_length = line.chars().count();

            // For the 'A' command, position cursor AFTER the last character
            // This allows inserting at the end of the line
            let end_position = line_length; // Position after last character
            let new_logical = LogicalPosition::new(current_logical.line, end_position);

            // Update logical cursor first
            self.buffer.set_cursor(new_logical);

            // Sync display cursor with logical cursor
            self.sync_display_cursor_with_logical();

            // Update visual selection if active
            self.update_visual_selection_on_cursor_move(new_logical);

            // Add redraw event for visual selection if active
            if self.visual_selection_start.is_some() {
                events.push(ViewEvent::CurrentAreaRedrawRequired);
            }
        }

        // Ensure cursor is visible with Insert-mode scrolling logic
        // The A command will immediately switch to Insert mode, so we need to use
        // Insert mode scrolling behavior here to ensure proper horizontal scrolling
        let original_mode = self.editor_mode;

        // Temporarily set to Insert mode for proper scrolling calculation
        self.editor_mode = EditorMode::Insert;
        let visibility_events = self.ensure_cursor_visible_with_events(content_width);

        // Restore original mode
        self.editor_mode = original_mode;

        events.extend(visibility_events);

        events
    }

    /// Move cursor to end of current line with capability checking
    pub fn move_cursor_to_end_of_line(&mut self, content_width: usize) -> Vec<ViewEvent> {
        // Check if navigation is allowed on this pane
        if !self.capabilities.contains(PaneCapabilities::NAVIGABLE) {
            return vec![]; // Navigation not allowed on this pane
        }

        // Get current logical position
        let current_logical = self.buffer.cursor();

        let mut events = vec![
            ViewEvent::ActiveCursorUpdateRequired,
            ViewEvent::PositionIndicatorUpdateRequired,
        ];

        // Get the current line content to find its end position
        if let Some(line) = self.buffer.content().get_line(current_logical.line) {
            let line_length = line.chars().count();

            // Position cursor at the last character (not after it) for Normal/Visual mode
            let end_position = if line_length > 0 {
                // Stay ON the last character for Normal/Visual mode (vim behavior)
                match self.editor_mode {
                    EditorMode::Insert => line_length, // Insert mode: can be after last character
                    _ => line_length.saturating_sub(1), // Normal/Visual: ON the last character
                }
            } else {
                0 // Empty line, stay at column 0
            };

            let new_logical = LogicalPosition::new(current_logical.line, end_position);

            // Update logical cursor first
            self.buffer.set_cursor(new_logical);

            // Sync display cursor with logical cursor
            self.sync_display_cursor_with_logical();

            // Update visual selection if active
            self.update_visual_selection_on_cursor_move(new_logical);

            // Add redraw event for visual selection if active
            if self.visual_selection_start.is_some() {
                events.push(ViewEvent::CurrentAreaRedrawRequired);
            }
        }

        // Ensure cursor is visible and add visibility events
        let visibility_events = self.ensure_cursor_visible_with_events(content_width);
        events.extend(visibility_events);

        events
    }

    /// Move cursor to start of document with capability checking
    pub fn move_cursor_to_document_start(&mut self, content_width: usize) -> Vec<ViewEvent> {
        // Check if navigation is allowed on this pane
        if !self.capabilities.contains(PaneCapabilities::NAVIGABLE) {
            return vec![]; // Navigation not allowed on this pane
        }

        // Use proper cursor positioning method to ensure logical/display sync
        let start_position = Position::origin();
        let _result = self.set_display_cursor(start_position);

        // Reset virtual column to 0 (vim gg behavior)
        self.virtual_column = 0;

        let mut events = vec![
            ViewEvent::ActiveCursorUpdateRequired,
            ViewEvent::PositionIndicatorUpdateRequired,
        ];

        // Update visual selection if active
        let new_cursor_pos = self.buffer.cursor();
        self.update_visual_selection_on_cursor_move(new_cursor_pos);

        // Add redraw event for visual selection if active
        if self.visual_selection_start.is_some() {
            events.push(ViewEvent::CurrentAreaRedrawRequired);
        }

        // Ensure cursor is visible and add visibility events
        let visibility_events = self.ensure_cursor_visible_with_events(content_width);
        events.extend(visibility_events);

        events
    }

    /// Move cursor to end of document with capability checking
    pub fn move_cursor_to_document_end(&mut self, content_width: usize) -> Vec<ViewEvent> {
        // Check if navigation is allowed on this pane
        if !self.capabilities.contains(PaneCapabilities::NAVIGABLE) {
            return vec![]; // Navigation not allowed on this pane
        }

        // Find the last valid display line by iterating
        let mut last_line_idx = 0;
        let mut idx = 0;
        while self.display_cache.get_display_line(idx).is_some() {
            last_line_idx = idx;
            idx += 1;
        }

        let mut events = vec![
            ViewEvent::ActiveCursorUpdateRequired,
            ViewEvent::PositionIndicatorUpdateRequired,
        ];

        // Move to beginning of the last line (vim G behavior)
        let end_position = Position::new(last_line_idx, 0);
        let _result = self.set_display_cursor(end_position);

        // Reset virtual column to 0 (vim G behavior)
        self.virtual_column = 0;

        // Update visual selection if active
        let new_cursor_pos = self.buffer.cursor();
        self.update_visual_selection_on_cursor_move(new_cursor_pos);

        // Add redraw event for visual selection if active
        if self.visual_selection_start.is_some() {
            events.push(ViewEvent::CurrentAreaRedrawRequired);
        }

        // Ensure cursor is visible and add visibility events
        let visibility_events = self.ensure_cursor_visible_with_events(content_width);
        events.extend(visibility_events);

        events
    }

    /// Move cursor to specific line number (1-based) with capability checking
    /// If line_number is out of bounds, clamps to the last available line (vim behavior)
    pub fn move_cursor_to_line(
        &mut self,
        line_number: usize,
        content_width: usize,
    ) -> Vec<ViewEvent> {
        // Check if navigation is allowed on this pane
        if !self.capabilities.contains(PaneCapabilities::NAVIGABLE) {
            return vec![]; // Navigation not allowed on this pane
        }

        if line_number == 0 {
            return vec![];
        }

        let max_line_count = self.display_cache.display_line_count();

        if max_line_count == 0 {
            return vec![]; // No lines to navigate to
        }

        // Clamp to valid range (1 to max_line_count)
        let clamped_line_number = line_number.min(max_line_count);
        let target_line_idx = clamped_line_number - 1; // Convert to 0-based

        // Set cursor position
        self.display_cursor = Position::new(target_line_idx, 0);

        // Sync logical cursor with display cursor
        if let Some(logical_pos) = self
            .display_cache
            .display_to_logical_position(target_line_idx, 0)
        {
            let new_logical_pos = LogicalPosition::new(logical_pos.row, logical_pos.col);
            self.buffer.set_cursor(new_logical_pos);

            // Update visual selection if active
            self.update_visual_selection_on_cursor_move(new_logical_pos);
        }

        let mut events = vec![
            ViewEvent::ActiveCursorUpdateRequired,
            ViewEvent::PositionIndicatorUpdateRequired,
        ];

        // Add redraw event for visual selection if active
        if self.visual_selection_start.is_some() {
            events.push(ViewEvent::CurrentAreaRedrawRequired);
        }

        // Ensure cursor is visible and add visibility events
        let visibility_events = self.ensure_cursor_visible_with_events(content_width);
        events.extend(visibility_events);

        events
    }
}
