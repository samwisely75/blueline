//! # PaneState Module
//!
//! Contains the PaneState struct and its implementations for managing individual pane state.
//! This includes scrolling, cursor positioning, word navigation, and display cache management.

use crate::repl::events::{EditorMode, LogicalPosition, Pane};
use crate::repl::geometry::{Dimensions, Position};
use crate::repl::models::{BufferModel, DisplayCache};
use std::ops::{Index, IndexMut};

/// Information about a wrapped line segment
#[derive(Debug, Clone)]
struct WrappedSegment {
    #[allow(dead_code)] // Used for debug/display purposes
    content: String,
    logical_start: usize,
    logical_end: usize,
}

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
    pub editor_mode: EditorMode,     // Current editor mode for this pane
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
            editor_mode: EditorMode::Normal, // Start in Normal mode
        };
        pane_state.build_display_cache(pane_width, wrap_enabled);
        pane_state
    }

    /// Build display cache for this pane's content using CharacterBuffer with word boundaries
    pub fn build_display_cache(&mut self, content_width: usize, wrap_enabled: bool) {
        tracing::info!(
            "Building display cache with ICU word segmentation: content_width={}, wrap_enabled={}",
            content_width,
            wrap_enabled
        );

        // Use CharacterBuffer directly to preserve word boundary information
        self.display_cache = self
            .build_display_cache_from_character_buffer(content_width, wrap_enabled)
            .unwrap_or_else(|e| {
                tracing::error!("Failed to build display cache with word boundaries: {}", e);
                DisplayCache::new()
            });

        tracing::debug!(
            "Display cache built: {} display lines, {} logical lines mapped",
            self.display_cache.display_lines.len(),
            self.display_cache.logical_to_display.len()
        );
    }

    /// Build display cache from CharacterBuffer preserving word boundaries
    fn build_display_cache_from_character_buffer(
        &mut self,
        content_width: usize,
        wrap_enabled: bool,
    ) -> anyhow::Result<DisplayCache> {
        use crate::repl::models::display_cache::*;
        use crate::repl::models::display_char::DisplayChar;
        use std::collections::HashMap;
        use std::time::Instant;

        // Ensure word boundaries are calculated for all lines
        let character_buffer = self.buffer.content_mut().character_buffer_mut();
        let line_count = character_buffer.line_count();

        tracing::debug!("Pre-calculating word boundaries for {} lines", line_count);

        // Pre-calculate word boundaries for all lines
        for line_idx in 0..line_count {
            character_buffer.get_line_word_boundaries(line_idx);
            if let Some(line) = character_buffer.get_line(line_idx) {
                let word_starts = line
                    .chars()
                    .iter()
                    .enumerate()
                    .filter(|(_, c)| c.is_word_start)
                    .map(|(i, _)| i)
                    .collect::<Vec<_>>();
                let word_ends = line
                    .chars()
                    .iter()
                    .enumerate()
                    .filter(|(_, c)| c.is_word_end)
                    .map(|(i, _)| i)
                    .collect::<Vec<_>>();

                if !word_starts.is_empty() || !word_ends.is_empty() {
                    tracing::debug!(
                        "Line {}: '{}' -> word_starts={:?}, word_ends={:?}",
                        line_idx,
                        line.to_string().chars().take(50).collect::<String>(), // First 50 chars
                        word_starts,
                        word_ends
                    );
                }
            }
        }
        let mut display_lines = Vec::new();
        let mut logical_to_display = HashMap::new();

        for logical_idx in 0..line_count {
            if let Some(buffer_line) = character_buffer.get_line(logical_idx) {
                let line_text = buffer_line.to_string();

                let wrapped_segments = if wrap_enabled {
                    Self::wrap_line_with_positions(&line_text, content_width)
                } else {
                    vec![WrappedSegment {
                        content: line_text.clone(),
                        logical_start: 0,
                        logical_end: buffer_line.char_count(),
                    }]
                };

                let mut display_indices = Vec::new();

                for (segment_idx, segment_info) in wrapped_segments.iter().enumerate() {
                    let display_idx = display_lines.len();
                    display_indices.push(display_idx);

                    // Create DisplayChars from the segment, preserving word boundaries
                    let mut display_chars = Vec::new();
                    let mut current_screen_col = 0;

                    // Extract the relevant BufferChars from the line
                    for logical_col in segment_info.logical_start..segment_info.logical_end {
                        if let Some(buffer_char) = buffer_line.get_char(logical_col) {
                            let display_char = DisplayChar::from_buffer_char(
                                buffer_char.clone(),
                                (display_idx, current_screen_col),
                            );
                            current_screen_col += display_char.display_width();
                            display_chars.push(display_char);
                        }
                    }

                    let display_line = DisplayLine::new(
                        display_chars,
                        logical_idx,
                        segment_info.logical_start,
                        segment_info.logical_end,
                        segment_idx > 0,
                    );

                    display_lines.push(display_line);
                }

                logical_to_display.insert(logical_idx, display_indices);
            }
        }

        Ok(DisplayCache {
            total_display_lines: display_lines.len(),
            display_lines,
            logical_to_display,
            content_width,
            content_hash: 0, // Not used with eager invalidation strategy
            generated_at: Instant::now(),
            is_valid: true,
            wrap_enabled,
        })
    }

    /// Wrap a line into segments with position tracking
    fn wrap_line_with_positions(line: &str, content_width: usize) -> Vec<WrappedSegment> {
        if content_width == 0 {
            return vec![WrappedSegment {
                content: line.to_string(),
                logical_start: 0,
                logical_end: line.chars().count(),
            }];
        }

        let mut segments = Vec::new();
        let chars: Vec<char> = line.chars().collect();
        let mut start = 0;

        while start < chars.len() {
            let end = (start + content_width).min(chars.len());
            let segment_content: String = chars[start..end].iter().collect();

            segments.push(WrappedSegment {
                content: segment_content,
                logical_start: start,
                logical_end: end,
            });

            start = end;
        }

        if segments.is_empty() {
            segments.push(WrappedSegment {
                content: String::new(),
                logical_start: 0,
                logical_end: 0,
            });
        }

        segments
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
        } else if content_width > 0 {
            // MODE-AWARE HORIZONTAL SCROLL: Different trigger points for Insert vs Normal mode
            let should_scroll = match self.editor_mode {
                EditorMode::Insert => {
                    // Insert mode: Scroll early to make room for typing next character
                    display_pos.col >= old_horizontal_offset + content_width
                }
                _ => {
                    // Normal/Visual mode: Only scroll when absolutely necessary
                    display_pos.col > old_horizontal_offset + content_width
                }
            };

            if should_scroll {
                // DISPLAY-COLUMN-AWARE HORIZONTAL SCROLL: Calculate the actual display columns needed
                // to make the cursor visible, accounting for DBCS character widths

                // CHARACTER-WIDTH-AWARE HORIZONTAL SCROLL: When scrolling, we need to scroll past
                // complete characters, accounting for their actual display widths
                let min_scroll_needed = display_pos
                    .col
                    .saturating_sub(content_width.saturating_sub(1));

                // CHARACTER-WIDTH-BASED SCROLL: HSO should equal the width of the leftmost character
                if let Some(display_line) = self.display_cache.get_display_line(display_pos.row) {
                    if let Some(leftmost_char) =
                        display_line.char_at_display_col(old_horizontal_offset)
                    {
                        // Scroll by the width of the leftmost character
                        let char_width = leftmost_char.display_width();
                        new_horizontal_offset = old_horizontal_offset + char_width;

                        tracing::debug!("PaneState::ensure_cursor_visible: cursor off-screen at pos {}, scrolling past leftmost char '{}' width={} from {} to {}", 
                            display_pos.col, leftmost_char.ch(), char_width, old_horizontal_offset, new_horizontal_offset);
                    } else {
                        // Fallback: no character found, use mathematical minimum
                        new_horizontal_offset = old_horizontal_offset + min_scroll_needed;
                        tracing::debug!("PaneState::ensure_cursor_visible: no leftmost char found, using min_scroll_needed={}", min_scroll_needed);
                    }
                } else {
                    // Fallback: no display line found
                    new_horizontal_offset = old_horizontal_offset + min_scroll_needed;
                    tracing::debug!("PaneState::ensure_cursor_visible: no display line found, using min_scroll_needed={}", min_scroll_needed);
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

    // Mode management methods
    /// Get current editor mode for this pane
    pub fn get_mode(&self) -> EditorMode {
        self.editor_mode
    }

    /// Set editor mode for this pane
    pub fn set_mode(&mut self, mode: EditorMode) {
        self.editor_mode = mode;
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
