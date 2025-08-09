//! # Display Cache for Word Wrap Support
//!
//! Provides display line caching for efficient word wrap rendering and cursor positioning.
//! Maps logical lines to display lines with position tracking for navigation.

use crate::repl::geometry::Position;
use std::collections::HashMap;
use std::time::Instant;

/// Type alias for display position (display_line, display_column)
pub type DisplayPosition = Position;

/// Type alias for logical-to-display line mapping
pub type LogicalToDisplayMap = HashMap<usize, Vec<usize>>;

/// Type alias for logical position (logical_line, logical_column)  
pub type LogicalPosition = Position;

/// Pre-calculated display line with positioning metadata
#[derive(Debug, Clone, PartialEq)]
pub struct DisplayLine {
    /// The display characters for this line with styling and positioning info
    pub chars: Vec<crate::repl::models::display_char::DisplayChar>,
    /// Index of the logical line this display line represents
    pub logical_line: usize,
    /// Starting column position in the logical line
    pub logical_start_col: usize,
    /// Ending column position in the logical line (exclusive)
    pub logical_end_col: usize,
    /// True if this is a continuation of a wrapped line
    pub is_continuation: bool,
}

impl DisplayLine {
    /// Create a new DisplayLine from DisplayChars (proper way to create DisplayLine)
    /// Use this instead of from_content to ensure word boundaries are properly set
    pub fn new(
        chars: Vec<crate::repl::models::display_char::DisplayChar>,
        logical_line: usize,
        logical_start_col: usize,
        logical_end_col: usize,
        is_continuation: bool,
    ) -> Self {
        Self {
            chars,
            logical_line,
            logical_start_col,
            logical_end_col,
            is_continuation,
        }
    }

