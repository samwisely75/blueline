//! Scrolling operations and cursor visibility management for PaneState
//!
//! This module contains methods for:
//! - Horizontal and vertical scrolling
//! - Ensuring cursor visibility within viewport
//! - Cursor position synchronization
//! - Mode-aware scrolling behavior

use crate::repl::events::{EditorMode, LogicalPosition, PaneCapabilities, ViewEvent};
use crate::repl::models::geometry::Position;

use super::{CursorMoveResult, PaneState, ScrollAdjustResult, ScrollResult};

impl PaneState {
    /// Handle horizontal scrolling within this pane
    pub fn scroll_horizontally(&mut self, direction: i32, amount: usize) -> ScrollResult {
        let old_offset = self.scroll_offset.col; // horizontal offset
        let new_offset = if direction > 0 {
            old_offset + amount
        } else {
            old_offset.saturating_sub(amount)
        };

        self.scroll_offset.col = new_offset;

        // Handle cursor repositioning to stay visible after horizontal scroll
        let current_cursor = self.buffer.cursor();
        let mut cursor_moved = false;

        // Convert current logical position to display coordinates
        if let Some(display_pos) = self
            .display_cache
            .logical_to_display_position(current_cursor.line, current_cursor.column)
        {
            // Check if cursor is still visible after horizontal scroll
            let content_width = self.get_content_width();

            // If cursor is off-screen, move it to the first/last visible column
            let new_cursor_column = if display_pos.col < new_offset {
                // Cursor is off-screen to the left, move to first visible column
                new_offset
            } else if display_pos.col >= new_offset + content_width {
                // Cursor is off-screen to the right, move to last visible column
                new_offset + content_width - 1
            } else {
                // Cursor is still visible, keep current position
                display_pos.col
            };

            // Convert back to logical position and update cursor if needed
            if let Some(logical_pos) = self
                .display_cache
                .display_to_logical_position(display_pos.row, new_cursor_column)
            {
                let new_cursor_position = LogicalPosition::new(logical_pos.row, logical_pos.col);
                let clamped_position = self.buffer.content().clamp_position(new_cursor_position);

                if clamped_position != current_cursor {
                    self.buffer.set_cursor(clamped_position);
                    cursor_moved = true;
                }
            }
        }

        ScrollResult {
            old_offset,
            new_offset,
            cursor_moved,
        }
    }

    /// Set display cursor position for this pane with proper clamping
    pub fn set_display_cursor(&mut self, position: Position) -> CursorMoveResult {
        let old_display_pos = self.display_cursor;

        tracing::debug!(
            "PaneState::set_display_cursor: requested_pos={:?}",
            position
        );

        // Convert to logical position first (this will clamp if needed)
        if let Some(logical_pos) = self
            .display_cache
            .display_to_logical_position(position.row, position.col)
        {
            let logical_position = LogicalPosition::new(logical_pos.row, logical_pos.col);
            tracing::debug!(
                "PaneState::set_display_cursor: converted display ({}, {}) to logical ({}, {})",
                position.row,
                position.col,
                logical_position.line,
                logical_position.column
            );

            // Update logical cursor
            self.buffer.set_cursor(logical_position);

            // Set display cursor to the actual position that corresponds to the clamped logical position
            if let Some(actual_display_pos) = self
                .display_cache
                .logical_to_display_position(logical_position.line, logical_position.column)
            {
                self.display_cursor = actual_display_pos;
                tracing::debug!(
                    "PaneState::set_display_cursor: updated display cursor to actual position {:?}",
                    actual_display_pos
                );
            } else {
                self.display_cursor = position;
            }
        } else {
            tracing::warn!(
                "PaneState::set_display_cursor: failed to convert display position {:?} to logical",
                position
            );
            self.display_cursor = position;
        }

        let cursor_moved = self.display_cursor != old_display_pos;

        CursorMoveResult {
            cursor_moved,
            old_display_pos,
            new_display_pos: self.display_cursor,
        }
    }

