//! Display line representation for word-wrapped text rendering
//!
//! Represents a single display line that may be part of a wrapped logical line.
//! Handles character positioning, word navigation, and display width calculations.

use crate::repl::models::buffer_char::BufferLine;
use crate::repl::models::display_char::DisplayChar;

/// Type alias for character position entry in display line
type CharPosition<'a> = (usize, &'a DisplayChar);

/// Pre-calculated display line with positioning metadata
#[derive(Debug, Clone, PartialEq)]
pub struct DisplayLine {
    /// The display characters for this line with styling and positioning info
    pub chars: Vec<DisplayChar>,
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
        chars: Vec<DisplayChar>,
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

    /// Get the display width of the line in columns
    pub fn display_width(&self) -> usize {
        self.chars.iter().map(|dc| dc.display_width()).sum()
    }

    /// Get the number of characters in the line
    pub fn char_count(&self) -> usize {
        self.chars.len()
    }

    /// Get the logical character index range for this display line
    pub fn logical_char_range(&self) -> (usize, usize) {
        (self.logical_start_col, self.logical_end_col)
    }

    /// Get display column position for a logical character index within this display line
    pub fn logical_index_to_display_col(&self, logical_index: usize) -> usize {
        // The logical_index is relative to this display line segment
        // (0 means first char of this segment, not absolute position in buffer)
        let mut display_col = 0;

        for (char_index, display_char) in self.chars.iter().enumerate() {
            // Check if we've reached the target character index
            if char_index >= logical_index {
                return display_col;
            }
            display_col += display_char.display_width();
        }

        // FALLBACK: Past the end, return the total display width
        self.display_width()
    }

    /// Build a character positions array mapping display columns to characters
    fn build_character_positions(&self) -> Vec<CharPosition> {
        let mut char_positions = Vec::new();
        let mut current_pos = 0;
        for display_char in &self.chars {
            char_positions.push((current_pos, display_char));
            current_pos += display_char.display_width();
        }
        char_positions
    }

    /// Find the character index corresponding to a display column
    fn find_character_index(&self, char_positions: &[CharPosition], display_col: usize) -> usize {
        let mut current_index = 0;
        for (i, &(pos, dc)) in char_positions.iter().enumerate() {
            let char_end = pos + dc.display_width();
            // Check if this character contains the display column
            if pos <= display_col && display_col < char_end {
                current_index = i;
                break;
            }
            // If we're past the last character, use the last index
            if i == char_positions.len() - 1 {
                current_index = i;
            }
        }
        current_index
    }

    /// Find the next word start from the current display column position using ICU segmentation
    ///
    /// HIGH-LEVEL LOGIC:
    /// 1. Build array of (display_position, display_char) pairs for efficient lookup
    /// 2. Find the character index that corresponds to current_display_col
    /// 3. Search forward for the next character marked with is_word_start flag
    /// 4. Return the display column of that word start, or None if not found
    pub fn find_next_word_start(&self, current_display_col: usize) -> Option<usize> {
        tracing::debug!(
            "find_next_word_start: current_display_col={}, line_content='{}'",
            current_display_col,
            self.content().chars().take(50).collect::<String>()
        );

        // Build character positions array for efficient lookup
        let char_positions = self.build_character_positions();
        if char_positions.is_empty() {
            tracing::debug!("find_next_word_start: empty line, returning None");
            return None;
        }

        // Find character index corresponding to current display column
        // Note: For forward search, we use simpler logic to find starting position
        let mut current_index = 0;
        for (i, &(pos, _)) in char_positions.iter().enumerate() {
            if pos >= current_display_col {
                current_index = i;
                break;
            }
        }

        tracing::debug!(
            "find_next_word_start: current_index={}, searching from char '{}'",
            current_index,
            char_positions
                .get(current_index)
                .map_or('?', |(_, dc)| dc.ch())
        );

        // Search forward for next word start using ICU segmentation flags
        #[allow(clippy::needless_range_loop)] // Index needed for position lookup
        for i in (current_index + 1)..char_positions.len() {
            let display_char = char_positions[i].1;
            tracing::debug!(
                "find_next_word_start: checking char at index {} (display_col={}): '{}', is_word_start={}",
                i, char_positions[i].0, display_char.ch(), display_char.buffer_char.is_word_start
            );
            // WORD START CHECK: ICU segmentation marked this character as starting a new word
            if display_char.buffer_char.is_word_start {
                tracing::debug!(
                    "find_next_word_start: found word start at display_col={}, char='{}'",
                    char_positions[i].0,
                    display_char.ch()
                );
                return Some(char_positions[i].0);
            }
        }

        tracing::debug!("find_next_word_start: no word start found, returning None");
        None
    }