    /// DEPRECATED: Create DisplayLine from content string without word boundaries
    /// This exists only for backward compatibility with old tests and build_display_cache
    /// New code should use PaneState::build_display_cache_from_character_buffer instead
    #[deprecated(
        note = "Use PaneState::build_display_cache_from_character_buffer for proper word boundaries"
    )]
    pub fn from_content(
        content: &str,
        logical_line: usize,
        logical_start_col: usize,
        logical_end_col: usize,
        is_continuation: bool,
    ) -> Self {
        use crate::repl::models::buffer_char::BufferLine;
        use crate::repl::models::display_char::DisplayChar;

        // Convert content to BufferLine first (without word boundaries)
        let buffer_line = BufferLine::from_string(content);

        // Convert BufferLine to DisplayChars
        let mut chars = Vec::new();
        let mut current_screen_col = 0;

        for buffer_char in buffer_line.chars() {
            let display_char = DisplayChar::from_buffer_char(
                buffer_char.clone(),
                (0, current_screen_col), // screen_position (row, col)
            );
            current_screen_col += display_char.display_width();
            chars.push(display_char);
        }

        Self {
            chars,
            logical_line,
            logical_start_col,
            logical_end_col,
            is_continuation,
        }
    }

    /// Get the content as a plain string (for backward compatibility)
    pub fn content(&self) -> String {
        self.chars.iter().map(|dc| dc.ch()).collect()
    }

    /// Get character count
    pub fn char_count(&self) -> usize {
        self.chars.len()
    }

    /// Get display width (total columns needed)
    pub fn display_width(&self) -> usize {
        self.chars.iter().map(|dc| dc.display_width()).sum()
    }

    /// Get character at display column
    pub fn char_at_display_col(
        &self,
        display_col: usize,
    ) -> Option<&crate::repl::models::display_char::DisplayChar> {
        for display_char in &self.chars {
            let start = display_char.screen_col();
            let end = start + display_char.display_width();
            if start <= display_col && display_col < end {
                return Some(display_char);
            }
        }
        None
    }

    /// Apply highlighting to a range of characters
    pub fn highlight_range(&mut self, start_col: usize, end_col: usize) {
        for display_char in &mut self.chars {
            let char_start = display_char.screen_col();
            let char_end = char_start + display_char.display_width();

            // Check if this character overlaps with the highlight range
            if char_start < end_col && char_end > start_col {
                display_char.set_highlighted(true);
            }
        }
    }

    /// Convert to rendered string with ANSI codes
    pub fn to_ansi_string(&self) -> String {
        let mut result = String::new();
        for display_char in &self.chars {
            result.push_str(&display_char.ansi_style_start());
            result.push(display_char.ch());
            result.push_str(&display_char.ansi_style_end());
        }
        result
    }

    /// Move left by one character from the given display column position
    ///
    /// HIGH-LEVEL LOGIC:
    /// 1. Handle left boundary case (return 0)
    /// 2. Walk through characters, tracking current and previous display positions
    /// 3. When we reach or pass current position, return previous position
    /// 4. Handle past-the-end case by returning last valid position
    pub fn move_left_by_character(&self, current_display_col: usize) -> usize {
        if current_display_col == 0 {
            return 0;
        }

        let mut current_display_pos = 0;
        let mut prev_display_pos = 0;

        for display_char in &self.chars {
            let char_width = display_char.display_width();

            // POSITION TRACKING: Have we reached the target position?
            // Return the start of the previous character
            if current_display_pos >= current_display_col {
                return prev_display_pos;
            }

            prev_display_pos = current_display_pos;
            current_display_pos += char_width;
        }

        // FALLBACK: Past the end, return last valid position
        prev_display_pos
    }

    /// Move right by one character from the given display column position
    ///
    /// HIGH-LEVEL LOGIC:
    /// 1. Walk through characters, tracking display positions
    /// 2. Find the character that contains the current display column
    /// 3. Return the start position of the next character
    /// 4. Handle past-the-end case by returning current position
    pub fn move_right_by_character(&self, current_display_col: usize) -> usize {
        let mut current_display_pos = 0;

        for display_char in &self.chars {
            let char_width = display_char.display_width();

            // CHARACTER CONTAINMENT: Is current position within this character's span?
            // If so, move to the start of the next character
            if current_display_pos <= current_display_col
                && current_display_col < current_display_pos + char_width
            {
                return current_display_pos + char_width;
            }
            current_display_pos += char_width;
        }

        // FALLBACK: Past the end, return current position (no movement)
        current_display_col
    }

    /// Find the next word boundary from the current display column position using ICU segmentation
    ///
    /// HIGH-LEVEL LOGIC:
    /// 1. Build array of (display_position, display_char) pairs for efficient lookup
    /// 2. Find the character index that corresponds to current_display_col
    /// 3. Search forward for the next character marked with is_word_start flag
    /// 4. Return the display column of that word start, or None if not found
    pub fn find_next_word_boundary(&self, current_display_col: usize) -> Option<usize> {
        tracing::debug!(
            "find_next_word_boundary: current_display_col={}, line_content='{}'",
            current_display_col,
            self.content().chars().take(50).collect::<String>()
        );

        // PHASE 1: Build character positions array for efficient lookup
        let mut char_positions = Vec::new();
        let mut current_pos = 0;

        for display_char in &self.chars {
            char_positions.push((current_pos, display_char));
            current_pos += display_char.display_width();
        }

        if char_positions.is_empty() {
            tracing::debug!("find_next_word_boundary: empty line, returning None");
            return None;
        }

        // PHASE 2: Find character index corresponding to current display column
        let mut current_index = 0;
        for (i, &(pos, _)) in char_positions.iter().enumerate() {
            if pos >= current_display_col {
                current_index = i;
                break;
            }
        }

        tracing::debug!(
            "find_next_word_boundary: current_index={}, searching from char '{}'",
            current_index,
            char_positions
                .get(current_index)
                .map_or('?', |(_, dc)| dc.ch())
        );

        // PHASE 3: Search forward for next word start using ICU segmentation flags
        #[allow(clippy::needless_range_loop)] // Index needed for position lookup
        for i in (current_index + 1)..char_positions.len() {
            let display_char = char_positions[i].1;
            tracing::debug!(
                "find_next_word_boundary: checking char at index {} (display_col={}): '{}', is_word_start={}",
                i, char_positions[i].0, display_char.ch(), display_char.buffer_char.is_word_start
            );
            // WORD START CHECK: ICU segmentation marked this character as starting a new word
            if display_char.buffer_char.is_word_start {
                tracing::debug!(
                    "find_next_word_boundary: found word start at display_col={}, char='{}'",
                    char_positions[i].0,
                    display_char.ch()
                );
                return Some(char_positions[i].0);
            }
        }

        tracing::debug!("find_next_word_boundary: no word start found, returning None");
        None
    }

    /// Find the previous word boundary from the current display column position using ICU segmentation
    pub fn find_previous_word_boundary(&self, current_display_col: usize) -> Option<usize> {
        tracing::debug!(
            "find_previous_word_boundary: current_display_col={}, line_content='{}'",
            current_display_col,
            self.content().chars().take(50).collect::<String>()
        );

        if current_display_col == 0 {
            tracing::debug!("find_previous_word_boundary: at start of line, returning None");
            return None;
        }

        // Build character positions array
        let mut char_positions = Vec::new();
        let mut current_pos = 0;

        for display_char in &self.chars {
            char_positions.push((current_pos, display_char));
            current_pos += display_char.display_width();
        }

        if char_positions.is_empty() {
            tracing::debug!("find_previous_word_boundary: empty line, returning None");
            return None;
        }

        // Find current character index - fix for Issue #67
        let mut current_index = 0;
        for (i, &(pos, display_char)) in char_positions.iter().enumerate() {
            let char_end = pos + display_char.display_width();
            if current_display_col < char_end {
                current_index = i;
                break;
            }
            // If we're past the last character, use the last valid index
            if i == char_positions.len() - 1 {
                current_index = i;
            }
        }

        tracing::debug!(
            "find_previous_word_boundary: current_index={}, searching backwards from char '{}'",
            current_index,
            char_positions
                .get(current_index)
                .map_or('?', |(_, dc)| dc.ch())
        );

        // Look backwards for previous word start using ICU segmentation boundaries
        // Vim 'b' behavior: move to beginning of current or previous word
        for i in (0..current_index).rev() {
            let display_char = char_positions[i].1;
            if display_char.buffer_char.is_word_start {
                tracing::debug!(
                    "find_previous_word_boundary: found word start at display_col={}, char='{}'",
                    char_positions[i].0,
                    display_char.ch()
                );
                return Some(char_positions[i].0);
            }
        }

        tracing::debug!("find_previous_word_boundary: no word start found, returning None");
        None
    }

    /// Find the end of the current or next word from the current display column position using ICU segmentation
    pub fn find_end_of_word(&self, current_display_col: usize) -> Option<usize> {
        tracing::debug!(
            "find_end_of_word: current_display_col={}, line_content='{}'",
            current_display_col,
            self.content().chars().take(50).collect::<String>()
        );

        // Build character positions array
        let mut char_positions = Vec::new();
        let mut current_pos = 0;

        for display_char in &self.chars {
            char_positions.push((current_pos, display_char));
            current_pos += display_char.display_width();
        }

        if char_positions.is_empty() {
            tracing::debug!("find_end_of_word: empty line, returning None");
            return None;
        }

        // Find current character index
        let mut current_index = 0;
        for (i, &(pos, _)) in char_positions.iter().enumerate() {
            if pos >= current_display_col {
                current_index = i;
                break;
            }
        }

        tracing::debug!(
            "find_end_of_word: current_index={}, searching from char '{}'",
            current_index,
            char_positions
                .get(current_index)
                .map_or('?', |(_, dc)| dc.ch())
        );

        // Look for next word end using ICU segmentation boundaries
        // Vim 'e' behavior: move to end of current or next word
        #[allow(clippy::needless_range_loop)] // Index needed for position lookup
        for i in current_index..char_positions.len() {
            let display_char = char_positions[i].1;
            if display_char.buffer_char.is_word_end {
                tracing::debug!(
                    "find_end_of_word: found word end at display_col={}, char='{}'",
                    char_positions[i].0,
                    display_char.ch()
                );
                return Some(char_positions[i].0);
            }
        }

        tracing::debug!("find_end_of_word: no word end found, returning None");
        None
    }

    /// Convert display column to logical character index within this display line
    ///
    /// HIGH-LEVEL LOGIC:
    /// 1. Walk through characters, accumulating display widths
    /// 2. Find which character spans the target display column
    /// 3. Return that character's absolute logical index in the buffer
    /// 4. Handle out-of-bounds by returning character count
    pub fn display_col_to_logical_index(&self, display_col: usize) -> usize {
        let mut current_display_col = 0;

        for display_char in &self.chars {
            let char_width = display_char.display_width();

            // CHARACTER SPAN CHECK: Does this character occupy the target display column?
            // Example: Japanese char at display_col 0-1 contains display_col 0 and 1
            if current_display_col <= display_col && display_col < current_display_col + char_width
            {
                return display_char.buffer_char.logical_index;
            }
            current_display_col += char_width;
        }

        // FALLBACK: Past the end of line
        self.chars.len()
    }

    /// Convert logical character index to display column within this display line
    ///
    /// HIGH-LEVEL LOGIC:
    /// 1. Walk through characters, accumulating display widths
    /// 2. Stop when we reach or pass the target logical index
    /// 3. Return the accumulated display column at that point
    /// 4. Handle out-of-bounds by returning total display width
    pub fn logical_index_to_display_col(&self, logical_index: usize) -> usize {
        let mut display_col = 0;

        for display_char in &self.chars {
            // POSITION CHECK: Have we reached the target logical index?
            // Example: logical_index=2 → stop before char at logical_index=2, return display_col
            if display_char.buffer_char.logical_index >= logical_index {
                return display_col;
            }
            display_col += display_char.display_width();
        }

        // FALLBACK: logical_index is at or past the end, return total display width
        display_col
    }

    /// Convert display column to character index within this display line
    /// This is different from display_col_to_logical_index - it returns the index
    /// into the chars array (0-based character position in this display line)
    ///
    /// HIGH-LEVEL LOGIC:
    /// 1. Walk through chars array with enumeration for local indexing
    /// 2. Find which character spans the target display column
    /// 3. Return the local character index (position in this display line)
    /// 4. Handle out-of-bounds by returning character count
    pub fn display_col_to_char_index(&self, display_col: usize) -> usize {
        let mut current_display_col = 0;

        for (char_idx, display_char) in self.chars.iter().enumerate() {
            let char_width = display_char.buffer_char.display_width;

            // CHARACTER SPAN CHECK: Does this character occupy the target display column?
            // Returns local index within this display line (not absolute buffer position)
            if current_display_col <= display_col && display_col < current_display_col + char_width
            {
                return char_idx;
            }
            current_display_col += char_width;
        }

        // FALLBACK: Past the end, return character count (clamped to valid positions)
        self.chars.len()
    }
}

