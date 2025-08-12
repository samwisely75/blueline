//! # Character-aware Buffer Management
//!
//! Provides a unified representation for characters that tracks both logical
//! and display properties, solving the single-byte/double-byte positioning problem.
//!
//! HIGH-LEVEL ARCHITECTURE:
//! This module implements the Unicode-aware character handling system for the text editor:
//! - BufferChar: Represents individual characters with logical positioning information
//! - CharacterBuffer: Container for sequences of BufferChar with text operations
//! - Unicode Support: Handles CJK double-byte characters, combining marks, and full-width characters
//! - Word Segmentation: Provides language-aware word boundary detection for navigation
//!
//! CORE RESPONSIBILITIES:
//! 1. Character Classification: Distinguishes between word, punctuation, whitespace, and double-byte characters
//! 2. Unicode Normalization: Ensures consistent text representation across different input methods
//! 3. Display Width Calculation: Accurately computes screen columns needed for rendering
//! 4. Word Boundary Detection: Supports Vim-style word navigation with proper international text handling
//!
//! CRITICAL DESIGN DECISIONS:
//! - Separates logical character position from display column position
//! - Uses Unicode scalar values consistently to avoid grapheme cluster issues  
//! - Implements character type classification for proper navigation behavior
//! - Provides pluggable word segmentation for different languages and scripts

use crate::text::word_segmenter::{WordBoundaries, WordSegmenter, WordSegmenterFactory};

/// Type alias for boxed word segmenter to improve readability
type BoxedWordSegmenter = Option<Box<dyn WordSegmenter + Send>>;

/// Check if a character is an ideographic character (CJK and similar scripts)
pub fn is_ideographic_character(ch: char) -> bool {
    let code = ch as u32;

    // CJK ideographs and related scripts
    // Hiragana: U+3040–U+309F
    // Katakana: U+30A0–U+30FF
    // CJK Unified Ideographs: U+4E00–U+9FAF
    // CJK Unified Ideographs Extension A: U+3400–U+4DBF
    // CJK Unified Ideographs Extension B: U+20000–U+2A6DF
    // CJK Compatibility Ideographs: U+F900–U+FAFF
    // Full-width ASCII variants: U+FF00-U+FFEF
    // Hangul (Korean): U+AC00–U+D7AF
    (0x3040..=0x309F).contains(&code) // Hiragana
        || (0x30A0..=0x30FF).contains(&code) // Katakana
        || (0x4E00..=0x9FAF).contains(&code) // CJK Unified Ideographs
        || (0x3400..=0x4DBF).contains(&code) // CJK Extension A
        || (0x20000..=0x2A6DF).contains(&code) // CJK Extension B
        || (0xF900..=0xFAFF).contains(&code) // CJK Compatibility Ideographs
        || (0xFF00..=0xFFEF).contains(&code) // Full-width characters
        || (0xAC00..=0xD7AF).contains(&code) // Hangul (Korean)
}

/// Represents different character types for navigation purposes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CharacterType {
    /// ASCII alphanumeric characters and underscore
    Word,
    /// Double-byte characters (CJK and similar scripts)
    DoubleByteChar,
    /// Punctuation and symbols
    Punctuation,
    /// Whitespace characters
    Whitespace,
}

/// Represents a single character in the buffer with logical properties only
///
/// HIGH-LEVEL DESIGN:
/// BufferChar encapsulates all information needed for a single character in the text buffer:
/// - Unicode character data with proper scalar value handling
/// - Logical positioning information for accurate cursor navigation
/// - Byte offset tracking for efficient string operations
/// - Display width calculation for proper terminal rendering
/// - Character type classification for Vim-style word navigation
#[derive(Debug, Clone, PartialEq)]
pub struct BufferChar {
    /// The actual Unicode character
    pub ch: char,
    /// Logical position in the character sequence (0-based)
    pub logical_index: usize,
    /// Byte offset of this character in the UTF-8 string
    pub byte_offset: usize,
    /// Byte length of this character in UTF-8 (1-4 bytes)
    pub byte_length: usize,
    /// Logical length (always 1 for any character)
    pub logical_length: usize,
    /// Display width in terminal columns (1 for ASCII, 2 for CJK, 0 for combining chars)
    pub display_width: usize,
    /// Whether this character is selected (for visual mode)
    pub selected: bool,
    /// Whether this character starts a word (from unicode-segmentation)
    pub is_word_start: bool,
    /// Whether this character ends a word (from unicode-segmentation)
    pub is_word_end: bool,
}

