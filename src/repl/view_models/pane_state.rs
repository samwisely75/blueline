//! # PaneState Module
//!
//! Contains the PaneState struct and its implementations for managing individual pane state.
//! This includes scrolling, cursor positioning, word navigation, and display cache management.
//!
//! HIGH-LEVEL ARCHITECTURE:
//! PaneState encapsulates all state and operations for a single editor pane:
//! - Manages cursor position in both logical and display coordinates
//! - Handles horizontal and vertical scrolling with bounds checking
//! - Coordinates text operations with DisplayCache for proper wrapping
//! - Maintains editor mode state and line number width calculations
//!
//! CORE RESPONSIBILITIES:
//! 1. Cursor Management: Tracks logical position and converts to display coordinates
//! 2. Scroll Coordination: Maintains viewport position and cursor visibility
//! 3. Text Operations: Handles character insertion/deletion with proper event emission
//! 4. Display Integration: Works with DisplayCache for text wrapping and rendering
//!
//! CRITICAL ARCHITECTURAL DECISION:
//! PaneState eliminates feature envy by keeping all pane-specific logic centralized.
//! Previously scattered across multiple classes, this consolidation improves maintainability
//! and follows the Single Responsibility Principle.

use crate::repl::events::{
    EditorMode, LogicalPosition, LogicalRange, ModelEvent, Pane, PaneCapabilities, ViewEvent,
};
use crate::repl::geometry::{Dimensions, Position};
use crate::repl::models::{BufferModel, DisplayCache, DisplayLine};
use std::ops::{Index, IndexMut};

/// Minimum width for line number column as specified in requirements
const MIN_LINE_NUMBER_WIDTH: usize = 3;

/// Type alias for deletion result: (deleted_text, model_event)
type DeletionResult = Option<(String, ModelEvent)>;

/// Type alias for visual selection state (start_position, end_position)
type VisualSelection = (Option<LogicalPosition>, Option<LogicalPosition>);

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
///
/// HIGH-LEVEL DESIGN:
/// This struct aggregates all state needed for a single editor pane:
/// - BufferModel: Contains the actual text content and logical operations
/// - DisplayCache: Handles text wrapping and display line calculations  
/// - Position tracking: Maintains both logical and display cursor coordinates
/// - Scroll management: Tracks viewport offset for large content navigation
/// - Visual selection: Supports Vim-style visual mode selections
/// - Mode state: Each pane maintains its own editor mode independently
#[derive(Debug, Clone)]
pub struct PaneState {
    pub buffer: BufferModel,
    pub display_cache: DisplayCache,
    pub display_cursor: Position, // (display_line, display_column)
    pub scroll_offset: Position,  // (vertical, horizontal)
    pub visual_selection_start: Option<LogicalPosition>,
    pub visual_selection_end: Option<LogicalPosition>,
    pub pane_dimensions: Dimensions,    // (width, height)
    pub editor_mode: EditorMode,        // Current editor mode for this pane
    pub line_number_width: usize,       // Width needed for line numbers display
    pub virtual_column: usize,          // Vim-style virtual column - desired cursor position
    pub capabilities: PaneCapabilities, // What operations are allowed on this pane
}

impl PaneState {
    pub fn new(
        pane: Pane,
        pane_width: usize,
        pane_height: usize,
        wrap_enabled: bool,
        capabilities: PaneCapabilities,
    ) -> Self {
        let mut pane_state = Self {
            buffer: BufferModel::new(pane),
            display_cache: DisplayCache::new(),
            display_cursor: Position::origin(),
            scroll_offset: Position::origin(),
            visual_selection_start: None,
            visual_selection_end: None,
            pane_dimensions: Dimensions::new(pane_width, pane_height),
            editor_mode: EditorMode::Normal, // Start in Normal mode
            line_number_width: MIN_LINE_NUMBER_WIDTH, // Start with minimum width
            virtual_column: 0,               // Start at column 0
            capabilities,                    // Set capabilities based on pane type
        };
        pane_state.build_display_cache(pane_width, wrap_enabled, 4); // Default tab width, will be updated later
                                                                     // Calculate initial line number width based on content
        pane_state.update_line_number_width();
        pane_state
    }