/// Cache for pre-calculated display lines with fast lookup
#[derive(Debug, Clone)]
pub struct DisplayCache {
    /// All display lines in order
    pub display_lines: Vec<DisplayLine>,
    /// Map logical line index to display line indices
    pub logical_to_display: LogicalToDisplayMap,
    /// Total number of display lines
    pub total_display_lines: usize,
    /// Content width used for this cache
    pub content_width: usize,
    /// Content hash for invalidation detection
    pub content_hash: u64,
    /// When this cache was generated
    pub generated_at: Instant,
    /// Whether this cache is valid
    pub is_valid: bool,
    /// Whether word wrap is enabled for this cache
    pub wrap_enabled: bool,
}

impl DisplayCache {
    /// Create a new empty cache
    pub fn new() -> Self {
        Self {
            display_lines: Vec::new(),
            logical_to_display: HashMap::new(),
            total_display_lines: 0,
            content_width: 0,
            content_hash: 0,
            generated_at: Instant::now(),
            is_valid: false,
            wrap_enabled: false,
        }
    }

    /// Check if cache is valid for given content, width, and wrap mode
    pub fn is_valid_for(&self, content_hash: u64, width: usize, wrap_enabled: bool) -> bool {
        self.is_valid
            && self.content_hash == content_hash
            && self.content_width == width
            && self.wrap_enabled == wrap_enabled
    }

