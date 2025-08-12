//! # Word Segmentation for International Text
//!
//! This module provides word boundary detection using unicode-segmentation crate,
//! supporting proper international text handling (CJK, Arabic, Thai, etc.).
//!
//! ## Key Features
//!
//! - Unicode Standard Annex #29 compliant word segmentation
//! - Lightweight pure-Rust implementation
//! - Optimized for caching at the logical buffer level
//! - Clean abstraction allowing future segmentation backend changes

use cjk;

/// Word boundary flags for a character position
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct WordFlags {
    /// True if this character position starts a word
    pub is_word_start: bool,
    /// True if this character position ends a word  
    pub is_word_end: bool,
}

impl WordFlags {
    /// Create new word flags
    pub fn new(is_word_start: bool, is_word_end: bool) -> Self {
        Self {
            is_word_start,
            is_word_end,
        }
    }
}

/// Type alias for segmentation results
pub type SegmentationResult = Result<WordBoundaries, Box<dyn std::error::Error>>;

/// Type alias for segmenter creation results
pub type SegmenterResult = Result<UnicodeWordSegmenter, Box<dyn std::error::Error>>;

/// Word boundary information for a segment of text
#[derive(Debug, Clone, PartialEq)]
pub struct WordBoundaries {
    /// All boundary positions (character indices) in the text
    pub positions: Vec<usize>,
}

impl WordBoundaries {
    /// Convert byte-based boundary positions to word flags for each character
    ///
    /// # Arguments
    /// * `text` - The original text to analyze
    ///
    /// # Returns
    /// A vector of WordFlags with one entry per character (not byte)
    pub fn to_word_flags(&self, text: &str) -> Vec<WordFlags> {
        let char_count = text.chars().count();
        let mut flags = vec![WordFlags::default(); char_count];

        // Build a mapping from byte position to character index
        let mut byte_to_char = vec![0; text.len() + 1];
        let mut char_index = 0;
        let mut byte_pos = 0;

        for ch in text.chars() {
            // All bytes for this character map to this character index
            for b in 0..ch.len_utf8() {
                if byte_pos + b < byte_to_char.len() {
                    byte_to_char[byte_pos + b] = char_index;
                }
            }
            byte_pos += ch.len_utf8();
            char_index += 1;
        }

        // Handle end position
        if byte_pos < byte_to_char.len() {
            byte_to_char[byte_pos] = char_index;
        }

        // Mark boundary positions as word starts/ends
        for i in 1..self.positions.len() {
            let byte_pos = self.positions[i];

            // Mark as word start (except for position 0 and end)
            if byte_pos > 0 && byte_pos <= text.len() {
                let char_pos = if byte_pos < byte_to_char.len() {
                    byte_to_char[byte_pos]
                } else {
                    char_count
                };

                if char_pos < flags.len() {
                    flags[char_pos].is_word_start = true;
                }
            }

            // Mark previous character as word end
            if byte_pos > 0 {
                let prev_byte_pos = self.positions[i - 1];
                let prev_char_pos = if prev_byte_pos < byte_to_char.len() {
                    byte_to_char[prev_byte_pos]
                } else {
                    char_count.saturating_sub(1)
                };

                // Find the last character of the previous word
                if prev_char_pos < flags.len() {
                    // Look forward to find the actual end character
                    let mut end_char_pos = prev_char_pos;
                    while end_char_pos < char_count && end_char_pos + 1 < char_count {
                        // Check if the next character starts at our boundary
                        let next_char_byte_start = text
                            .chars()
                            .take(end_char_pos + 1)
                            .map(|c| c.len_utf8())
                            .sum::<usize>();
                        if next_char_byte_start >= byte_pos {
                            break;
                        }
                        end_char_pos += 1;
                    }
                    if end_char_pos < flags.len() {
                        flags[end_char_pos].is_word_end = true;
                    }
                }
            }
        }

        flags
    }
}

/// Trait for word segmentation implementations
pub trait WordSegmenter: Send {
    /// Find word boundaries in the given text
    fn find_word_boundaries(&self, text: &str) -> SegmentationResult;
}

/// Unicode-segmentation based word segmenter implementation
pub struct UnicodeWordSegmenter;