impl BufferChar {
    /// Create a new BufferChar
    pub fn new(ch: char, logical_index: usize, byte_offset: usize) -> Self {
        use unicode_width::UnicodeWidthChar;

        let byte_length = ch.len_utf8();
        let display_width = UnicodeWidthChar::width(ch).unwrap_or(1);

        Self {
            ch,
            logical_index,
            byte_offset,
            byte_length,
            logical_length: 1, // Always 1 for any character
            display_width,
            selected: false,
            is_word_start: false, // Will be set by word segmentation
            is_word_end: false,   // Will be set by word segmentation
        }
    }

    /// Check if this character is whitespace
    pub fn is_whitespace(&self) -> bool {
        self.ch.is_whitespace()
    }

    /// Check if this character is a newline
    pub fn is_newline(&self) -> bool {
        self.ch == '\n'
    }

    /// Get the character type for navigation purposes
    pub fn character_type(&self) -> CharacterType {
        if self.ch.is_whitespace() {
            CharacterType::Whitespace
        } else if is_ideographic_character(self.ch) {
            CharacterType::DoubleByteChar
        } else if self.is_word_character() {
            CharacterType::Word
        } else {
            CharacterType::Punctuation
        }
    }

    /// Check if this character is a word character (ASCII alphanumeric + underscore + Unicode letters/numbers)
    pub fn is_word_character(&self) -> bool {
        // ASCII word characters
        if self.ch.is_ascii_alphanumeric() || self.ch == '_' {
            return true;
        }

        // Unicode word characters (letters and numbers from any language)
        self.ch.is_alphabetic() || self.ch.is_numeric()
    }
}

/// A line of BufferChars with logical operations
#[derive(Debug, Clone, PartialEq)]
pub struct BufferLine {
    chars: Vec<BufferChar>,
    /// Cached word boundaries relative to this line's start
    /// None means boundaries need to be calculated
    word_boundaries_cache: Option<WordBoundaries>,
}

impl BufferLine {
    /// Create an empty BufferLine
    pub fn new() -> Self {
        Self {
            chars: Vec::new(),
            word_boundaries_cache: None,
        }
    }

    /// Create a BufferLine from a string
    pub fn from_string(text: &str) -> Self {
        let mut chars = Vec::new();
        let mut byte_offset = 0;

        for (logical_index, ch) in text.chars().enumerate() {
            let buffer_char = BufferChar::new(ch, logical_index, byte_offset);
            byte_offset += ch.len_utf8();
            chars.push(buffer_char);
        }

        Self {
            chars,
            word_boundaries_cache: None, // Will be calculated when needed
        }
    }

    /// Get all chars
    pub fn chars(&self) -> &[BufferChar] {
        &self.chars
    }

    /// Get character count (logical length)
    pub fn char_count(&self) -> usize {
        self.chars.len()
    }

    /// Get the character at a logical index
    pub fn get_char(&self, logical_index: usize) -> Option<&BufferChar> {
        self.chars.get(logical_index)
    }

    /// Insert a character at a logical position
    pub fn insert_char(&mut self, logical_index: usize, ch: char) {
        let insert_pos = logical_index.min(self.chars.len());

        // Calculate byte offset for insertion position
        let byte_offset = if insert_pos == 0 {
            0
        } else if insert_pos >= self.chars.len() {
            // Inserting at end - calculate total byte length
            self.chars.iter().map(|bc| bc.byte_length).sum()
        } else {
            // Inserting in middle - use byte offset of character at insert position
            self.chars[insert_pos].byte_offset
        };

        let new_char = BufferChar::new(ch, insert_pos, byte_offset);
        let new_char_byte_len = ch.len_utf8();

        // Insert the character
        self.chars.insert(insert_pos, new_char);

        // Update logical indices and byte offsets for characters after insertion point
        for (i, buffer_char) in self.chars.iter_mut().enumerate().skip(insert_pos + 1) {
            buffer_char.logical_index = i;
            buffer_char.byte_offset += new_char_byte_len;
        }

        // Invalidate word boundaries cache since content changed
        self.invalidate_word_boundaries_cache();
    }

    /// Delete a character at a logical position
    pub fn delete_char(&mut self, logical_index: usize) -> Option<BufferChar> {
        if logical_index >= self.chars.len() {
            return None;
        }

        let removed_char = self.chars.remove(logical_index);

        // Update logical indices for characters after deletion point
        for (i, buffer_char) in self.chars.iter_mut().enumerate().skip(logical_index) {
            buffer_char.logical_index = i;
        }

        // Invalidate word boundaries cache since content changed
        self.invalidate_word_boundaries_cache();

        Some(removed_char)
    }