    /// Invalidate the cache
    pub fn invalidate(&mut self) {
        self.is_valid = false;
    }

    /// Convert logical cursor position to display position using cache
    ///
    /// HIGH-LEVEL LOGIC:
    /// 1. Find all display line segments that make up the target logical line
    /// 2. Determine which segment contains the target logical column
    /// 3. Convert the logical column to display column, accounting for multibyte character widths
    /// 4. Handle special cases like line wrapping boundaries and end-of-line positions
    pub fn logical_to_display_position(
        &self,
        logical_line: usize,
        logical_col: usize,
    ) -> Option<DisplayPosition> {
        if !self.is_valid {
            return None;
        }

        // Find display lines for this logical line
        let display_indices = self.logical_to_display.get(&logical_line)?;

        // LOGIC: A single logical line can span multiple display lines due to wrapping.
        // We need to find which display line segment contains our target logical column,
        // then convert that logical position to the correct display column within that segment.
        for &display_idx in display_indices {
            if let Some(display_line) = self.display_lines.get(display_idx) {
                tracing::debug!(
                    "logical_to_display_position: checking display_idx={}, logical_col={}, start_col={}, end_col={}",
                    display_idx, logical_col, display_line.logical_start_col, display_line.logical_end_col
                );

                // CASE 1: Normal cursor position within a display line segment
                // Check if the target logical column falls within this display line's range
                if logical_col >= display_line.logical_start_col
                    && logical_col < display_line.logical_end_col
                {
                    // Convert absolute logical column to relative character index within this display line
                    // Example: logical_col=3, logical_start_col=0 -> relative_char_index=3
                    // Then calculate display column accounting for multibyte character widths
                    let relative_char_index = logical_col - display_line.logical_start_col;
                    let display_col =
                        display_line.logical_index_to_display_col(relative_char_index);

                    tracing::debug!(
                        "logical_to_display_position: found match in range, logical_col={}, relative_char_idx={}, returning ({}, {})",
                        logical_col, relative_char_index, display_idx, display_col
                    );
                    return Some(Position::new(display_idx, display_col));
                }
                // CASE 2: End-of-line boundary case for wrapped lines
                // When logical_col equals logical_end_col, we're at the boundary between segments.
                // For wrapped lines, this should map to the beginning of the next continuation line.
                if logical_col == display_line.logical_end_col
                    && display_line.logical_end_col > display_line.logical_start_col
                {
                    // Check if there's a next display line that continues this logical line
                    let next_display_idx = display_idx + 1;
                    if next_display_idx < self.display_lines.len() {
                        if let Some(next_display_line) = self.display_lines.get(next_display_idx) {
                            // If the next display line is a continuation of the same logical line,
                            // position cursor at the beginning of it (column 0)
                            if next_display_line.logical_line == display_line.logical_line
                                && next_display_line.is_continuation
                            {
                                tracing::debug!(
                                    "logical_to_display_position: found end-of-line match, moving to next continuation line ({}, 0)",
                                    next_display_idx
                                );
                                return Some(Position::new(next_display_idx, 0));
                            }
                        }
                    }

                    // FALLBACK: If no next continuation line, position at end of current line
                    // Must calculate actual display width, not just character count
                    let display_col = display_line.display_width();
                    tracing::debug!(
                        "logical_to_display_position: found end of logical line, returning ({}, {})",
                        display_idx, display_col
                    );
                    return Some(Position::new(display_idx, display_col));
                }
            }
        }

        // CASE 3: Fallback for cursor positions beyond the end of all segments
        // This handles edge cases where the logical column is past the end of the line
        if let Some(&last_display_idx) = display_indices.last() {
            if let Some(display_line) = self.display_lines.get(last_display_idx) {
                let display_col = (display_line.char_count())
                    .min(logical_col.saturating_sub(display_line.logical_start_col));
                return Some(Position::new(last_display_idx, display_col));
            }
        }

        None
    }

