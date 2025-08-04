//! # PaneState Module
//!
//! Contains the PaneState struct and its implementations for managing individual pane state.
//! This includes scrolling, cursor positioning, word navigation, and display cache management.

use crate::repl::events::{LogicalPosition, Pane};
use crate::repl::geometry::{Dimensions, Position};
use crate::repl::models::{BufferModel, DisplayCache};
use std::ops::{Index, IndexMut};

/// Type alias for optional position
pub type OptionalPosition = Option<Position>;

/// Result of a scrolling operation, contains information needed for event emission
#[derive(Debug, Clone)]
pub struct ScrollResult {
    pub old_offset: usize,
    pub new_offset: usize,
    pub cursor_moved: bool,
}

/// Result of a cursor movement operation, contains information needed for event emission
#[derive(Debug, Clone)]
pub struct CursorMoveResult {
    pub cursor_moved: bool,
    pub old_display_pos: Position,
    pub new_display_pos: Position,
}

/// Result of a scroll adjustment for cursor visibility
#[derive(Debug, Clone)]
pub struct ScrollAdjustResult {
    pub vertical_changed: bool,
    pub horizontal_changed: bool,
    pub old_vertical_offset: usize,
    pub new_vertical_offset: usize,
    pub old_horizontal_offset: usize,
    pub new_horizontal_offset: usize,
}

/// State container for a single pane (Request or Response)
#[derive(Debug, Clone)]
pub struct PaneState {
    pub buffer: BufferModel,
    pub display_cache: DisplayCache,
    pub display_cursor: Position, // (display_line, display_column)
    pub scroll_offset: Position,  // (vertical, horizontal)
    pub visual_selection_start: Option<LogicalPosition>,
    pub visual_selection_end: Option<LogicalPosition>,
    pub pane_dimensions: Dimensions, // (width, height)
}

impl PaneState {
    pub fn new(pane: Pane, pane_width: usize, pane_height: usize, wrap_enabled: bool) -> Self {
        let mut pane_state = Self {
            buffer: BufferModel::new(pane),
            display_cache: DisplayCache::new(),
            display_cursor: Position::origin(),
            scroll_offset: Position::origin(),
            visual_selection_start: None,
            visual_selection_end: None,
            pane_dimensions: Dimensions::new(pane_width, pane_height),
        };
        pane_state.build_display_cache(pane_width, wrap_enabled);
        pane_state
    }

    /// Build display cache for this pane's content
    pub fn build_display_cache(&mut self, content_width: usize, wrap_enabled: bool) {
        let lines = self.buffer.content().lines().to_vec();
        self.display_cache =
            crate::repl::models::build_display_cache(&lines, content_width, wrap_enabled)
                .unwrap_or_else(|_| DisplayCache::new());
    }

    /// Get page size for scrolling (pane height minus UI chrome)
    pub fn get_page_size(&self) -> usize {
        self.pane_dimensions.height.saturating_sub(2).max(1)
    }

    /// Get half page size for scrolling
    pub fn get_half_page_size(&self) -> usize {
        (self.pane_dimensions.height / 2).max(1)
    }

    /// Get content width for this pane
    pub fn get_content_width(&self) -> usize {
        self.pane_dimensions.width
    }

    /// Update pane dimensions (for terminal resize)
    pub fn update_dimensions(&mut self, width: usize, height: usize) {
        self.pane_dimensions = Dimensions::new(width, height);
    }