    /// Find the previous word start from the current display column position using ICU segmentation
    pub fn find_previous_word_start(&self, current_display_col: usize) -> Option<usize> {
        tracing::debug!(
            "find_previous_word_start: current_display_col={}, line_content='{}'",
            current_display_col,
            self.content().chars().take(50).collect::<String>()
        );

        // Don't return early at position 0 - we might need to check if position 0 is a word start
        // when moving from position 1 back to position 0

        // Build character positions array
        let char_positions = self.build_character_positions();
        if char_positions.is_empty() {
            tracing::debug!("find_previous_word_start: empty line, returning None");
            return None;
        }

        // Find current character index - needs special logic for backward search
        let current_index = self.find_character_index(&char_positions, current_display_col);

        tracing::debug!(
            "find_previous_word_start: current_index={}, searching backwards from char '{}'",
            current_index,
            char_positions
                .get(current_index)
                .map_or('?', |(_, dc)| dc.ch())
        );

        // Look backwards for previous word start using ICU segmentation boundaries
        // Vim 'b' behavior: move to beginning of current or previous word
        // Fix: Include all positions from current_index-1 down to 0 to reach first character
        if current_index > 0 {
            for i in (0..current_index).rev() {
                let display_char = char_positions[i].1;
                if display_char.buffer_char.is_word_start {
                    // Skip whitespace-only word starts - we want actual word starts
                    let ch = display_char.ch();
                    if !ch.is_whitespace() {
                        tracing::debug!(
                            "find_previous_word_start: found word start at display_col={}, char='{}'",
                            char_positions[i].0,
                            display_char.ch()
                        );
                        return Some(char_positions[i].0);
                    }
                }
            }
        }

        // Special case: if position 0 is not marked as word_start but contains a non-whitespace character,
        // it should be considered a valid word start (for lines that start with words)
        if !char_positions.is_empty() {
            let first_char = char_positions[0].1;
            if !first_char.ch().is_whitespace() && current_display_col > 0 {
                tracing::debug!(
                    "find_previous_word_start: falling back to position 0 as word start, char='{}'",
                    first_char.ch()
                );
                return Some(0);
            }
        }

        tracing::debug!("find_previous_word_start: no word start found, returning None");
        None
    }

    /// Find the next word end from the current display column position using ICU segmentation
    pub fn find_next_word_end(&self, current_display_col: usize) -> Option<usize> {
        tracing::debug!(
            "find_next_word_end: current_display_col={}, line_content='{}'",
            current_display_col,
            self.content().chars().take(50).collect::<String>()
        );

        // Build character positions array
        let char_positions = self.build_character_positions();
        if char_positions.is_empty() {
            tracing::debug!("find_next_word_end: empty line, returning None");
            return None;
        }

        // Find current character index - need to find which character contains the display column
        let current_index = self.find_character_index(&char_positions, current_display_col);

        tracing::debug!(
            "find_next_word_end: current_index={}, searching from char '{}'",
            current_index,
            char_positions
                .get(current_index)
                .map_or('?', |(_, dc)| dc.ch())
        );

        // Look for next word end using ICU segmentation boundaries
        // Vim 'e' behavior: move to end of current or next word
        // If we're already at a word end, skip to the next word end
        let mut start_index = current_index;

        // Check if we're already at a word end position
        if current_index < char_positions.len() {
            let current_char = char_positions[current_index].1;
            if current_char.buffer_char.is_word_end {
                // We're at a word end, so we need to find the next word end
                tracing::debug!(
                    "find_next_word_end: currently at word end '{}', searching for next word end",
                    current_char.ch()
                );
                start_index = current_index + 1;
            }
        }

        #[allow(clippy::needless_range_loop)] // Index needed for position lookup
        for i in start_index..char_positions.len() {
            let display_char = char_positions[i].1;
            if display_char.buffer_char.is_word_end {
                // Skip whitespace/punctuation-only word ends - we want actual word ends
                let ch = display_char.ch();
                if ch.is_alphanumeric() || ch.is_alphabetic() {
                    tracing::debug!(
                        "find_next_word_end: found word end at display_col={}, char='{}'",
                        char_positions[i].0,
                        display_char.ch()
                    );
                    return Some(char_positions[i].0);
                }
            }
        }

        tracing::debug!("find_next_word_end: no ICU word boundaries found, trying fallback");

        // FALLBACK: Implement vim 'e' behavior with character-based detection
        // Vim 'e' behavior:
        // 1. If on whitespace/punctuation: skip to next word and find its end
        // 2. If on word character: find end of current word
        // 3. If already at end of word: skip to next word and find its end

        // Make sure current_index is valid
        if current_index >= char_positions.len() {
            // We're past the end of the line
            return None;
        }

        let mut pos = current_index;

        if pos < char_positions.len() {
            let current_char = char_positions[pos].1.ch();
            if current_char.is_alphanumeric() || current_char.is_alphabetic() {
                // We're on a word character - find end of current word first
                let mut word_end_pos = pos;
                while word_end_pos < char_positions.len() {
                    let ch = char_positions[word_end_pos].1.ch();
                    if !ch.is_alphanumeric() && !ch.is_alphabetic() {
                        break;
                    }
                    word_end_pos += 1;
                }
                let current_word_end = word_end_pos.saturating_sub(1);

                // Check if we're already at the end of the current word
                if current_word_end == current_index {
                    // Already at word end - skip to next word end
                    pos = current_index + 1;
                } else {
                    // Not at word end - return current word end
                    tracing::debug!(
                        "find_next_word_end: fallback found current word end at display_col={}, char='{}'",
                        char_positions[current_word_end].0,
                        char_positions[current_word_end].1.ch()
                    );
                    return Some(char_positions[current_word_end].0);
                }
            } else {
                // We're on whitespace or punctuation - skip to next alphanumeric word
                pos = current_index + 1;
            }
        }

        // Skip non-alphanumeric characters (whitespace and punctuation) to find next word
        while pos < char_positions.len() {
            let ch = char_positions[pos].1.ch();
            if ch.is_alphanumeric() || ch.is_alphabetic() {
                break;
            }
            pos += 1;
        }

        // Find end of next alphanumeric word
        if pos < char_positions.len() {
            // We've found the start of an alphanumeric word - find its end
            while pos < char_positions.len() {
                let ch = char_positions[pos].1.ch();
                if !ch.is_alphanumeric() && !ch.is_alphabetic() {
                    break;
                }
                pos += 1;
            }
            // Return the position of the last character of the word
            if pos > 0 {
                let end_pos = pos.saturating_sub(1);
                tracing::debug!(
                    "find_next_word_end: fallback found next word end at display_col={}, char='{}'",
                    char_positions[end_pos].0,
                    char_positions[end_pos].1.ch()
                );
                return Some(char_positions[end_pos].0);
            }
        }

        tracing::debug!("find_next_word_end: no word end found even with fallback, returning None");
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
    pub fn logical_index_to_display_column(&self, logical_index: usize) -> usize {
        let mut display_col = 0;

        for display_char in &self.chars {
            // CHARACTER INDEX CHECK: Have we reached the target character?
            if display_char.buffer_char.logical_index >= logical_index {
                return display_col;
            }
            display_col += display_char.display_width();
        }

        // FALLBACK: Past the end, return total display width
        self.display_width()
    }

    /// Move left by one character, respecting multi-byte character boundaries
    ///
    /// HIGH-LEVEL LOGIC:
    /// 1. Track current and previous character positions
    /// 2. Walk through characters, accumulating display widths
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

        // FALLBACK: Past the end, return last character position
        prev_display_pos
    }