    /// Convert to regular string
    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        self.chars.iter().map(|bc| bc.ch).collect()
    }

    /// Check if the line is empty
    pub fn is_empty(&self) -> bool {
        self.chars.is_empty()
    }

    /// Clamp a logical index to valid bounds
    pub fn clamp_logical_index(&self, logical_index: usize) -> usize {
        logical_index.min(self.chars.len())
    }

    /// Move left by one character from the given logical index
    pub fn move_left_by_character(&self, current_logical_index: usize) -> usize {
        current_logical_index.saturating_sub(1)
    }

    /// Move right by one character from the given logical index
    pub fn move_right_by_character(&self, current_logical_index: usize) -> usize {
        (current_logical_index + 1).min(self.chars.len())
    }

    /// Invalidate the cached word boundaries
    pub fn invalidate_word_boundaries_cache(&mut self) {
        tracing::debug!(
            "Invalidating word boundary cache for line: '{}'",
            self.to_string().chars().take(50).collect::<String>()
        );

        self.word_boundaries_cache = None;
        // Also clear word flags from all characters since they're now invalid
        for buffer_char in &mut self.chars {
            buffer_char.is_word_start = false;
            buffer_char.is_word_end = false;
        }
    }

    /// Get or calculate word boundaries for this line
    pub fn get_word_boundaries(&mut self, segmenter: &dyn WordSegmenter) -> &WordBoundaries {
        if self.word_boundaries_cache.is_none() {
            tracing::debug!(
                "Word boundaries cache miss, calculating for line: '{}'",
                self.to_string().chars().take(50).collect::<String>()
            );
            self.refresh_word_boundaries(segmenter);
        }

        // Safe to unwrap since we just ensured it exists
        self.word_boundaries_cache.as_ref().unwrap()
    }

    /// Refresh word boundaries cache for this line
    pub fn refresh_word_boundaries(&mut self, segmenter: &dyn WordSegmenter) {
        let text = self.to_string();
        if text.is_empty() {
            self.word_boundaries_cache = Some(WordBoundaries { positions: vec![0] });
            return;
        }

        match segmenter.find_word_boundaries(&text) {
            Ok(boundaries) => {
                let flags = boundaries.to_word_flags(&text);

                // Apply word flags to each character
                for (i, flag) in flags.iter().enumerate() {
                    if let Some(buffer_char) = self.chars.get_mut(i) {
                        buffer_char.is_word_start = flag.is_word_start;
                        buffer_char.is_word_end = flag.is_word_end;
                    }
                }

                self.word_boundaries_cache = Some(boundaries);
            }
            Err(e) => {
                tracing::warn!("Failed to segment line: {}", e);
                // Fall back to empty boundaries
                self.word_boundaries_cache = Some(WordBoundaries { positions: vec![0] });
            }
        }
    }

    /// Find the next word start from the current logical position
    pub fn find_next_word_start(&self, current_logical_index: usize) -> Option<usize> {
        if current_logical_index >= self.chars.len() {
            return None;
        }

        let mut pos = current_logical_index;

        // Skip current word if we're in one (vim 'w' behavior)
        if pos < self.chars.len() {
            let current_type = self.chars[pos].character_type();
            if current_type == CharacterType::Word || current_type == CharacterType::DoubleByteChar
            {
                // Skip to end of current word
                while pos < self.chars.len() {
                    let char_type = self.chars[pos].character_type();
                    if char_type != CharacterType::Word
                        && char_type != CharacterType::DoubleByteChar
                    {
                        break;
                    }
                    pos += 1;
                }
            }
        }

        // Skip whitespace and punctuation to find next word
        while pos < self.chars.len() {
            let char_type = self.chars[pos].character_type();
            if char_type == CharacterType::Word || char_type == CharacterType::DoubleByteChar {
                return Some(pos);
            }
            pos += 1;
        }

        None
    }

    /// Find the previous word start from the current logical position
    pub fn find_previous_word_start(&self, current_logical_index: usize) -> Option<usize> {
        if current_logical_index == 0 || self.chars.is_empty() {
            return None;
        }

        let mut pos = current_logical_index
            .min(self.chars.len())
            .saturating_sub(1);

        // Skip whitespace and punctuation backwards
        while pos > 0 {
            let char_type = self.chars[pos].character_type();
            if char_type == CharacterType::Word || char_type == CharacterType::DoubleByteChar {
                break;
            }
            pos = pos.saturating_sub(1);
        }

        // If we found a word character, find the beginning of the word
        if pos < self.chars.len() {
            let target_type = self.chars[pos].character_type();
            if target_type == CharacterType::Word || target_type == CharacterType::DoubleByteChar {
                while pos > 0 {
                    let prev_type = self.chars[pos.saturating_sub(1)].character_type();
                    if prev_type != CharacterType::Word
                        && prev_type != CharacterType::DoubleByteChar
                    {
                        break;
                    }
                    pos = pos.saturating_sub(1);
                }
                return Some(pos);
            }
        }

        None
    }

    /// Find the next word end from the current logical position
    pub fn find_next_word_end(&self, current_logical_index: usize) -> Option<usize> {
        if current_logical_index >= self.chars.len() {
            return None;
        }

        let mut pos = current_logical_index;

        // Skip whitespace to find next word
        while pos < self.chars.len()
            && self.chars[pos].character_type() == CharacterType::Whitespace
        {
            pos += 1;
        }

        // If we're at a word character, find its end
        if pos < self.chars.len() {
            let current_type = self.chars[pos].character_type();
            if current_type == CharacterType::Word || current_type == CharacterType::DoubleByteChar
            {
                while pos < self.chars.len() {
                    let char_type = self.chars[pos].character_type();
                    if char_type != CharacterType::Word
                        && char_type != CharacterType::DoubleByteChar
                    {
                        break;
                    }
                    pos += 1;
                }
                // Return the end of word position (last character)
                return Some(pos.saturating_sub(1));
            } else if current_type == CharacterType::Punctuation {
                // Handle punctuation sequences
                while pos < self.chars.len()
                    && self.chars[pos].character_type() == CharacterType::Punctuation
                {
                    pos += 1;
                }
                // Return the end of punctuation sequence
                return Some(pos.saturating_sub(1));
            }
        }

        None
    }
}

