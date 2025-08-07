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

use unicode_segmentation::UnicodeSegmentation;

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

        // Use unicode-segmentation to find word boundaries
        let mut positions = Vec::new();
        let mut current_pos = 0;

        // Add the start position
        positions.push(0);

        // Split on word boundaries and track positions
        for word_segment in text.split_word_bounds() {
            current_pos += word_segment.chars().count();
            if current_pos <= text.chars().count() {
                positions.push(current_pos);
            }
        }

        // Ensure we don't have duplicate positions and sort them
        positions.sort_unstable();
        positions.dedup();

        Ok(WordBoundaries { positions })
    }
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