    /// Build display cache for this pane's content using CharacterBuffer with word boundaries
    pub fn build_display_cache(
        &mut self,
        content_width: usize,
        wrap_enabled: bool,
        tab_width: usize,
    ) {
        tracing::info!(
            "Building display cache with unicode-segmentation word boundaries: content_width={}, wrap_enabled={}, tab_width={}",
            content_width,
            wrap_enabled,
            tab_width
        );

        // Use CharacterBuffer directly to preserve word boundary information
        self.display_cache = self
            .build_display_cache_from_character_buffer(content_width, wrap_enabled, tab_width)
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
        tab_width: usize,
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
                            let display_char = DisplayChar::from_buffer_char_with_tab_width(
                                buffer_char.clone(),
                                (display_idx, current_screen_col),
                                tab_width,
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

    /// Calculate display width for a range of buffer characters
    fn calculate_display_width(
        buffer_chars: &[crate::repl::models::buffer_char::BufferChar],
    ) -> usize {
        use unicode_width::UnicodeWidthChar;
        buffer_chars
            .iter()
            .map(|bc| UnicodeWidthChar::width(bc.ch).unwrap_or(0))
            .sum()
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

        // Convert line to BufferChars for accurate display width calculation
        use crate::repl::models::buffer_char::BufferLine;
        let buffer_line = BufferLine::from_string(line);
        let buffer_chars = buffer_line.chars();

        let mut segments = Vec::new();
        let mut current_char_pos = 0;
        let total_chars = buffer_chars.len();

        while current_char_pos < total_chars {
            let mut current_display_width = 0;
            let mut segment_end_char_pos = current_char_pos;
            let mut last_word_boundary_char_pos = None;

            // Find how many characters fit within content_width display columns
            while segment_end_char_pos < total_chars && current_display_width < content_width {
                let buffer_char = &buffer_chars[segment_end_char_pos];
                use unicode_width::UnicodeWidthChar;
                let char_display_width = UnicodeWidthChar::width(buffer_char.ch).unwrap_or(0);

                // Check if adding this character would exceed the content width
                if current_display_width + char_display_width > content_width {
                    break;
                }

                // Mark word boundaries for better wrapping
                if buffer_char.ch.is_whitespace() {
                    last_word_boundary_char_pos = Some(segment_end_char_pos + 1);
                }

                current_display_width += char_display_width;
                segment_end_char_pos += 1;
            }

            // If we haven't advanced and we're not at the last character, force advance by one
            // to prevent infinite loops with zero-width characters
            if segment_end_char_pos == current_char_pos && current_char_pos < total_chars {
                segment_end_char_pos = current_char_pos + 1;
            }

            // Try to break at word boundary if possible (only if we have more characters to process)
            let actual_end = if segment_end_char_pos < total_chars {
                if let Some(word_boundary) = last_word_boundary_char_pos {
                    if word_boundary > current_char_pos {
                        word_boundary
                    } else {
                        segment_end_char_pos
                    }
                } else {
                    segment_end_char_pos
                }
            } else {
                segment_end_char_pos
            };

            // Extract the segment content
            let segment_content: String = buffer_chars[current_char_pos..actual_end]
                .iter()
                .map(|bc| bc.ch)
                .collect();

            segments.push(WrappedSegment {
                content: segment_content,
                logical_start: current_char_pos,
                logical_end: actual_end,
            });

            current_char_pos = actual_end;
        }

        // WRAP MODE CURSOR POSITIONING FIX: Create an empty continuation segment for cursor positioning
        // When content exactly fills display lines, we need a place for the cursor to wrap to
        if !segments.is_empty() {
            let last_segment = segments.last().unwrap();
            // Only process if we've reached the end of all characters
            if last_segment.logical_end == total_chars && total_chars > 0 {
                // Check if the last segment exactly fills a display line
                let segment_display_width = Self::calculate_display_width(
                    &buffer_chars[last_segment.logical_start..last_segment.logical_end],
                );

                // If this segment exactly fills the content width, create empty continuation
                if segment_display_width == content_width {
                    segments.push(WrappedSegment {
                        content: String::new(),
                        logical_start: total_chars,
                        logical_end: total_chars,
                    });
                }
            }
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
                    EditorMode::Insert => {
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

    // Line number management methods
    /// Update line number width based on current buffer content
    /// This should be called whenever buffer content changes
    pub fn update_line_number_width(&mut self) {
        let content = self.buffer.content().get_text();
        let line_count = if content.is_empty() {
            1 // At least show line 1 even for empty content
        } else {
            content.lines().count().max(1)
        };

        // Calculate width needed for the largest line number to prevent cursor positioning bugs
        let width = line_count.to_string().len();

        // Minimum width as specified in the requirements (never smaller than 3)
        self.line_number_width = width.max(MIN_LINE_NUMBER_WIDTH);
    }

    /// Get current line number width for this pane
    pub fn get_line_number_width(&self) -> usize {
        self.line_number_width
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

    // Virtual column management methods for Vim-style navigation

    /// Update virtual column to current cursor column (called when horizontal movement occurs)
    pub fn update_virtual_column(&mut self) {
        self.virtual_column = self.display_cursor.col;
    }

    /// Get the current virtual column
    pub fn get_virtual_column(&self) -> usize {
        self.virtual_column
    }

    /// Set virtual column explicitly (used for restoring desired position)
    pub fn set_virtual_column(&mut self, column: usize) {
        self.virtual_column = column;
    }

    /// Get the capabilities of this pane
    pub fn get_capabilities(&self) -> PaneCapabilities {
        self.capabilities
    }

    /// Check if this pane has a specific capability
    pub fn has_capability(&self, capability: PaneCapabilities) -> bool {
        self.capabilities.contains(capability)
    }

    /// Extract text from the current visual selection based on visual mode
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
                        let line_length = line.len();

                        // Skip lines that are too short to have content in the block region
                        if left_col < line_length {
                            let actual_right_col = (right_col + 1).min(line_length); // +1 to include character at end position
                            let block_text = &line[left_col..actual_right_col];
                            selected_text.push_str(block_text);
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
                        let start_col = selection_start.column.min(line.len());
                        let end_col = (selection_end.column + 1).min(line.len()); // +1 to include character at end position
                        selected_text.push_str(&line[start_col..end_col]);
                    }
                } else {
                    // Multi-line selection
                    for line_num in selection_start.line..=selection_end.line {
                        if let Some(line) = content.get_line(line_num) {
                            if line_num == selection_start.line {
                                // First line: from start column to end
                                let start_col = selection_start.column.min(line.len());
                                selected_text.push_str(&line[start_col..]);
                            } else if line_num == selection_end.line {
                                // Last line: from beginning to end column
                                let end_col = (selection_end.column + 1).min(line.len());
                                selected_text.push_str(&line[..end_col]);
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
    /// - `tab_width`: Tab stop width for formatting
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

    /// Join current line with previous line (backspace at beginning of line)
    fn join_with_previous_line(
        &mut self,
        current_cursor: LogicalPosition,
        content_width: usize,
        wrap_enabled: bool,
        tab_width: usize,
    ) -> Vec<ViewEvent> {
        tracing::debug!("🗑️  At line start, joining with previous line");

        // Get length of previous line to position cursor correctly
        let prev_line_length = self
            .buffer
            .content()
            .get_line(current_cursor.line - 1)
            .map(|line| line.len())
            .unwrap_or(0);

        // Create range to delete the newline character (join lines)
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

        // Move cursor to end of previous line (where the join happened)
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

        // Cursor stays in same position after successful deletion
        tracing::debug!(
            "🗑️  Deleted character after cursor, cursor position unchanged: {:?}",
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

    /// Join current line with next line (delete at end of line)
    fn join_with_next_line(
        &mut self,
        current_cursor: LogicalPosition,
        content_width: usize,
        wrap_enabled: bool,
        tab_width: usize,
    ) -> Vec<ViewEvent> {
        tracing::debug!("🗑️  At line end, joining with next line");

        // Create range to delete the newline character (join lines)
        // We delete from cursor position to start of next line
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

        // Cursor stays at current position (end of merged line)
        tracing::debug!(
            "🗑️  Joined lines, cursor position unchanged: {:?}",
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

    /// Start visual selection at current cursor position
    pub fn start_visual_selection(&mut self) -> Vec<ViewEvent> {
        // Check if visual selection is allowed on this pane
        if !self.capabilities.contains(PaneCapabilities::SELECTABLE) {
            return vec![]; // Selection not allowed on this pane
        }

        let current_cursor = self.buffer.cursor();
        self.visual_selection_start = Some(current_cursor);
        self.visual_selection_end = Some(current_cursor);

        tracing::info!(
            "🎯 PaneState::start_visual_selection at position {:?}",
            current_cursor
        );

        vec![
            ViewEvent::CurrentAreaRedrawRequired,
            ViewEvent::StatusBarUpdateRequired,
            ViewEvent::ActiveCursorUpdateRequired,
        ]
    }

    /// End visual selection and clear selection state
    pub fn end_visual_selection(&mut self) -> Vec<ViewEvent> {
        self.visual_selection_start = None;
        self.visual_selection_end = None;

        tracing::info!("🎯 PaneState::end_visual_selection - cleared selection state");

        vec![
            ViewEvent::CurrentAreaRedrawRequired,
            ViewEvent::StatusBarUpdateRequired,
            ViewEvent::ActiveCursorUpdateRequired,
        ]
    }

    /// Update visual selection end position
    pub fn update_visual_selection(&mut self, position: LogicalPosition) -> Vec<ViewEvent> {
        // Check if visual selection is allowed on this pane
        if !self.capabilities.contains(PaneCapabilities::SELECTABLE) {
            return vec![]; // Selection not allowed on this pane
        }

        if self.visual_selection_start.is_some() {
            self.visual_selection_end = Some(position);
            tracing::debug!(
                "🎯 PaneState::update_visual_selection end position to {:?}",
                position
            );
            vec![ViewEvent::CurrentAreaRedrawRequired]
        } else {
            vec![]
        }
    }

    /// Get current visual selection state
    pub fn get_visual_selection(&self) -> VisualSelection {
        (self.visual_selection_start, self.visual_selection_end)
    }

    /// Check if a position is within the current visual selection
    pub fn is_position_selected(&self, position: LogicalPosition) -> bool {
        // Early return if no selection exists
        let (Some(start), Some(end)) = (self.visual_selection_start, self.visual_selection_end)
        else {
            tracing::trace!("🎯 is_position_selected: no visual selection active");
            return false;
        };

        let editor_mode = self.editor_mode;
        tracing::trace!(
            "🎯 is_position_selected: checking position {:?} against selection {:?} to {:?} in mode {:?}",
            position,
            start,
            end,
            editor_mode
        );

        // Delegate to mode-specific selection checking
        match editor_mode {
            EditorMode::Visual => self.is_position_in_character_selection(position, start, end),
            EditorMode::VisualLine => self.is_position_in_line_selection(position, start, end),
            EditorMode::VisualBlock => self.is_position_in_block_selection(position, start, end),
            _ => {
                // Not in a visual mode, no selection
                tracing::trace!(
                    "🎯 is_position_selected: not in visual mode ({:?}), returning false",
                    editor_mode
                );
                false
            }
        }
    }

    /// Check if position is in character-wise visual selection
    fn is_position_in_character_selection(
        &self,
        position: LogicalPosition,
        start: LogicalPosition,
        end: LogicalPosition,
    ) -> bool {
        let (actual_start, actual_end) = if start <= end {
            (start, end)
        } else {
            (end, start)
        };

        position >= actual_start && position <= actual_end
    }

    /// Check if position is in line-wise visual selection
    fn is_position_in_line_selection(
        &self,
        position: LogicalPosition,
        start: LogicalPosition,
        end: LogicalPosition,
    ) -> bool {
        let first_line = start.line.min(end.line);
        let last_line = start.line.max(end.line);

        position.line >= first_line && position.line <= last_line
    }

    /// Check if position is in block-wise visual selection
    fn is_position_in_block_selection(
        &self,
        position: LogicalPosition,
        start: LogicalPosition,
        end: LogicalPosition,
    ) -> bool {
        let first_line = start.line.min(end.line);
        let last_line = start.line.max(end.line);
        let first_col = start.column.min(end.column);
        let last_col = start.column.max(end.column);

        position.line >= first_line
            && position.line <= last_line
            && position.column >= first_col
            && position.column <= last_col
    }

    /// Update visual selection end position during cursor movement
    /// Returns Some(ViewEvent) if visual selection was updated, None otherwise
    pub fn update_visual_selection_on_cursor_move(
        &mut self,
        new_position: LogicalPosition,
    ) -> Option<ViewEvent> {
        // Only update if we have an active selection and selection is allowed
        if self.visual_selection_start.is_some()
            && self.capabilities.contains(PaneCapabilities::SELECTABLE)
        {
            self.visual_selection_end = Some(new_position);
            tracing::debug!(
                "🎯 PaneState::update_visual_selection_on_cursor_move to {:?}",
                new_position
            );
            Some(ViewEvent::CurrentAreaRedrawRequired)
        } else {
            None
        }
    }

    /// Delete the currently selected text based on visual mode
    /// Returns the deleted text and the ModelEvent if successful
    pub fn delete_selected_text(&mut self) -> DeletionResult {
        // Extract selection boundaries
        let (start, end) = (self.visual_selection_start?, self.visual_selection_end?);

        // Extract the text before deleting it (for feedback/undo)
        let selected_text = self.get_selected_text()?;

        // Determine the actual deletion range based on visual mode
        let deletion_range = match self.editor_mode {
            EditorMode::VisualLine => {
                // Line-wise deletion: delete entire lines including their newlines
                let first_line = start.line.min(end.line);
                let last_line = start.line.max(end.line);
                let total_lines = self.buffer.content().line_count();

                // If we're deleting the last line(s), we need special handling
                if last_line >= total_lines.saturating_sub(1) {
                    // Deleting lines that include the last line - delete from start of first line
                    // to end of last line without including a non-existent newline
                    let last_line_length = self.buffer.content().line_length(last_line);
                    LogicalRange::new(
                        LogicalPosition::new(first_line, 0),
                        LogicalPosition::new(last_line, last_line_length),
                    )
                } else {
                    // Normal case - delete from start of first line to start of line after last line
                    // This includes the newline of the last deleted line
                    LogicalRange::new(
                        LogicalPosition::new(first_line, 0),
                        LogicalPosition::new(last_line + 1, 0),
                    )
                }
            }
            EditorMode::VisualBlock => {
                // Block-wise deletion: delete rectangular region across multiple lines
                // This is more complex than other modes as we need to delete from each line individually
                return self.delete_visual_block_selection();
            }
            _ => {
                // Character-wise deletion (Visual mode)
                LogicalRange::new(
                    LogicalPosition::new(start.line.min(end.line), start.column.min(end.column)),
                    LogicalPosition::new(start.line.max(end.line), start.column.max(end.column)),
                )
            }
        };

        // Perform the actual deletion using the buffer's delete_range method
        let pane = self.buffer.pane();
        let model_event = self
            .buffer
            .content_mut()
            .delete_range(pane, deletion_range)?;

        // Clear the visual selection after successful deletion
        self.visual_selection_start = None;
        self.visual_selection_end = None;

        Some((selected_text, model_event))
    }

    /// Handle Visual Block deletion - delete rectangular region across multiple lines
    fn delete_visual_block_selection(&mut self) -> DeletionResult {
        let (start, end) = (self.visual_selection_start?, self.visual_selection_end?);

        // Extract the text before deleting it
        let selected_text = self.get_selected_text()?;

        // Calculate the rectangular region
        // VISUAL BLOCK FIX: Use selection_start as anchor and selection_end as current cursor
        let top_line = start.line.min(end.line);
        let bottom_line = start.line.max(end.line);
        let start_col = start.column; // Column where Visual Block mode started
        let end_col = end.column; // Current cursor column

        // Determine selection direction and boundaries
        let (left_col, right_col) = if start_col <= end_col {
            (start_col, end_col)
        } else {
            (end_col, start_col)
        };

        // Delete from bottom to top to avoid line index shifting
        let pane = self.buffer.pane();
        let mut events = Vec::new();

        for line_num in (top_line..=bottom_line).rev() {
            // Only delete within the column range for this line
            let line_length = self.buffer.content().line_length(line_num);

            // Skip lines that are too short to have content in the block region
            if left_col >= line_length {
                continue;
            }

            // Calculate actual deletion range for this line
            let actual_right_col = right_col.min(line_length.saturating_sub(1));

            if left_col <= actual_right_col {
                let delete_range = LogicalRange::new(
                    LogicalPosition::new(line_num, left_col),
                    LogicalPosition::new(line_num, actual_right_col + 1),
                );

                if let Some(event) = self.buffer.content_mut().delete_range(pane, delete_range) {
                    events.push(event);
                }
            }
        }

        // Clear the visual selection after successful deletion
        self.visual_selection_start = None;
        self.visual_selection_end = None;

        // Return the first deletion event (they should all be similar)
        events
            .into_iter()
            .next()
            .map(|first_event| (selected_text, first_event))
    }

    // Helper methods to reduce arrow code complexity

    /// Check if character at cursor position extends beyond visible area
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
    // Basic Cursor Movement Methods
    // ========================================

    /// Move cursor left with capability checking and visual selection support
    pub fn move_cursor_left(&mut self, content_width: usize) -> Vec<ViewEvent> {
        // Check if navigation is allowed on this pane
        if !self.capabilities.contains(PaneCapabilities::NAVIGABLE) {
            return vec![]; // Navigation not allowed on this pane
        }

        let current_display_pos = self.display_cursor;
        let mut moved = false;

        // Check if we can move left within current display line
        if current_display_pos.col > 0 {
            // Use character-aware left movement
            if let Some(current_line) = self.display_cache.get_display_line(current_display_pos.row)
            {
                let new_col = current_line.move_left_by_character(current_display_pos.col);
                let new_display_pos = Position::new(current_display_pos.row, new_col);
                self.display_cursor = new_display_pos;
                // Update virtual column for horizontal movement
                self.update_virtual_column();
                moved = true;
            }
        } else if current_display_pos.row > 0 {
            // VISUAL BLOCK FIX: In Visual Block mode, prevent moving to previous line
            if self.editor_mode != EditorMode::VisualBlock {
                // Move to end of previous display line
                let prev_display_line = current_display_pos.row - 1;
                if let Some(prev_line) = self.display_cache.get_display_line(prev_display_line) {
                    // Use display width instead of character count for proper multibyte character support
                    let new_col = prev_line.display_width().saturating_sub(1);
                    let new_display_pos = Position::new(prev_display_line, new_col);
                    self.display_cursor = new_display_pos;
                    // Update virtual column for horizontal movement
                    self.update_virtual_column();
                    moved = true;
                }
            }
        }

        if moved {
            // Sync logical cursor with new display position
            let new_display_pos = self.display_cursor;
            if let Some(logical_pos) = self
                .display_cache
                .display_to_logical_position(new_display_pos.row, new_display_pos.col)
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

    /// Move cursor right with capability checking and visual selection support
    pub fn move_cursor_right(&mut self, content_width: usize) -> Vec<ViewEvent> {
        // Check if navigation is allowed on this pane
        if !self.capabilities.contains(PaneCapabilities::NAVIGABLE) {
            return vec![]; // Navigation not allowed on this pane
        }

        let current_display_pos = self.display_cursor;
        let mut moved = false;

        // Check if cursor can move right within current line
        let can_move_right_in_line = if let Some(current_line) =
            self.display_cache.get_display_line(current_display_pos.row)
        {
            let line_display_width = current_line.display_width();

            match self.editor_mode {
                EditorMode::Insert => {
                    // Insert mode: Allow cursor to go one position past end of line
                    current_display_pos.col < line_display_width
                }
                EditorMode::VisualBlock => {
                    // Visual Block mode: Allow cursor to move beyond line content
                    true // Always allow right movement in Visual Block mode
                }
                _ => {
                    // Normal/Visual mode: Stop at last character position
                    if line_display_width == 0 {
                        false // Empty line - no movement allowed
                    } else {
                        let next_pos =
                            current_line.move_right_by_character(current_display_pos.col);
                        next_pos < line_display_width
                    }
                }
            }
        } else {
            false
        };

        // Check if cursor can move to next line
        let can_move_to_next_line = if !can_move_right_in_line {
            // VISUAL BLOCK FIX: In Visual Block mode, prevent moving to next line
            if self.editor_mode == EditorMode::VisualBlock {
                false
            } else {
                let next_display_line = current_display_pos.row + 1;
                self.display_cache
                    .get_display_line(next_display_line)
                    .is_some()
            }
        } else {
            false
        };

        // Perform the actual cursor movement
        if can_move_right_in_line {
            // Move right within current line
            if let Some(current_line) = self.display_cache.get_display_line(current_display_pos.row)
            {
                let new_col = current_line.move_right_by_character(current_display_pos.col);
                self.display_cursor = Position::new(current_display_pos.row, new_col);
                self.update_virtual_column();
                moved = true;
            }
        } else if can_move_to_next_line {
            // Move to beginning of next line
            let next_display_line = current_display_pos.row + 1;
            self.display_cursor = Position::new(next_display_line, 0);
            self.update_virtual_column();
            moved = true;
        }

        if moved {
            // Sync logical cursor with new display position
            let new_display_pos = self.display_cursor;
            if let Some(logical_pos) = self
                .display_cache
                .display_to_logical_position(new_display_pos.row, new_display_pos.col)
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

    /// Move cursor up with capability checking and virtual column support
    pub fn move_cursor_up(&mut self, content_width: usize) -> Vec<ViewEvent> {
        // Check if navigation is allowed on this pane
        if !self.capabilities.contains(PaneCapabilities::NAVIGABLE) {
            return vec![]; // Navigation not allowed on this pane
        }

        let current_display_pos = self.display_cursor;

        if current_display_pos.row > 0 {
            let new_line = current_display_pos.row - 1;

            // Vim-style virtual column: try to restore the desired column position
            let virtual_col = self.virtual_column;
            let new_col = if let Some(display_line) = self.display_cache.get_display_line(new_line)
            {
                let line_char_count = display_line.char_count();
                let max_col = if self.editor_mode == EditorMode::Insert {
                    line_char_count // Insert mode: can be positioned after last character
                } else {
                    line_char_count.saturating_sub(1) // Normal/Visual: stop at last character
                };
                let clamped_col = virtual_col.min(max_col);
                // Snap to character boundary to handle DBCS characters
                display_line.snap_to_character_boundary(clamped_col)
            } else {
                virtual_col
            };

            let new_display_pos = Position::new(new_line, new_col);
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

    /// Move cursor down with capability checking and virtual column support
    pub fn move_cursor_down(&mut self, content_width: usize) -> Vec<ViewEvent> {
        // Check if navigation is allowed on this pane
        if !self.capabilities.contains(PaneCapabilities::NAVIGABLE) {
            return vec![]; // Navigation not allowed on this pane
        }

        let current_display_pos = self.display_cursor;
        let next_display_line = current_display_pos.row + 1;

        // Check if the next display line actually exists
        if let Some(display_line) = self.display_cache.get_display_line(next_display_line) {
            // Vim-style virtual column: try to restore the desired column position
            let virtual_col = self.virtual_column;
            let line_char_count = display_line.char_count();
            let max_col = if self.editor_mode == EditorMode::Insert {
                line_char_count // Insert mode: can be positioned after last character
            } else {
                line_char_count.saturating_sub(1) // Normal/Visual: stop at last character
            };
            let clamped_col = virtual_col.min(max_col);
            // Snap to character boundary to handle DBCS characters
            let new_col = display_line.snap_to_character_boundary(clamped_col);
            let new_display_pos = Position::new(next_display_line, new_col);

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

    // ========================================
    // Line-based Cursor Movement Methods
    // ========================================

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

        // If there are display lines, move to the end of the last line
        if let Some(last_line) = self.display_cache.get_display_line(last_line_idx) {
            let line_display_width = last_line.display_width();

            // Position at the last character (not after it) for Normal/Visual mode
            let end_col = if line_display_width > 0 {
                match self.editor_mode {
                    EditorMode::Insert => line_display_width, // Insert mode: can be after last character
                    _ => line_display_width.saturating_sub(1), // Normal/Visual: ON the last character
                }
            } else {
                0 // Empty line, stay at column 0
            };

            let end_position = Position::new(last_line_idx, end_col);
            let _result = self.set_display_cursor(end_position);

            // Update visual selection if active
            let new_cursor_pos = self.buffer.cursor();
            self.update_visual_selection_on_cursor_move(new_cursor_pos);

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