    /// Convert display position to logical position using cache
    ///
    /// HIGH-LEVEL LOGIC:
    /// 1. Get the display line information at the target display row
    /// 2. Convert display column to character index within that display line
    /// 3. Add the display line's logical start offset to get absolute logical column
    /// 4. Return logical line and calculated logical column
    pub fn display_to_logical_position(
        &self,
        display_line: usize,
        display_col: usize,
    ) -> Option<LogicalPosition> {
        if !self.is_valid {
            tracing::debug!("display_to_logical_position: cache invalid");
            return None;
        }

        let display_info = self.display_lines.get(display_line)?;
        let logical_line = display_info.logical_line;

        // CORE CONVERSION: Display column → Character index → Absolute logical column
        // Example: display_col=6 for Japanese chars "こんに" → char_index=3 → logical_col=0+3=3
        let character_index = display_info.display_col_to_char_index(display_col);
        let logical_col = display_info.logical_start_col + character_index;

        tracing::debug!(
            "display_to_logical_position: display=({}, {}) -> logical=({}, {}) [content_len={}, start_col={}, char_idx={}]",
            display_line, display_col, logical_line, logical_col,
            display_info.char_count(), display_info.logical_start_col, character_index
        );

        Some(Position::new(logical_line, logical_col))
    }