impl Default for BufferLine {
    fn default() -> Self {
        Self::new()
    }
}

/// A character-aware buffer that tracks both logical and display positions
///
/// HIGH-LEVEL ARCHITECTURE:
/// CharacterBuffer implements the text storage system for the REPL editor:
/// - Multi-line text storage with Unicode-aware character handling
/// - Logical position tracking for accurate cursor navigation
/// - Word boundary detection for Vim-style navigation commands
/// - Efficient insertion/deletion operations with proper indexing updates
///
/// CORE DESIGN PRINCIPLES:
/// - Maintains separation between logical character position and display column
/// - Provides O(1) character access within lines for cursor operations
/// - Caches word segmenter for performance in navigation operations  
/// - Supports international text with proper Unicode normalization
pub struct CharacterBuffer {
    lines: Vec<BufferLine>,
    /// Cached word segmenter for performance
    word_segmenter: BoxedWordSegmenter,
}

impl CharacterBuffer {
    /// Create a new empty buffer
    pub fn new() -> Self {
        Self {
            lines: vec![BufferLine::new()],
            word_segmenter: Some(WordSegmenterFactory::create()),
        }
    }

    /// Create a buffer from text lines
    pub fn from_lines(text_lines: &[String]) -> Self {
        if text_lines.is_empty() {
            return Self::new();
        }

        let lines = text_lines
            .iter()
            .map(|line| BufferLine::from_string(line))
            .collect();

        Self {
            lines,
            word_segmenter: Some(WordSegmenterFactory::create()),
        }
    }

    /// Get all lines
    pub fn lines(&self) -> &[BufferLine] {
        &self.lines
    }

    /// Get line count
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    /// Get a specific line
    pub fn get_line(&self, line_index: usize) -> Option<&BufferLine> {
        self.lines.get(line_index)
    }

    /// Get mutable access to a line
    pub fn get_line_mut(&mut self, line_index: usize) -> Option<&mut BufferLine> {
        self.lines.get_mut(line_index)
    }

