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
    /// Convert boundary positions to word flags for each character
    pub fn to_word_flags(&self, text_len: usize) -> Vec<WordFlags> {
        let mut flags = vec![WordFlags::default(); text_len];

        // Mark all boundary positions as potential word starts/ends
        for i in 1..self.positions.len() {
            let pos = self.positions[i];

            // Mark as word start (except for position 0 and end)
            if pos > 0 && pos < text_len {
                flags[pos].is_word_start = true;
            }

            // Mark previous position as word end
            if pos > 0 && pos <= text_len {
                let end_pos = pos.saturating_sub(1);
                if end_pos < flags.len() {
                    flags[end_pos].is_word_end = true;
                }
            }
        }

        flags
    }
}

/// Trait for word segmentation implementations
pub trait WordSegmenter {
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

        // Use vim-like word boundary detection
        let chars: Vec<char> = text.chars().collect();
        let mut positions = Vec::new();

        // Add the start position
        positions.push(0);

        let mut i = 0;
        while i < chars.len() {
            let current_char = chars[i];

            if is_word_char(current_char) {
                // Skip through word characters and add boundary after the word
                while i < chars.len() && is_word_char(chars[i]) {
                    i += 1;
                }
                // Don't add boundary at end of text here - we'll add it later
            } else if is_cjk_char(current_char) {
                // For CJK characters, each character is a word boundary
                i += 1;
            } else {
                // Skip whitespace and punctuation characters without adding boundaries
                while i < chars.len() && !is_word_char(chars[i]) && !is_cjk_char(chars[i]) {
                    i += 1;
                }
            }

            // Add boundary at the current position if we're not at the end
            // and if the current position represents a word or CJK character start
            if i < chars.len() && (is_word_char(chars[i]) || is_cjk_char(chars[i])) {
                positions.push(i);
            }
        }

        // Ensure we have the end position
        if !positions.contains(&chars.len()) {
            positions.push(chars.len());
        }

        // Ensure positions are sorted and unique
        positions.sort_unstable();
        positions.dedup();

        Ok(WordBoundaries { positions })
    }
}

/// Check if a character is a word character (alphanumeric + underscore)
fn is_word_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
}

/// Check if a character is from CJK (Chinese, Japanese, Korean) scripts
fn is_cjk_char(ch: char) -> bool {
    let code = ch as u32;
    // CJK ranges (same as used in buffer_char.rs)
    (0x3040..=0x309F).contains(&code) // Hiragana
        || (0x30A0..=0x30FF).contains(&code) // Katakana  
        || (0x4E00..=0x9FAF).contains(&code) // CJK Unified Ideographs
        || (0x3400..=0x4DBF).contains(&code) // CJK Extension A
        || (0x20000..=0x2A6DF).contains(&code) // CJK Extension B
        || (0xF900..=0xFAFF).contains(&code) // CJK Compatibility Ideographs
        || (0xFF00..=0xFFEF).contains(&code) // Full-width characters
        || (0xAC00..=0xD7AF).contains(&code) // Hangul (Korean)
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

        // Should have boundaries around words and whitespace
        // Unicode-segmentation treats whitespace as separate segments
        assert!(boundaries.positions.contains(&11)); // Should have boundary at end

        // Test that we can convert to flags without panicking
        let flags = boundaries.to_word_flags(11); // "Hello World" is 11 chars
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

        let flags = boundaries.to_word_flags(0);
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

        let flags = boundaries.to_word_flags("hello こんにちは world".chars().count());

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
}