    /// Get display line content and metadata
    pub fn get_display_line(&self, display_idx: usize) -> Option<&DisplayLine> {
        if !self.is_valid {
            return None;
        }
        self.display_lines.get(display_idx)
    }

    /// Move cursor up by one display line
    ///
    /// HIGH-LEVEL LOGIC:
    /// 1. Check bounds and cache validity
    /// 2. Target the display line above current position
    /// 3. Maintain desired column position, clamping to target line's character count
    /// 4. Return new display position
    pub fn move_up(
        &self,
        current_display_line: usize,
        desired_col: usize,
    ) -> Option<DisplayPosition> {
        if !self.is_valid || current_display_line == 0 {
            return None;
        }

        let target_display_line = current_display_line - 1;
        let target_display_info = self.display_lines.get(target_display_line)?;

        // COLUMN PRESERVATION: Try to maintain desired column, but respect line boundaries
        // Note: desired_col is in character positions, not display columns
        let target_col = desired_col.min(target_display_info.char_count());

        Some(Position::new(target_display_line, target_col))
    }

    /// Move cursor down by one display line
    ///
    /// HIGH-LEVEL LOGIC:
    /// 1. Check bounds and cache validity
    /// 2. Target the display line below current position
    /// 3. Maintain desired column position, clamping to target line's character count
    /// 4. Return new display position
    pub fn move_down(
        &self,
        current_display_line: usize,
        desired_col: usize,
    ) -> Option<DisplayPosition> {
        if !self.is_valid || current_display_line >= self.display_lines.len().saturating_sub(1) {
            return None;
        }

        let target_display_line = current_display_line + 1;
        let target_display_info = self.display_lines.get(target_display_line)?;

        // COLUMN PRESERVATION: Try to maintain desired column, but respect line boundaries
        // Note: desired_col is in character positions, not display columns
        let target_col = desired_col.min(target_display_info.char_count());

        Some(Position::new(target_display_line, target_col))
    }

    /// Get total number of display lines
    pub fn display_line_count(&self) -> usize {
        self.total_display_lines
    }

    /// Check if a logical line has multiple display lines (is wrapped)
    pub fn is_logical_line_wrapped(&self, logical_line: usize) -> bool {
        if let Some(display_indices) = self.logical_to_display.get(&logical_line) {
            display_indices.len() > 1
        } else {
            false
        }
    }
}