    /// Synchronize display cursor with logical cursor position
    pub fn sync_display_cursor_with_logical(&mut self) -> CursorMoveResult {
        let old_display_pos = self.display_cursor;
        let logical_pos = self.buffer.cursor();

        if let Some(display_pos) = self
            .display_cache
            .logical_to_display_position(logical_pos.line, logical_pos.column)
        {
            tracing::debug!("PaneState::sync_display_cursor_with_logical: converted logical ({}, {}) to display ({}, {})", 
                logical_pos.line, logical_pos.column, display_pos.row, display_pos.col);
            self.display_cursor = display_pos;
        } else {
            tracing::warn!("PaneState::sync_display_cursor_with_logical: failed to convert logical ({}, {}) to display", 
                logical_pos.line, logical_pos.column);
        }

        let cursor_moved = self.display_cursor != old_display_pos;

        CursorMoveResult {
            cursor_moved,
            old_display_pos,
            new_display_pos: self.display_cursor,
        }
    }

    /// Ensure cursor is visible within the viewport, adjusting scroll offsets if needed
    pub fn ensure_cursor_visible(&mut self, content_width: usize) -> ScrollAdjustResult {
        let display_pos = self.display_cursor;
        let old_vertical_offset = self.scroll_offset.row;
        let old_horizontal_offset = self.scroll_offset.col;
        let pane_height = self.pane_dimensions.height;

        tracing::debug!("PaneState::ensure_cursor_visible: display_pos=({}, {}), scroll_offset=({}, {}), pane_size=({}, {})",
            display_pos.row, display_pos.col, old_vertical_offset, old_horizontal_offset, content_width, pane_height);

        let mut new_vertical_offset = old_vertical_offset;
        let mut new_horizontal_offset = old_horizontal_offset;

        // Vertical scrolling to keep cursor within visible area
        if display_pos.row < old_vertical_offset {
            new_vertical_offset = display_pos.row;
        } else if display_pos.row >= old_vertical_offset + pane_height && pane_height > 0 {
            new_vertical_offset = display_pos
                .row
                .saturating_sub(pane_height.saturating_sub(1));
        }

        // Horizontal scrolling
        // WRAP MODE FIX: When wrap is enabled, disable horizontal scrolling completely
        // Content should flow to multiple display lines instead of requiring horizontal scrolling
        if self.display_cache.wrap_enabled {
            // Reset horizontal offset to 0 when wrap mode is enabled
            new_horizontal_offset = 0;
            tracing::debug!("PaneState::ensure_cursor_visible: wrap mode enabled, resetting horizontal offset to 0");
        } else {
            // NOWRAP MODE: Normal horizontal scrolling behavior
            // The visible range is from old_horizontal_offset to (old_horizontal_offset + content_width - 1)
            // For example, if offset=0 and width=112, visible columns are 0-111
            if display_pos.col < old_horizontal_offset {
                new_horizontal_offset = display_pos.col;
                tracing::debug!("PaneState::ensure_cursor_visible: cursor off-screen left, adjusting horizontal offset to {}", new_horizontal_offset);
            } else if content_width > 0 {
                // MODE-AWARE HORIZONTAL SCROLL: Different trigger points for Insert vs Normal mode
                // Also check if the character at cursor position extends beyond the visible area
                let mut should_scroll_horizontally = match self.editor_mode {
                    crate::repl::events::EditorMode::Insert => {
                        // Insert mode: Scroll early to make room for typing next character
                        display_pos.col >= old_horizontal_offset + content_width
                    }
                    _ => {
                        // Normal/Visual mode: Only scroll when absolutely necessary
                        display_pos.col > old_horizontal_offset + content_width
                    }
                };

                // DOUBLE-BYTE CHARACTER FIX: Check if the character at cursor position
                // extends beyond the visible area (for double-byte characters)
                // This handles the case where cursor is at the edge and the character is wider than 1 column
                if !should_scroll_horizontally {
                    should_scroll_horizontally = self.check_character_extends_beyond_visible_area(
                        display_pos,
                        old_horizontal_offset,
                        content_width,
                    );
                }

                if should_scroll_horizontally {
                    // DISPLAY-COLUMN-AWARE HORIZONTAL SCROLL: Calculate the actual display columns needed
                    // to make the cursor visible, accounting for DBCS character widths

                    // CHARACTER-WIDTH-AWARE HORIZONTAL SCROLL: When scrolling, we need to scroll past
                    // complete characters, accounting for their actual display widths
                    let min_scroll_needed = display_pos
                        .col
                        .saturating_sub(content_width.saturating_sub(1));

                    // CHARACTER-WIDTH-BASED SCROLL: Calculate total width of characters to scroll past
                    new_horizontal_offset = self.calculate_horizontal_scroll_offset(
                        display_pos.row,
                        old_horizontal_offset,
                        min_scroll_needed,
                    );

                    let scroll_amount = new_horizontal_offset - old_horizontal_offset;
                    tracing::debug!("PaneState::ensure_cursor_visible: cursor off-screen at pos {}, need to scroll {}, scrolling {} from {} to {}", 
                        display_pos.col, min_scroll_needed, scroll_amount, old_horizontal_offset, new_horizontal_offset);
                }
            }
        }

        // Update scroll offset if changed
        let vertical_changed = new_vertical_offset != old_vertical_offset;
        let horizontal_changed = new_horizontal_offset != old_horizontal_offset;

        if vertical_changed || horizontal_changed {
            tracing::debug!(
                "PaneState::ensure_cursor_visible: adjusting scroll from ({}, {}) to ({}, {})",
                old_vertical_offset,
                old_horizontal_offset,
                new_vertical_offset,
                new_horizontal_offset
            );
            self.scroll_offset = Position::new(new_vertical_offset, new_horizontal_offset);
        } else {
            tracing::debug!("PaneState::ensure_cursor_visible: no scroll adjustment needed");
        }

        ScrollAdjustResult {
            vertical_changed,
            horizontal_changed,
            old_vertical_offset,
            new_vertical_offset,
            old_horizontal_offset,
            new_horizontal_offset,
        }
    }

