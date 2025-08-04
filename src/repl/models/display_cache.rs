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
    /// Create a new DisplayLine from content string (for backward compatibility)
    pub fn from_content(
        content: &str,
        logical_line: usize,
        logical_start_col: usize,
        logical_end_col: usize,
        is_continuation: bool,
    ) -> Self {
        use crate::repl::models::buffer_char::BufferLine;
        use crate::repl::models::display_char::DisplayChar;

        // Convert content to BufferLine first
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
    pub fn move_left_by_character(&self, current_display_col: usize) -> usize {
        if current_display_col == 0 {
            return 0;
        }

        let mut current_display_pos = 0;
        let mut prev_display_pos = 0;

        for display_char in &self.chars {
            let char_width = display_char.display_width();

            // If we've reached or passed the current position, return the previous position
            if current_display_pos >= current_display_col {
                return prev_display_pos;
            }

            prev_display_pos = current_display_pos;
            current_display_pos += char_width;
        }

        // If we're past the end, return the last valid position
        prev_display_pos
    }

    /// Move right by one character from the given display column position
    pub fn move_right_by_character(&self, current_display_col: usize) -> usize {
        let mut current_display_pos = 0;

        for display_char in &self.chars {
            let char_width = display_char.display_width();

            // Check if we're within this character's display range
            if current_display_pos <= current_display_col
                && current_display_col < current_display_pos + char_width
            {
                // Move to the next character
                return current_display_pos + char_width;
            }
            current_display_pos += char_width;
        }

        // If we're past the end, return current position
        current_display_col
    }

    /// Find the next word boundary from the current display column position
    pub fn find_next_word_boundary(&self, current_display_col: usize) -> Option<usize> {
        use crate::repl::models::buffer_char::CharacterType;

        // Build character positions array
        let mut char_positions = Vec::new();
        let mut current_pos = 0;

        for display_char in &self.chars {
            char_positions.push((current_pos, display_char));
            current_pos += display_char.display_width();
        }

        if char_positions.is_empty() {
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

        // Vim 'w' behavior: move to start of next word
        // 1. If we're in a word, skip to end of current word
        // 2. Skip any whitespace/punctuation
        // 3. Stop at beginning of next word

        let mut i = current_index;

        // If we're at the last character, no next word
        if i >= char_positions.len() {
            return None;
        }

        // Skip current character position to avoid staying in place
        if i < char_positions.len() - 1 {
            i += 1;
        }

        // Skip to end of current word if we're in one
        let current_type = char_positions[current_index].1.buffer_char.character_type();
        if current_type == CharacterType::Word || current_type == CharacterType::DoubleByteChar {
            while i < char_positions.len() {
                let char_type = char_positions[i].1.buffer_char.character_type();
                // Stop if we hit a different character type (including transition between Word and DoubleByteChar)
                if char_type != current_type {
                    // Now we're at the end of current word, break to find next word
                    break;
                }
                i += 1;
            }
        }

        // Skip whitespace and punctuation to find next word
        while i < char_positions.len() {
            let char_type = char_positions[i].1.buffer_char.character_type();
            if char_type == CharacterType::Word || char_type == CharacterType::DoubleByteChar {
                // Found start of next word
                return Some(char_positions[i].0);
            }
            i += 1;
        }

        None
    }

    /// Find the previous word boundary from the current display column position  
    pub fn find_previous_word_boundary(&self, current_display_col: usize) -> Option<usize> {
        use crate::repl::models::buffer_char::CharacterType;

        if current_display_col == 0 {
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

        // Vim 'b' behavior: move to beginning of current or previous word
        // 1. If we're in a word, move to beginning of current word
        // 2. If we're not in a word, skip backwards through non-word chars to find previous word

        // Start from the character just before current position
        let mut i = if current_index > 0 {
            current_index - 1
        } else {
            0
        };

        // Skip backwards through current non-word characters (whitespace/punctuation)
        while i > 0 {
            let char_type = char_positions[i].1.buffer_char.character_type();
            if char_type == CharacterType::Word || char_type == CharacterType::DoubleByteChar {
                break;
            }
            i -= 1;
        }

        // If we found a word character, find the beginning of this word
        let char_type = char_positions[i].1.buffer_char.character_type();
        if char_type == CharacterType::Word || char_type == CharacterType::DoubleByteChar {
            // Move backwards to find beginning of current word (same character type)
            while i > 0 {
                let prev_char_type = char_positions[i - 1].1.buffer_char.character_type();
                // Stop if we hit a different character type (including transition between Word and DoubleByteChar)
                if prev_char_type != char_type {
                    break;
                }
                i -= 1;
            }
            return Some(char_positions[i].0);
        }

        // If we're at position 0 and it's not a word character, return None
        if i == 0 {
            let char_type = char_positions[0].1.buffer_char.character_type();
            if char_type == CharacterType::Word || char_type == CharacterType::DoubleByteChar {
                return Some(0);
            }
            return None;
        }

        None
    }

    /// Find the end of the current or next word from the current display column position
    pub fn find_end_of_word(&self, current_display_col: usize) -> Option<usize> {
        use crate::repl::models::buffer_char::CharacterType;

        let mut current_display_pos = 0;

        // First, determine if we're currently at the end of a word
        let mut at_word_end = false;
        for display_char in &self.chars {
            let char_width = display_char.display_width();
            if current_display_pos == current_display_col {
                let char_type = display_char.buffer_char.character_type();
                let is_word =
                    char_type == CharacterType::Word || char_type == CharacterType::DoubleByteChar;

                // Check if next character is non-word (indicating we're at word end)
                if is_word {
                    let next_pos = current_display_pos + char_width;
                    let mut next_display_pos = 0;
                    for next_char in &self.chars {
                        let next_char_width = next_char.display_width();
                        if next_display_pos == next_pos {
                            let next_char_type = next_char.buffer_char.character_type();
                            let next_is_word = next_char_type == CharacterType::Word
                                || next_char_type == CharacterType::DoubleByteChar;
                            at_word_end = !next_is_word;
                            break;
                        }
                        next_display_pos += next_char_width;
                    }
                    // If we're at the last character, we're at word end
                    if next_pos >= self.chars.len() {
                        at_word_end = true;
                    }
                }
                break;
            }
            current_display_pos += char_width;
        }

        // Now find the end of the next word
        current_display_pos = 0;
        let mut found_word_start = false;
        let mut skipping_current_word = at_word_end;

        for display_char in &self.chars {
            let char_width = display_char.display_width();

            // Skip to current position
            if current_display_pos < current_display_col {
                current_display_pos += char_width;
                continue;
            }

            let char_type = display_char.buffer_char.character_type();
            let is_word =
                char_type == CharacterType::Word || char_type == CharacterType::DoubleByteChar;

            // If we're at word end, skip past current word and any whitespace
            if skipping_current_word {
                if !is_word {
                    skipping_current_word = false;
                }
                current_display_pos += char_width;
                continue;
            }

            if is_word {
                found_word_start = true;
            } else if found_word_start {
                // Found end of word, return position of last character in word
                return Some(current_display_pos.saturating_sub(1));
            }

            current_display_pos += char_width;
        }

        // If we found a word but reached end of line, return last position
        if found_word_start {
            return Some(current_display_pos.saturating_sub(1));
        }

        None
    }

    /// Convert display column to logical character index within this display line
    pub fn display_col_to_logical_index(&self, display_col: usize) -> usize {
        let mut current_display_col = 0;

        for display_char in &self.chars {
            let char_width = display_char.display_width();

            // Check if the display column falls within this character
            if current_display_col <= display_col && display_col < current_display_col + char_width
            {
                return display_char.buffer_char.logical_index;
            }
            current_display_col += char_width;
        }

        // If past the end, return the character count
        self.chars.len()
    }

    /// Convert logical character index to display column within this display line
    pub fn logical_index_to_display_col(&self, logical_index: usize) -> usize {
        let mut display_col = 0;

        for display_char in &self.chars {
            if display_char.buffer_char.logical_index >= logical_index {
                return display_col;
            }
            display_col += display_char.display_width();
        }

        // If logical_index is at or past the end, return total display width
        display_col
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

        for &display_idx in display_indices {
            if let Some(display_line) = self.display_lines.get(display_idx) {
                tracing::debug!(
                    "logical_to_display_position: checking display_idx={}, logical_col={}, start_col={}, end_col={}",
                    display_idx, logical_col, display_line.logical_start_col, display_line.logical_end_col
                );

                // Check if cursor falls within this display line segment
                if logical_col >= display_line.logical_start_col
                    && logical_col < display_line.logical_end_col
                {
                    let display_col = logical_col - display_line.logical_start_col;
                    tracing::debug!(
                        "logical_to_display_position: found match in range, returning ({}, {})",
                        display_idx,
                        display_col
                    );
                    return Some(Position::new(display_idx, display_col));
                }
                // BUGFIX: Handle end-of-line case - when logical_col equals logical_end_col,
                // it should map to the beginning of the NEXT display line (column 0),
                // not the end of the current display line. This fixes the boundary condition
                // where cursor gets stuck at column 1 on wrapped continuation lines.
                if logical_col == display_line.logical_end_col
                    && display_line.logical_end_col > display_line.logical_start_col
                {
                    // Check if there's a next display line for this logical line
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

                    // If no next continuation line, position at end of current line (fallback)
                    let display_col = display_line.logical_end_col - display_line.logical_start_col;
                    tracing::debug!(
                        "logical_to_display_position: found end of logical line, returning ({}, {})",
                        display_idx, display_col
                    );
                    return Some(Position::new(display_idx, display_col));
                }
            }
        }

        // Fallback to last display line of this logical line
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

        // BUGFIX: Clamp display column to valid cursor positions to prevent horizontal scrolling issues
        // For a line with N characters, valid cursor positions are 0 to N-1 (on each character).
        // Position N would be after the last character and can trigger unwanted horizontal scrolling.
        // When moving across wrapped line segments, display_col might exceed valid positions.
        let content_length = display_info.char_count();
        let clamped_display_col = display_col.min(content_length);
        let logical_col = display_info.logical_start_col + clamped_display_col;

        tracing::debug!(
            "display_to_logical_position: display=({}, {}) -> logical=({}, {}) [content_len={}, start_col={}, clamped_col={}]",
            display_line, display_col, logical_line, logical_col,
            content_length, display_info.logical_start_col, clamped_display_col
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

        // Try to maintain column position, but clamp to line length
        let target_col = desired_col.min(target_display_info.char_count());

        Some(Position::new(target_display_line, target_col))
    }

    /// Move cursor down by one display line
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

        // Try to maintain column position, but clamp to line length
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
        let display_line =
            DisplayLine::from_content(mixed_text, 0, 0, mixed_text.chars().count(), false);

        // From start (position 0), 'w' should go to "Borat"
        let next_word = display_line.find_next_word_boundary(0);
        assert!(next_word.is_some());
        let borat_start = next_word.unwrap();
        assert_eq!(borat_start, 11, "Should jump to 'B' in 'Borat'");

        // From inside "Borat", 'w' should go to "です"
        let after_borat = display_line.find_next_word_boundary(borat_start + 1);
        assert!(after_borat.is_some(), "Should find 'です' after 'Borat'");
        let desu_start = after_borat.unwrap();
        assert_eq!(desu_start, 17, "Should jump to 'で' in 'です'");

        // From "です", 'b' should go back to "Borat"
        let back_to_borat = display_line.find_previous_word_boundary(desu_start);
        assert!(back_to_borat.is_some());
        assert_eq!(
            back_to_borat.unwrap(),
            borat_start,
            "Should go back to 'Borat'"
        );
    }
}