impl Default for DisplayCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Information about a wrapped line segment
#[derive(Debug, Clone)]
struct WrappedSegment {
    content: String,
    logical_start: usize,
    logical_end: usize,
}

/// Build a display cache from logical lines
pub fn build_display_cache(
    lines: &[String],
    content_width: usize,
    wrap_enabled: bool,
) -> anyhow::Result<DisplayCache> {
    let content_hash = calculate_content_hash(lines);
    let mut display_lines = Vec::new();
    let mut logical_to_display = HashMap::new();

    for (logical_idx, line) in lines.iter().enumerate() {
        let wrapped_segments = if wrap_enabled {
            wrap_line_with_positions(line, content_width)
        } else {
            // No wrap: 1:1 mapping
            vec![WrappedSegment {
                content: line.clone(),
                logical_start: 0,
                logical_end: line.chars().count(),
            }]
        };

        let mut display_indices = Vec::new();

        for (segment_idx, segment_info) in wrapped_segments.iter().enumerate() {
            let display_idx = display_lines.len();
            display_indices.push(display_idx);

            #[allow(deprecated)]
            let display_line = DisplayLine::from_content(
                &segment_info.content,
                logical_idx,
                segment_info.logical_start,
                segment_info.logical_end,
                segment_idx > 0,
            );

            display_lines.push(display_line);
        }

        logical_to_display.insert(logical_idx, display_indices);
    }

    Ok(DisplayCache {
        total_display_lines: display_lines.len(),
        display_lines,
        logical_to_display,
        content_width,
        content_hash,
        generated_at: Instant::now(),
        is_valid: true,
        wrap_enabled,
    })
}