    /// Insert a character at a logical position
    pub fn insert_char(&mut self, line: usize, logical_col: usize, ch: char) {
        // Ensure we have enough lines
        while self.lines.len() <= line {
            self.lines.push(BufferLine::new());
        }

        if ch == '\n' {
            // Handle newline insertion by splitting the line
            let current_line = &mut self.lines[line];
            let chars_after_cursor: Vec<BufferChar> = current_line.chars[logical_col..].to_vec();

            // Truncate current line at insertion point
            current_line.chars.truncate(logical_col);

            // Create new line with characters after cursor
            let mut new_line = BufferLine::new();
            for (i, mut buffer_char) in chars_after_cursor.into_iter().enumerate() {
                buffer_char.logical_index = i;
                new_line.chars.push(buffer_char);
            }

            self.lines.insert(line + 1, new_line);

            // Note: Word boundaries will be calculated lazily when needed
            // Both lines now have invalid caches due to content changes
        } else {
            // Regular character insertion
            self.lines[line].insert_char(logical_col, ch);

            // Note: Word boundaries cache invalidated by insert_char method
        }
    }

    /// Delete a character at a logical position
    pub fn delete_char(&mut self, line: usize, logical_col: usize) -> Option<char> {
        if line >= self.lines.len() {
            return None;
        }

        if logical_col == 0 && line > 0 {
            // Join with previous line
            let current_line = self.lines.remove(line);
            let prev_line = &mut self.lines[line - 1];

            // Append characters from current line to previous line
            let start_logical = prev_line.char_count();

            for (i, mut buffer_char) in current_line.chars.into_iter().enumerate() {
                buffer_char.logical_index = start_logical + i;
                prev_line.chars.push(buffer_char);
            }

            // Invalidate word boundaries cache for the joined line
            prev_line.invalidate_word_boundaries_cache();

            Some('\n') // Conceptually deleted a newline
        } else {
            // Delete character within line
            self.lines[line].delete_char(logical_col).map(|bc| bc.ch)
        }
    }

    /// Convert the buffer back to a Vec<String> for compatibility
    pub fn to_string_lines(&self) -> Vec<String> {
        self.lines.iter().map(|line| line.to_string()).collect()
    }

    /// Get word boundaries for a specific line (calculates if not cached)
    pub fn get_line_word_boundaries(&mut self, line_index: usize) -> Option<&WordBoundaries> {
        if let (Some(segmenter), Some(line)) =
            (&self.word_segmenter, self.lines.get_mut(line_index))
        {
            Some(line.get_word_boundaries(segmenter.as_ref()))
        } else {
            None
        }
    }

    /// Refresh word boundaries for a specific line
    pub fn refresh_line_word_boundaries(&mut self, line_index: usize) {
        if let (Some(segmenter), Some(line)) =
            (&self.word_segmenter, self.lines.get_mut(line_index))
        {
            line.refresh_word_boundaries(segmenter.as_ref());
        }
    }
}

impl Default for CharacterBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for CharacterBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CharacterBuffer")
            .field("lines", &self.lines)
            .field("word_segmenter", &"<segmenter>")
            .finish()
    }
}

impl Clone for CharacterBuffer {
    fn clone(&self) -> Self {
        Self {
            lines: self.lines.clone(),
            word_segmenter: Some(WordSegmenterFactory::create()),
        }
    }
}