impl UnicodeWordSegmenter {
    /// Create a new unicode-segmentation word segmenter
    pub fn new() -> SegmenterResult {
        Ok(Self)
    }
}

impl WordSegmenter for UnicodeWordSegmenter {
    fn find_word_boundaries(&self, text: &str) -> SegmentationResult {
        if text.is_empty() {
            return Ok(WordBoundaries { positions: vec![0] });
        }

        // Use vim-like word boundary detection with byte-based indices
        let chars: Vec<char> = text.chars().collect();
        let mut positions = Vec::new();

        // Add the start position (byte 0)
        positions.push(0);

        let mut byte_pos = 0;

        // Process each character, tracking transitions between different types
        let mut prev_type: Option<CharType> = None;

        for ch in chars.iter() {
            let current_type = get_char_type(*ch);

            // Add boundary at transitions between different meaningful character types
            match (prev_type, current_type) {
                // Transitions that require boundaries
                (Some(CharType::Word), CharType::Cjk) => positions.push(byte_pos),
                (Some(CharType::Cjk), CharType::Word) => positions.push(byte_pos),
                (Some(CharType::Cjk), CharType::Cjk) => positions.push(byte_pos), // Each CJK is separate
                (Some(CharType::Other), CharType::Word) => positions.push(byte_pos),
                (Some(CharType::Other), CharType::Cjk) => positions.push(byte_pos),
                (Some(CharType::Word), CharType::Other) => {} // Don't add boundary after word
                (Some(CharType::Cjk), CharType::Other) => {}  // Don't add boundary after CJK
                _ => {}                                       // No boundary needed
            }

            byte_pos += ch.len_utf8();

            // Always update prev_type to properly track transitions
            prev_type = Some(current_type);
        }

        // Note: unicode-segmentation does not include end position as a boundary
        // End position is not a "word start" - only include actual word boundaries

        // Ensure positions are sorted and unique
        positions.sort_unstable();
        positions.dedup();

        Ok(WordBoundaries { positions })
    }
}

/// Character types for boundary detection
#[derive(Debug, Clone, Copy, PartialEq)]
enum CharType {
    Word,  // ASCII alphanumeric + underscore
    Cjk,   // CJK characters (each is a separate word)
    Other, // Whitespace, punctuation, symbols
}

/// Get the character type for boundary detection
fn get_char_type(ch: char) -> CharType {
    // Check CJK first since CJK chars also return true for is_alphanumeric()
    if is_cjk_char(ch) {
        CharType::Cjk
    } else if is_word_char(ch) {
        CharType::Word
    } else {
        CharType::Other
    }
}

/// Check if a character is a word character (alphanumeric + underscore)
fn is_word_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
}

/// Check if a character is from CJK (Chinese, Japanese, Korean) scripts
fn is_cjk_char(ch: char) -> bool {
    cjk::is_cjk_codepoint(ch)
}

/// Factory for creating word segmenters
pub struct WordSegmenterFactory;