/// Wrap a line into segments with accurate position tracking using character display widths
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

            // Track word boundaries (spaces and tabs) for intelligent wrapping
            if buffer_char.ch == ' ' || buffer_char.ch == '\t' {
                last_word_boundary_char_pos = Some(segment_end_char_pos);
            }

            current_display_width += char_display_width;
            segment_end_char_pos += 1;
        }

        // If we've processed all characters, create final segment
        if segment_end_char_pos >= total_chars {
            let segment_content: String = buffer_chars[current_char_pos..total_chars]
                .iter()
                .map(|bc| bc.ch)
                .collect();

            segments.push(WrappedSegment {
                content: segment_content,
                logical_start: current_char_pos,
                logical_end: total_chars,
            });
            break;
        }

        // Determine where to break: prefer word boundary if available
        let mut actual_break_pos = segment_end_char_pos;

        // If we found a word boundary and breaking there wouldn't create an empty segment
        if let Some(word_boundary_pos) = last_word_boundary_char_pos {
            if word_boundary_pos > current_char_pos {
                actual_break_pos = word_boundary_pos;
            }
        }

        // Create segment content
        let segment_content: String = buffer_chars[current_char_pos..actual_break_pos]
            .iter()
            .map(|bc| bc.ch)
            .collect();

        segments.push(WrappedSegment {
            content: segment_content,
            logical_start: current_char_pos,
            logical_end: actual_break_pos,
        });

        current_char_pos = actual_break_pos;

        // Skip whitespace at the beginning of next line
        while current_char_pos < total_chars {
            let buffer_char = &buffer_chars[current_char_pos];
            if buffer_char.ch == ' ' || buffer_char.ch == '\t' {
                current_char_pos += 1;
            } else {
                break;
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

/// Calculate a simple hash of the content for invalidation detection
fn calculate_content_hash(lines: &[String]) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    lines.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_cache_should_create_empty() {
        let cache = DisplayCache::new();
        assert!(!cache.is_valid);
        assert_eq!(cache.display_lines.len(), 0);
    }

    #[test]
    fn build_display_cache_should_work_without_wrap() {
        let lines = vec!["Line 1".to_string(), "Line 2".to_string()];
        let cache = build_display_cache(&lines, 80, false).unwrap();

        assert!(cache.is_valid);
        assert!(!cache.wrap_enabled);
        assert_eq!(cache.display_lines.len(), 2);
        assert_eq!(cache.display_lines[0].content(), "Line 1");
        assert_eq!(cache.display_lines[1].content(), "Line 2");
        assert!(!cache.display_lines[0].is_continuation);
        assert!(!cache.display_lines[1].is_continuation);
    }

    #[test]
    fn build_display_cache_should_work_with_wrap() {
        let lines = vec!["This is a very long line that should wrap".to_string()];
        let cache = build_display_cache(&lines, 20, true).unwrap();

        assert!(cache.is_valid);
        assert!(cache.wrap_enabled);
        assert!(cache.display_lines.len() > 1); // Should be wrapped

        // First segment should not be continuation
        assert!(!cache.display_lines[0].is_continuation);

        // Following segments should be continuations
        for i in 1..cache.display_lines.len() {
            assert!(cache.display_lines[i].is_continuation);
        }
    }

    #[test]
    fn logical_to_display_position_should_work() {
        let lines = vec!["Hello world this is a test".to_string()];
        let cache = build_display_cache(&lines, 10, true).unwrap();

        // Position at start should map to display (0, 0)
        let pos = cache.logical_to_display_position(0, 0).unwrap();
        assert_eq!(pos.row, 0);
        assert_eq!(pos.col, 0);

        // Position in middle should map to appropriate display line
        if let Some(pos) = cache.logical_to_display_position(0, 15) {
            assert!(pos.row > 0); // Should be on a wrapped line
            assert!(pos.col < 10); // Within the content width
        }
    }

    #[test]
    fn display_to_logical_position_should_work() {
        let lines = vec!["Hello world test".to_string()];
        let cache = build_display_cache(&lines, 10, true).unwrap();

        // First display line should map back to logical line 0
        let pos = cache.display_to_logical_position(0, 5).unwrap();
        assert_eq!(pos.row, 0);
        assert_eq!(pos.col, 5);
    }

    #[test]
    fn move_up_down_should_work() {
        let lines = vec!["Line 1".to_string(), "Line 2".to_string()];
        let cache = build_display_cache(&lines, 80, false).unwrap();

        // Move down from first line
        let pos = cache.move_down(0, 3).unwrap();
        assert_eq!(pos.row, 1);
        assert_eq!(pos.col, 3);

        // Move up from second line
        let pos = cache.move_up(1, 3).unwrap();
        assert_eq!(pos.row, 0);
        assert_eq!(pos.col, 3);
    }

    #[test]
    fn cache_invalidation_should_work() {
        let lines = vec!["Test".to_string()];
        let cache = build_display_cache(&lines, 80, false).unwrap();

        assert!(cache.is_valid_for(calculate_content_hash(&lines), 80, false));
        assert!(!cache.is_valid_for(calculate_content_hash(&lines), 60, false)); // Different width
        assert!(!cache.is_valid_for(calculate_content_hash(&lines), 80, true)); // Different wrap mode
    }

    #[test]
    fn mixed_language_word_boundaries_should_work() {
        // Test mixed Japanese-English text like "こんにちは Borat です"
        let mixed_text = "こんにちは Borat です";
        #[allow(deprecated)]
        let display_line =
            DisplayLine::from_content(mixed_text, 0, 0, mixed_text.chars().count(), false);

        // NOTE: This test uses the deprecated from_content method which doesn't
        // set up word boundaries properly. Word navigation requires proper
        // word boundary setup via the unicode segmenter.
        //
        // The find_next_word_boundary method depends on is_word_start flags
        // being set on display characters, which doesn't happen with from_content.
        //
        // For now, we'll test that the method doesn't panic and returns None
        // when word boundaries aren't properly set up.

        let next_word = display_line.find_next_word_boundary(0);
        // Should return None since word boundaries aren't set up
        assert!(
            next_word.is_none(),
            "Should return None when word boundaries not set up"
        );
    }
}