    /// Ensure cursor is visible and return view events (wrapper around ensure_cursor_visible)
    pub fn ensure_cursor_visible_with_events(&mut self, content_width: usize) -> Vec<ViewEvent> {
        let result = self.ensure_cursor_visible(content_width);

        if result.vertical_changed || result.horizontal_changed {
            // For horizontal scrolling, use horizontal offsets; for vertical scrolling, use vertical offsets
            // If both changed, prioritize horizontal since it's more common in response navigation
            let (old_offset, new_offset) = if result.horizontal_changed {
                (result.old_horizontal_offset, result.new_horizontal_offset)
            } else {
                (result.old_vertical_offset, result.new_vertical_offset)
            };

            vec![ViewEvent::CurrentAreaScrollChanged {
                old_offset,
                new_offset,
            }]
        } else {
            vec![]
        }
    }

    // ========================================
    // Private Helper Methods
    // ========================================

    /// Check if the character at cursor position extends beyond the visible area
    fn check_character_extends_beyond_visible_area(
        &self,
        display_pos: Position,
        horizontal_offset: usize,
        content_width: usize,
    ) -> bool {
        let Some(display_line) = self.display_cache.get_display_line(display_pos.row) else {
            return false;
        };

        let Some(char_at_cursor) = display_line.char_at_display_col(display_pos.col) else {
            return false;
        };

        let char_width = char_at_cursor.display_width();
        // If the character extends beyond the visible area, trigger scrolling
        // This includes when cursor is exactly at the boundary but character is 2-wide
        display_pos.col + char_width > horizontal_offset + content_width
    }

    /// Calculate horizontal scroll offset accounting for character widths
    fn calculate_horizontal_scroll_offset(
        &self,
        display_row: usize,
        old_horizontal_offset: usize,
        min_scroll_needed: usize,
    ) -> usize {
        let Some(display_line) = self.display_cache.get_display_line(display_row) else {
            return old_horizontal_offset + min_scroll_needed;
        };

        let mut accumulated_width = 0;
        let mut check_col = old_horizontal_offset;

        // Keep scrolling past complete characters until we've scrolled enough
        while accumulated_width < min_scroll_needed {
            let Some(char_at_col) = display_line.char_at_display_col(check_col) else {
                // No more characters, use what we have
                break;
            };

            let char_width = char_at_col.display_width();
            accumulated_width += char_width;
            check_col += char_width;

            tracing::debug!("PaneState::calculate_horizontal_scroll: scrolling past char '{}' width={}, accumulated={}", 
                char_at_col.ch(), char_width, accumulated_width);
        }

        old_horizontal_offset + accumulated_width
    }