impl WordSegmenterFactory {
    /// Create the best available word segmenter
    pub fn create() -> Box<dyn WordSegmenter> {
        match UnicodeWordSegmenter::new() {
            Ok(segmenter) => Box::new(segmenter),
            Err(e) => {
                tracing::error!("Failed to create unicode segmenter: {}", e);
                panic!("No word segmenter available: {e}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unicode_segmenter_should_create() {
        let segmenter = UnicodeWordSegmenter::new();
        assert!(
            segmenter.is_ok(),
            "Unicode segmenter should be created successfully"
        );
    }

    #[test]
    fn word_boundaries_should_work_with_simple_text() {
        let segmenter = UnicodeWordSegmenter::new().unwrap();
        let boundaries = segmenter.find_word_boundaries("Hello World").unwrap();

        // Debug: Print what we got
        println!("Text: 'Hello World'");
        println!("Boundaries: {:?}", boundaries.positions);

        // Unicode-segmentation should give us word boundaries
        assert!(!boundaries.positions.is_empty());
        assert_eq!(boundaries.positions[0], 0); // Start of text

        // Should have word start boundaries (no end position)
        // Our vim-like segmentation provides word start boundaries only
        assert!(boundaries.positions.contains(&6)); // Should have boundary at "World"

        // Test that we can convert to flags without panicking
        let flags = boundaries.to_word_flags("Hello World");
        assert_eq!(flags.len(), 11);

        // Debug: Print the flags
        for (i, flag) in flags.iter().enumerate() {
            if flag.is_word_start || flag.is_word_end {
                println!("Position {i}: {flag:?}");
            }
        }

        // At least some positions should be marked as word boundaries
        let has_word_starts = flags.iter().any(|flag| flag.is_word_start);
        let has_word_ends = flags.iter().any(|flag| flag.is_word_end);

        println!("Has word starts: {has_word_starts}, Has word ends: {has_word_ends}");
    }

    #[test]
    fn word_boundaries_should_handle_empty_text() {
        let segmenter = UnicodeWordSegmenter::new().unwrap();
        let boundaries = segmenter.find_word_boundaries("").unwrap();

        assert_eq!(boundaries.positions, vec![0]);

        let flags = boundaries.to_word_flags("");
        assert!(flags.is_empty());
    }

    #[test]
    fn word_boundaries_should_handle_mixed_text() {
        let segmenter = UnicodeWordSegmenter::new().unwrap();
        let boundaries = segmenter
            .find_word_boundaries("hello こんにちは world")
            .unwrap();

        // Should detect boundaries properly for mixed Latin/CJK text
        assert!(!boundaries.positions.is_empty());

        let flags = boundaries.to_word_flags("hello こんにちは world");

        // Should have word starts and ends
        let has_word_starts = flags.iter().any(|flag| flag.is_word_start);
        let has_word_ends = flags.iter().any(|flag| flag.is_word_end);

        assert!(has_word_starts, "Should detect word starts");
        assert!(has_word_ends, "Should detect word ends");
    }

    #[test]
    fn factory_should_create_segmenter() {
        let segmenter = WordSegmenterFactory::create();
        let boundaries = segmenter.find_word_boundaries("test").unwrap();

        assert!(!boundaries.positions.is_empty());
    }

    #[test]
    fn test_byte_based_boundaries() {
        let segmenter = UnicodeWordSegmenter::new().unwrap();

        // Test with multibyte characters
        let text = "hello こんにちは world";
        let boundaries = segmenter.find_word_boundaries(text).unwrap();

        // Expected: [0, 6, 9, 12, 15, 18, 22] (byte positions without end)
        // 0: start of "hello"
        // 6: start of "こ" (first CJK character)
        // 9: start of "ん" (each CJK character is a separate word)
        // 12: start of "に"
        // 15: start of "ち"
        // 18: start of "は"
        // 22: start of "world"
        // Note: Each CJK character is treated as a separate word for vim navigation

        assert_eq!(boundaries.positions, vec![0, 6, 9, 12, 15, 18, 22]);
        assert_eq!(text.len(), 27); // Verify total byte count

        // Test that positions point to valid character boundaries
        for &pos in &boundaries.positions {
            if pos < text.len() {
                // Should not panic when slicing at these byte positions
                let _ = &text[pos..];
            }
        }
    }

    #[test]
    fn test_multibyte_single_byte_transitions() {
        let segmenter = UnicodeWordSegmenter::new().unwrap();

        // Test cases where multibyte and single-byte characters are adjacent
        let test_cases = vec![
            ("helloこんにちは", "ASCII word adjacent to CJK"),
            ("こんにちはworld", "CJK adjacent to ASCII word"),
            ("testこんにちはworld", "ASCII-CJK-ASCII transitions"),
            ("こa", "Single CJK + single ASCII"),
            ("aこ", "Single ASCII + single CJK"),
            (
                "test test test 日本語テスト",
                "Multiple words with spaces before CJK",
            ),
            ("こんにちは Borat です", "CJK-ASCII-CJK mixed text"),
        ];

        for (text, description) in test_cases {
            println!("\n=== {description} ===");
            println!("Text: '{text}'");

            let boundaries = segmenter.find_word_boundaries(text).unwrap();
            println!("Boundaries: {:?}", boundaries.positions);

            // Verify boundaries point to valid character starts
            for &pos in &boundaries.positions {
                if pos < text.len() {
                    let slice = &text[pos..];
                    let first_char = slice.chars().next().unwrap();
                    println!("  Byte {pos} -> '{first_char}'");
                }
            }
        }
    }
}