impl PartialEq for CharacterBuffer {
    fn eq(&self, other: &Self) -> bool {
        // Compare only the lines, not the segmenter
        self.lines == other.lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buffer_char_should_track_logical_properties() {
        let ascii_char = BufferChar::new('a', 0, 0);
        assert_eq!(ascii_char.ch, 'a');
        assert_eq!(ascii_char.logical_index, 0);
        assert_eq!(ascii_char.byte_length, 1);
        assert_eq!(ascii_char.logical_length, 1);

        let japanese_char = BufferChar::new('こ', 1, 1);
        assert_eq!(japanese_char.ch, 'こ');
        assert_eq!(japanese_char.logical_index, 1);
        assert_eq!(japanese_char.byte_length, 3); // 'こ' is 3 bytes in UTF-8
        assert_eq!(japanese_char.logical_length, 1);
    }

    #[test]
    fn buffer_line_should_handle_mixed_text() {
        let line = BufferLine::from_string("hello こんにちは world");

        // Check character count (logical)
        assert_eq!(line.char_count(), 17); // "hello " (6) + "こんにちは" (5) + " world" (6)

        // Test basic structure
        assert_eq!(line.get_char(0).unwrap().ch, 'h');
        assert_eq!(line.get_char(6).unwrap().ch, 'こ');
        assert_eq!(line.get_char(7).unwrap().ch, 'ん');

        // Test byte lengths
        assert_eq!(line.get_char(0).unwrap().byte_length, 1); // 'h'
        assert_eq!(line.get_char(6).unwrap().byte_length, 3); // 'こ'
    }

    #[test]
    fn buffer_line_should_insert_and_delete_chars() {
        let mut line = BufferLine::from_string("hello");

        // Insert at beginning
        line.insert_char(0, 'X');
        assert_eq!(line.to_string(), "Xhello");
        assert_eq!(line.char_count(), 6);

        // Insert Japanese character in middle
        line.insert_char(3, 'こ');
        assert_eq!(line.to_string(), "Xheこllo");
        assert_eq!(line.char_count(), 7);

        // Delete the Japanese character
        let deleted = line.delete_char(3);
        assert_eq!(deleted.map(|bc| bc.ch), Some('こ'));
        assert_eq!(line.to_string(), "Xhello");
        assert_eq!(line.char_count(), 6);
    }

    #[test]
    fn character_buffer_should_handle_newlines() {
        let mut buffer = CharacterBuffer::new();

        // Insert some text
        buffer.insert_char(0, 0, 'h');
        buffer.insert_char(0, 1, 'i');

        // Insert newline
        buffer.insert_char(0, 2, '\n');

        // Insert on new line
        buffer.insert_char(1, 0, 'b');
        buffer.insert_char(1, 1, 'y');
        buffer.insert_char(1, 2, 'e');

        let string_lines = buffer.to_string_lines();
        assert_eq!(string_lines, vec!["hi", "bye"]);
        assert_eq!(buffer.line_count(), 2);
    }

    #[test]
    fn character_buffer_should_handle_backspace_line_joining() {
        let mut buffer = CharacterBuffer::from_lines(&["hello".to_string(), "world".to_string()]);

        // Delete at beginning of second line (should join lines)
        let deleted = buffer.delete_char(1, 0);
        assert_eq!(deleted, Some('\n'));

        let string_lines = buffer.to_string_lines();
        assert_eq!(string_lines, vec!["helloworld"]);
        assert_eq!(buffer.line_count(), 1);
    }

    #[test]
    fn buffer_char_should_classify_character_types() {
        let ascii_char = BufferChar::new('a', 0, 0);
        assert_eq!(ascii_char.character_type(), CharacterType::Word);

        let japanese_char = BufferChar::new('こ', 0, 0);
        assert_eq!(
            japanese_char.character_type(),
            CharacterType::DoubleByteChar
        );

        let space_char = BufferChar::new(' ', 0, 0);
        assert_eq!(space_char.character_type(), CharacterType::Whitespace);

        let punct_char = BufferChar::new('.', 0, 0);
        assert_eq!(punct_char.character_type(), CharacterType::Punctuation);
    }

    #[test]
    fn buffer_line_should_navigate_by_character() {
        let line = BufferLine::from_string("hello こんにちは world");

        // Test character-by-character movement
        assert_eq!(line.move_left_by_character(5), 4);
        assert_eq!(line.move_left_by_character(0), 0);

        assert_eq!(line.move_right_by_character(5), 6);
        assert_eq!(line.move_right_by_character(17), 17); // At end
    }

    #[test]
    fn buffer_line_should_find_word_boundaries() {
        let line = BufferLine::from_string("hello こんにちは world");

        // From start, should find "こんにちは" at position 6
        let next = line.find_next_word_start(0);
        assert_eq!(next, Some(6));

        // From Japanese word, should find "world" at position 12
        let next = line.find_next_word_start(6);
        assert_eq!(next, Some(12));

        // From "world", should find nothing
        let next = line.find_next_word_start(12);
        assert_eq!(next, None);
    }

    #[test]
    fn buffer_line_should_find_previous_word_boundaries() {
        let line = BufferLine::from_string("hello こんにちは world");

        // From end, should find "world" at position 12
        let prev = line.find_previous_word_start(17);
        assert_eq!(prev, Some(12));

        // From "world", should find "こんにちは" at position 6
        let prev = line.find_previous_word_start(12);
        assert_eq!(prev, Some(6));

        // From Japanese word, should find "hello" at position 0
        let prev = line.find_previous_word_start(6);
        assert_eq!(prev, Some(0));
    }

    #[test]
    fn buffer_line_should_find_next_word_end() {
        let line = BufferLine::from_string("hello こんにちは world");

        // From start of "hello", should find position 4 (end of "hello")
        let end = line.find_next_word_end(0);
        assert_eq!(end, Some(4));

        // From start of Japanese word, should find position 10 (end of "こんにちは")
        let end = line.find_next_word_end(6);
        assert_eq!(end, Some(10));

        // From start of "world", should find position 16 (end of "world")
        let end = line.find_next_word_end(12);
        assert_eq!(end, Some(16));
    }
}