    /// Handle horizontal scrolling within this pane
    pub fn scroll_horizontally(&mut self, direction: i32, amount: usize) -> ScrollResult {
        use crate::repl::events::LogicalPosition;

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

    /// Handle vertical page scrolling within this pane
    pub fn scroll_vertically_by_page(&mut self, direction: i32) -> ScrollResult {
        use crate::repl::events::LogicalPosition;

        let old_offset = self.scroll_offset.row; // vertical offset
        let page_size = self.get_page_size();

        // Vim typically scrolls by (page_size - 1) to maintain some context
        let scroll_amount = page_size.saturating_sub(1).max(1);

        tracing::debug!(
            "scroll_vertically_by_page: pane_dimensions=({}, {}), page_size={}, scroll_amount={}",
            self.pane_dimensions.width,
            self.pane_dimensions.height,
            page_size,
            scroll_amount
        );

        // Prevent scrolling beyond actual content bounds
        let max_scroll_offset = self
            .display_cache
            .display_line_count()
            .saturating_sub(page_size)
            .max(0);

        let new_offset = if direction > 0 {
            std::cmp::min(old_offset + scroll_amount, max_scroll_offset)
        } else {
            old_offset.saturating_sub(scroll_amount)
        };

        // If scroll offset wouldn't change, don't do anything
        if new_offset == old_offset {
            return ScrollResult {
                old_offset,
                new_offset: old_offset,
                cursor_moved: false,
            };
        }

        self.scroll_offset.row = new_offset;

        // BUGFIX: Move cursor by exactly the scroll amount in display coordinates
        // This should be simple: if we scroll by N display lines, cursor moves by N display lines
        let current_cursor = self.buffer.cursor();
        let mut cursor_moved = false;

        tracing::debug!("scroll_vertically_by_page: old_offset={}, new_offset={}, scroll_amount={}, current_cursor=({}, {})",
            old_offset, new_offset, scroll_amount, current_cursor.line, current_cursor.column);

        // Get current cursor display position
        if let Some(current_display_pos) = self
            .display_cache
            .logical_to_display_position(current_cursor.line, current_cursor.column)
        {
            // Move cursor by exactly the scroll amount in display lines
            let scroll_delta = new_offset as i32 - old_offset as i32;
            let new_display_line = (current_display_pos.row as i32 + scroll_delta).max(0) as usize;
            let new_display_col = current_display_pos.col; // Keep same column position

            tracing::debug!("scroll_vertically_by_page: current_display=({}, {}), scroll_delta={}, new_display=({}, {})",
                current_display_pos.row, current_display_pos.col, scroll_delta, new_display_line, new_display_col);

            // Convert new display position back to logical position
            if let Some(logical_pos) = self
                .display_cache
                .display_to_logical_position(new_display_line, new_display_col)
            {
                let cursor_position = LogicalPosition::new(logical_pos.row, logical_pos.col);
                let clamped_position = self.buffer.content().clamp_position(cursor_position);

                tracing::debug!(
                    "scroll_vertically_by_page: new_logical=({}, {}), clamped=({}, {})",
                    logical_pos.row,
                    logical_pos.col,
                    clamped_position.line,
                    clamped_position.column
                );

                // Update cursor position
                if clamped_position != current_cursor {
                    self.buffer.set_cursor(clamped_position);
                    self.display_cursor = Position::new(new_display_line, new_display_col);
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

    /// Handle vertical half-page scrolling within this pane
    pub fn scroll_vertically_by_half_page(&mut self, direction: i32) -> ScrollResult {
        use crate::repl::events::LogicalPosition;

        let old_offset = self.scroll_offset.row; // vertical offset
        let page_size = self.get_page_size();
        let scroll_amount = self.get_half_page_size();

        // Prevent half-page scrolling beyond actual content bounds
        let max_scroll_offset = self
            .display_cache
            .display_line_count()
            .saturating_sub(page_size)
            .max(0);

        let new_offset = if direction > 0 {
            std::cmp::min(old_offset + scroll_amount, max_scroll_offset)
        } else {
            old_offset.saturating_sub(scroll_amount)
        };

        // If scroll offset wouldn't change, don't do anything
        if new_offset == old_offset {
            return ScrollResult {
                old_offset,
                new_offset: old_offset,
                cursor_moved: false,
            };
        }

        self.scroll_offset.row = new_offset;

        // BUGFIX: Move cursor by exactly the scroll amount in display coordinates
        // Simple approach: cursor moves by the same amount as the scroll
        let current_cursor = self.buffer.cursor();
        let mut cursor_moved = false;

        // Get current cursor display position
        if let Some(current_display_pos) = self
            .display_cache
            .logical_to_display_position(current_cursor.line, current_cursor.column)
        {
            // Move cursor by exactly the scroll amount in display lines
            let scroll_delta = new_offset as i32 - old_offset as i32;
            let new_display_line = (current_display_pos.row as i32 + scroll_delta).max(0) as usize;
            let new_display_col = current_display_pos.col; // Keep same column position

            // Convert new display position back to logical position
            if let Some(logical_pos) = self
                .display_cache
                .display_to_logical_position(new_display_line, new_display_col)
            {
                let cursor_position = LogicalPosition::new(logical_pos.row, logical_pos.col);
                let clamped_position = self.buffer.content().clamp_position(cursor_position);

                // Update cursor position
                if clamped_position != current_cursor {
                    self.buffer.set_cursor(clamped_position);
                    self.display_cursor = Position::new(new_display_line, new_display_col);
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
        use crate::repl::events::LogicalPosition;

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
        if display_pos.col < old_horizontal_offset {
            new_horizontal_offset = display_pos.col;
            tracing::debug!("PaneState::ensure_cursor_visible: cursor off-screen left, adjusting horizontal offset to {}", new_horizontal_offset);
        } else if display_pos.col >= old_horizontal_offset + content_width && content_width > 0 {
            new_horizontal_offset = display_pos
                .col
                .saturating_sub(content_width.saturating_sub(1));
            tracing::debug!("PaneState::ensure_cursor_visible: cursor off-screen right at pos {}, adjusting horizontal offset from {} to {}", display_pos.col, old_horizontal_offset, new_horizontal_offset);
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

    /// Find the position of the beginning of the next word from current position
    /// Returns None if no next word is found
    /// Now supports Japanese characters as word characters
    pub fn find_next_word_position(&self, current_pos: Position) -> OptionalPosition {
        let mut current_line = current_pos.row;
        let mut current_col = current_pos.col;

        // Loop through display lines to find next word
        while current_line < self.display_cache.display_line_count() {
            if let Some(line_info) = self.display_cache.get_display_line(current_line) {
                // Try to find next word on current line
                if let Some(new_col) = line_info.find_next_word_boundary(current_col) {
                    return Some(Position::new(current_line, new_col));
                }

                // Move to next line and start at beginning
                current_line += 1;
                current_col = 0;

                // If we moved to next line, look for first word on that line
                if current_line < self.display_cache.display_line_count() {
                    if let Some(next_line_info) = self.display_cache.get_display_line(current_line)
                    {
                        if let Some(new_col) = next_line_info.find_next_word_boundary(0) {
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
    pub fn find_previous_word_position(&self, current_pos: Position) -> OptionalPosition {
        let mut current_line = current_pos.row;
        let mut current_col = current_pos.col;

        tracing::debug!(
            "find_previous_word_position: starting at display_pos=({}, {})",
            current_line,
            current_col
        );

        // Loop through display lines backwards to find previous word
        while let Some(line_info) = self.display_cache.get_display_line(current_line) {
            tracing::debug!("find_previous_word_position: checking line {} with {} chars, display_width={}, current_col={}", 
                current_line, line_info.char_count(), line_info.display_width(), current_col);

            // Try to find previous word on current line
            if let Some(new_col) = line_info.find_previous_word_boundary(current_col) {
                tracing::debug!(
                    "find_previous_word_position: found word on line {} at col {}",
                    current_line,
                    new_col
                );
                return Some(Position::new(current_line, new_col));
            }

            tracing::debug!(
                "find_previous_word_position: no word found on line {}, moving to previous line",
                current_line
            );

            // If we can't find a previous word on this line, move to previous line
            if current_line > 0 {
                current_line -= 1;
                if let Some(prev_line_info) = self.display_cache.get_display_line(current_line) {
                    current_col = prev_line_info.display_width();
                    tracing::debug!("find_previous_word_position: moved to line {}, set current_col to display_width={}", 
                        current_line, current_col);
                    // Try to find previous word from the end of the previous line
                    if let Some(new_col) = prev_line_info.find_previous_word_boundary(current_col) {
                        tracing::debug!(
                            "find_previous_word_position: found word on prev line {} at col {}",
                            current_line,
                            new_col
                        );
                        return Some(Position::new(current_line, new_col));
                    }
                    tracing::debug!(
                        "find_previous_word_position: no word found on prev line {}",
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
    pub fn find_end_of_word_position(&self, current_pos: Position) -> OptionalPosition {
        let mut current_line = current_pos.row;
        let mut current_col = current_pos.col;

        // Loop through display lines to find end of word
        while current_line < self.display_cache.display_line_count() {
            if let Some(line_info) = self.display_cache.get_display_line(current_line) {
                // Try to find end of word on current line
                if let Some(new_col) = line_info.find_end_of_word(current_col) {
                    return Some(Position::new(current_line, new_col));
                }

                // Move to next line
                current_line += 1;
                current_col = 0;

                // Try to find end of word on next line from beginning
                if current_line < self.display_cache.display_line_count() {
                    if let Some(next_line_info) = self.display_cache.get_display_line(current_line)
                    {
                        if let Some(new_col) = next_line_info.find_end_of_word(0) {
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
}

/// Array indexing for panes to enable clean access patterns
impl Index<Pane> for [PaneState; 2] {
    type Output = PaneState;
    fn index(&self, pane: Pane) -> &Self::Output {
        match pane {
            Pane::Request => &self[0],
            Pane::Response => &self[1],
        }
    }
}

impl IndexMut<Pane> for [PaneState; 2] {
    fn index_mut(&mut self, pane: Pane) -> &mut Self::Output {
        match pane {
            Pane::Request => &mut self[0],
            Pane::Response => &mut self[1],
        }
    }
}