    // ========================================
    // Page Navigation Methods
    // ========================================

    /// Move cursor down one page with capability checking
    pub fn move_cursor_page_down(&mut self) -> Vec<ViewEvent> {
        // Check if navigation is allowed on this pane
        if !self.capabilities.contains(PaneCapabilities::NAVIGABLE) {
            return vec![]; // Navigation not allowed on this pane
        }

        let max_line_count = self.display_cache.display_line_count();
        if max_line_count == 0 {
            return vec![]; // No lines to navigate
        }

        // Calculate page size based on current pane height
        let page_size = self.pane_dimensions.height;
        if page_size == 0 {
            return vec![];
        }

        let current_line = self.display_cursor.row;
        let target_line = (current_line + page_size).min(max_line_count.saturating_sub(1));

        // Only move if there's a significant change
        if target_line == current_line {
            return vec![]; // No movement needed
        }

        // Get the display line at the target position to handle virtual column properly
        if let Some(target_display_line) = self.display_cache.get_display_line(target_line) {
            // Vim-style virtual column: try to restore the desired column position
            let virtual_col = self.virtual_column;
            let line_char_count = target_display_line.char_count();

            // Clamp virtual column to the length of the target line to prevent cursor going beyond line end
            // Mode-dependent: Normal/Visual stops at last char, Insert can go one past
            let max_col = if self.editor_mode == EditorMode::Insert {
                line_char_count // Insert mode: can be positioned after last character
            } else {
                line_char_count.saturating_sub(1) // Normal/Visual: stop at last character
            };
            let clamped_col = virtual_col.min(max_col);

            // Snap to character boundary to handle DBCS characters
            let boundary_snapped_col = target_display_line.snap_to_character_boundary(clamped_col);

            // Create new display position with proper column handling
            let new_display_pos = Position::new(target_line, boundary_snapped_col);

            // Update display cursor
            self.display_cursor = new_display_pos;

            // Sync logical cursor with new display position
            if let Some(logical_pos) = self
                .display_cache
                .display_to_logical_position(new_display_pos.row, new_display_pos.col)
            {
                let new_logical_pos = LogicalPosition::new(logical_pos.row, logical_pos.col);
                self.buffer.set_cursor(new_logical_pos);

                // Update visual selection if active
                self.update_visual_selection_on_cursor_move(new_logical_pos);
            }
        } else {
            // Fallback: if we can't get the display line, just use column 0
            let new_display_pos = Position::new(target_line, 0);
            self.display_cursor = new_display_pos;

            // Sync logical cursor
            if let Some(logical_pos) = self
                .display_cache
                .display_to_logical_position(new_display_pos.row, new_display_pos.col)
            {
                let new_logical_pos = LogicalPosition::new(logical_pos.row, logical_pos.col);
                self.buffer.set_cursor(new_logical_pos);

                // Update visual selection if active
                self.update_visual_selection_on_cursor_move(new_logical_pos);
            }
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
        let content_width = self.get_content_width();
        let visibility_events = self.ensure_cursor_visible_with_events(content_width);
        events.extend(visibility_events);

        events
    }

    /// Move cursor up one page with capability checking
    pub fn move_cursor_page_up(&mut self) -> Vec<ViewEvent> {
        // Check if navigation is allowed on this pane
        if !self.capabilities.contains(PaneCapabilities::NAVIGABLE) {
            return vec![]; // Navigation not allowed on this pane
        }

        let max_line_count = self.display_cache.display_line_count();
        if max_line_count == 0 {
            return vec![]; // No lines to navigate
        }

        // Calculate page size based on current pane height
        let page_size = self.pane_dimensions.height;
        if page_size == 0 {
            return vec![];
        }

        let current_line = self.display_cursor.row;
        let target_line = current_line.saturating_sub(page_size);

        // Only move if there's a significant change
        if target_line == current_line {
            return vec![]; // No movement needed
        }

        // Get the display line at the target position to handle virtual column properly
        if let Some(target_display_line) = self.display_cache.get_display_line(target_line) {
            // Vim-style virtual column: try to restore the desired column position
            let virtual_col = self.virtual_column;
            let line_char_count = target_display_line.char_count();

            // Clamp virtual column to the length of the target line to prevent cursor going beyond line end
            // Mode-dependent: Normal/Visual stops at last char, Insert can go one past
            let max_col = if self.editor_mode == EditorMode::Insert {
                line_char_count // Insert mode: can be positioned after last character
            } else {
                line_char_count.saturating_sub(1) // Normal/Visual: stop at last character
            };
            let clamped_col = virtual_col.min(max_col);

            // Snap to character boundary to handle DBCS characters
            let boundary_snapped_col = target_display_line.snap_to_character_boundary(clamped_col);

            // Create new display position with proper column handling
            let new_display_pos = Position::new(target_line, boundary_snapped_col);

            // Update display cursor
            self.display_cursor = new_display_pos;

            // Sync logical cursor with new display position
            if let Some(logical_pos) = self
                .display_cache
                .display_to_logical_position(new_display_pos.row, new_display_pos.col)
            {
                let new_logical_pos = LogicalPosition::new(logical_pos.row, logical_pos.col);
                self.buffer.set_cursor(new_logical_pos);

                // Update visual selection if active
                self.update_visual_selection_on_cursor_move(new_logical_pos);
            }
        } else {
            // Fallback: if we can't get the display line, just use column 0
            let new_display_pos = Position::new(target_line, 0);
            self.display_cursor = new_display_pos;

            // Sync logical cursor
            if let Some(logical_pos) = self
                .display_cache
                .display_to_logical_position(new_display_pos.row, new_display_pos.col)
            {
                let new_logical_pos = LogicalPosition::new(logical_pos.row, logical_pos.col);
                self.buffer.set_cursor(new_logical_pos);

                // Update visual selection if active
                self.update_visual_selection_on_cursor_move(new_logical_pos);
            }
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
        let content_width = self.get_content_width();
        let visibility_events = self.ensure_cursor_visible_with_events(content_width);
        events.extend(visibility_events);

        events
    }

    /// Move cursor down half a page with capability checking
    pub fn move_cursor_half_page_down(&mut self) -> Vec<ViewEvent> {
        // Check if navigation is allowed on this pane
        if !self.capabilities.contains(PaneCapabilities::NAVIGABLE) {
            return vec![]; // Navigation not allowed on this pane
        }

        let max_line_count = self.display_cache.display_line_count();
        if max_line_count == 0 {
            return vec![]; // No lines to navigate
        }

        // Calculate half page size based on current pane height
        let page_size = self.pane_dimensions.height;
        if page_size == 0 {
            return vec![];
        }
        let half_page_size = page_size.div_ceil(2); // Round up for odd numbers

        let current_line = self.display_cursor.row;
        let target_line = (current_line + half_page_size).min(max_line_count.saturating_sub(1));

        // Only move if there's a significant change
        if target_line == current_line {
            return vec![]; // No movement needed
        }

        // Get the display line at the target position to handle virtual column properly
        if let Some(target_display_line) = self.display_cache.get_display_line(target_line) {
            // Vim-style virtual column: try to restore the desired column position
            let virtual_col = self.virtual_column;
            let line_char_count = target_display_line.char_count();

            // Clamp virtual column to the length of the target line to prevent cursor going beyond line end
            // Mode-dependent: Normal/Visual stops at last char, Insert can go one past
            let max_col = if self.editor_mode == EditorMode::Insert {
                line_char_count // Insert mode: can be positioned after last character
            } else {
                line_char_count.saturating_sub(1) // Normal/Visual: stop at last character
            };
            let clamped_col = virtual_col.min(max_col);

            // Snap to character boundary to handle DBCS characters
            let boundary_snapped_col = target_display_line.snap_to_character_boundary(clamped_col);

            // Create new display position with proper column handling
            let new_display_pos = Position::new(target_line, boundary_snapped_col);

            // Update display cursor
            self.display_cursor = new_display_pos;

            // Sync logical cursor with new display position
            if let Some(logical_pos) = self
                .display_cache
                .display_to_logical_position(new_display_pos.row, new_display_pos.col)
            {
                let new_logical_pos = LogicalPosition::new(logical_pos.row, logical_pos.col);
                self.buffer.set_cursor(new_logical_pos);

                // Update visual selection if active
                self.update_visual_selection_on_cursor_move(new_logical_pos);
            }
        } else {
            // Fallback: if we can't get the display line, just use column 0
            let new_display_pos = Position::new(target_line, 0);
            self.display_cursor = new_display_pos;

            // Sync logical cursor
            if let Some(logical_pos) = self
                .display_cache
                .display_to_logical_position(new_display_pos.row, new_display_pos.col)
            {
                let new_logical_pos = LogicalPosition::new(logical_pos.row, logical_pos.col);
                self.buffer.set_cursor(new_logical_pos);

                // Update visual selection if active
                self.update_visual_selection_on_cursor_move(new_logical_pos);
            }
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
        let content_width = self.get_content_width();
        let visibility_events = self.ensure_cursor_visible_with_events(content_width);
        events.extend(visibility_events);

        events
    }

    /// Move cursor up half a page with capability checking
    pub fn move_cursor_half_page_up(&mut self) -> Vec<ViewEvent> {
        // Check if navigation is allowed on this pane
        if !self.capabilities.contains(PaneCapabilities::NAVIGABLE) {
            return vec![]; // Navigation not allowed on this pane
        }

        let max_line_count = self.display_cache.display_line_count();
        if max_line_count == 0 {
            return vec![]; // No lines to navigate
        }

        // Calculate half page size based on current pane height
        let page_size = self.pane_dimensions.height;
        if page_size == 0 {
            return vec![];
        }
        let half_page_size = page_size.div_ceil(2); // Round up for odd numbers

        let current_line = self.display_cursor.row;
        let target_line = current_line.saturating_sub(half_page_size);

        // Only move if there's a significant change
        if target_line == current_line {
            return vec![]; // No movement needed
        }

        // Get the display line at the target position to handle virtual column properly
        if let Some(target_display_line) = self.display_cache.get_display_line(target_line) {
            // Vim-style virtual column: try to restore the desired column position
            let virtual_col = self.virtual_column;
            let line_char_count = target_display_line.char_count();

            // Clamp virtual column to the length of the target line to prevent cursor going beyond line end
            // Mode-dependent: Normal/Visual stops at last char, Insert can go one past
            let max_col = if self.editor_mode == EditorMode::Insert {
                line_char_count // Insert mode: can be positioned after last character
            } else {
                line_char_count.saturating_sub(1) // Normal/Visual: stop at last character
            };
            let clamped_col = virtual_col.min(max_col);

            // Snap to character boundary to handle DBCS characters
            let boundary_snapped_col = target_display_line.snap_to_character_boundary(clamped_col);

            // Create new display position with proper column handling
            let new_display_pos = Position::new(target_line, boundary_snapped_col);

            // Update display cursor
            self.display_cursor = new_display_pos;

            // Sync logical cursor with new display position
            if let Some(logical_pos) = self
                .display_cache
                .display_to_logical_position(new_display_pos.row, new_display_pos.col)
            {
                let new_logical_pos = LogicalPosition::new(logical_pos.row, logical_pos.col);
                self.buffer.set_cursor(new_logical_pos);

                // Update visual selection if active
                self.update_visual_selection_on_cursor_move(new_logical_pos);
            }
        } else {
            // Fallback: if we can't get the display line, just use column 0
            let new_display_pos = Position::new(target_line, 0);
            self.display_cursor = new_display_pos;

            // Sync logical cursor
            if let Some(logical_pos) = self
                .display_cache
                .display_to_logical_position(new_display_pos.row, new_display_pos.col)
            {
                let new_logical_pos = LogicalPosition::new(logical_pos.row, logical_pos.col);
                self.buffer.set_cursor(new_logical_pos);

                // Update visual selection if active
                self.update_visual_selection_on_cursor_move(new_logical_pos);
            }
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
        let content_width = self.get_content_width();
        let visibility_events = self.ensure_cursor_visible_with_events(content_width);
        events.extend(visibility_events);

        events
    }
}