    /// Move right by one character, respecting multi-byte character boundaries
    ///
    /// HIGH-LEVEL LOGIC:
    /// 1. Walk through characters tracking display positions
    /// 2. Find the character that contains the current position
    /// 3. Return the start of the next character
    /// 4. Handle end-of-line by returning line width
    pub fn move_right_by_character(&self, current_display_col: usize) -> usize {
        let mut current_display_pos = 0;

        for display_char in &self.chars {
            let char_width = display_char.display_width();
            let next_display_pos = current_display_pos + char_width;

            // BOUNDARY CHECK: Does current position fall within this character?
            // If so, return the start of the next character
            if current_display_col < next_display_pos {
                return next_display_pos;
            }

            current_display_pos = next_display_pos;
        }

        // FALLBACK: At or past the end, return total line width
        self.display_width()
    }

    /// Snap a display column to the nearest character boundary
    ///
    /// HIGH-LEVEL LOGIC:
    /// 1. Walk through characters tracking display positions
    /// 2. Find which character contains the target column
    /// 3. Return the start position of that character
    /// 4. Handle past-the-end by returning line width
    pub fn snap_to_character_boundary(&self, display_col: usize) -> usize {
        let mut current_display_pos = 0;

        for display_char in &self.chars {
            let char_width = display_char.display_width();
            let next_display_pos = current_display_pos + char_width;

            // CONTAINMENT CHECK: Does this character contain the target column?
            // Snap to the start of this character
            if display_col < next_display_pos {
                return current_display_pos;
            }

            current_display_pos = next_display_pos;
        }

        // FALLBACK: Past the end, return total line width
        self.display_width()
    }

    /// Get the character at a specific display column
    ///
    /// HIGH-LEVEL LOGIC:
    /// 1. Walk through characters tracking display positions
    /// 2. Find which character contains the target column
    /// 3. Return that character
    /// 4. Return None if past the end
    pub fn char_at_display_col(&self, display_col: usize) -> Option<&DisplayChar> {
        let mut current_display_pos = 0;

        for display_char in &self.chars {
            let char_width = display_char.display_width();
            let next_display_pos = current_display_pos + char_width;

            // CONTAINMENT CHECK: Does this character span the target column?
            if display_col >= current_display_pos && display_col < next_display_pos {
                return Some(display_char);
            }

            current_display_pos = next_display_pos;
        }

        // FALLBACK: No character at this position
        None
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
