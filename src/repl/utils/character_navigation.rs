//! # Character-aware Navigation Utilities
//!
//! Provides utilities for handling navigation with multi-byte characters (like Japanese)
//! that need to be treated as single units for cursor movement but may occupy multiple
//! display columns.

use unicode_width::UnicodeWidthChar;

/// Represents different character types for navigation purposes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CharacterType {
    /// ASCII alphanumeric characters and underscore
    Word,
    /// Japanese hiragana, katakana, and kanji characters
    Japanese,
    /// Punctuation and symbols
    Punctuation,
    /// Whitespace characters
    Whitespace,
}

/// Move cursor left by one display character from the given display column position
/// Returns the new display column position, accounting for multi-byte characters
pub fn move_left_by_character(text: &str, current_display_col: usize) -> usize {
    if current_display_col == 0 {
        return 0;
    }

    let mut display_pos = 0;
    let mut prev_display_pos = 0;

    // Find the character that contains or is just before the current display position
    for ch in text.chars() {
        let char_width = get_char_display_width(ch);

        // If we've reached or passed the current position, return the previous position
        if display_pos >= current_display_col {
            return prev_display_pos;
        }

        prev_display_pos = display_pos;
        display_pos += char_width;
    }

    // If we're past the end, return the last valid position
    prev_display_pos
}

/// Move cursor right by one display character from the given display column position
/// Returns the new display column position, accounting for multi-byte characters
pub fn move_right_by_character(text: &str, current_display_col: usize) -> usize {
    let mut display_pos = 0;
    let mut target_char_index = None;

    // First, find which character we're currently on
    for (idx, ch) in text.chars().enumerate() {
        let char_width = get_char_display_width(ch);

        // Check if we're at or past the current display position
        if display_pos <= current_display_col && current_display_col < display_pos + char_width {
            // We're on this character, move to the next one
            target_char_index = Some(idx + 1);
            break;
        }
        display_pos += char_width;
    }

    // If we didn't find a character (cursor is at end), return current position
    if target_char_index.is_none() {
        return current_display_col;
    }

    // Now calculate the display position of the target character
    let mut new_display_pos = 0;
    for (idx, ch) in text.chars().enumerate() {
        if idx == target_char_index.unwrap() {
            return new_display_pos;
        }
        new_display_pos += get_char_display_width(ch);
    }

    // If target is beyond end, return the total display width
    new_display_pos
}

/// Determine the character type for navigation purposes
pub fn get_character_type(ch: char) -> CharacterType {
    if ch.is_whitespace() {
        CharacterType::Whitespace
    } else if ch.is_ascii_alphanumeric() || ch == '_' {
        CharacterType::Word
    } else if is_japanese_character(ch) {
        CharacterType::Japanese
    } else {
        CharacterType::Punctuation
    }
}

/// Check if a character is a Japanese character (hiragana, katakana, or kanji)
pub fn is_japanese_character(ch: char) -> bool {
    let code = ch as u32;

    // Hiragana: U+3040–U+309F
    // Katakana: U+30A0–U+30FF
    // CJK Unified Ideographs: U+4E00–U+9FAF
    // CJK Unified Ideographs Extension A: U+3400–U+4DBF
    // Full-width ASCII variants: U+FF00-U+FFEF
    (0x3040..=0x309F).contains(&code) // Hiragana
        || (0x30A0..=0x30FF).contains(&code) // Katakana
        || (0x4E00..=0x9FAF).contains(&code) // CJK Unified Ideographs
        || (0x3400..=0x4DBF).contains(&code) // CJK Extension A
        || (0xFF00..=0xFFEF).contains(&code) // Full-width characters
}

/// Find the next word boundary from the current position
/// Handles Japanese characters as word characters
pub fn find_next_word_boundary(text: &str, current_col: usize) -> Option<usize> {
    let chars: Vec<char> = text.chars().collect();
    if current_col >= chars.len() {
        return None;
    }

    let mut pos = current_col;

    // If we're currently on a word or Japanese character, skip to end of current word
    if pos < chars.len() {
        let current_type = get_character_type(chars[pos]);
        if current_type == CharacterType::Word || current_type == CharacterType::Japanese {
            while pos < chars.len() {
                let char_type = get_character_type(chars[pos]);
                if char_type != CharacterType::Word && char_type != CharacterType::Japanese {
                    break;
                }
                pos += 1;
            }
        }
    }

    // Skip whitespace and punctuation to find next word/Japanese sequence
    while pos < chars.len() {
        let char_type = get_character_type(chars[pos]);
        if char_type == CharacterType::Word || char_type == CharacterType::Japanese {
            return Some(pos);
        }
        pos += 1;
    }

    None
}

/// Find the previous word boundary from the current position
/// Handles Japanese characters as word characters
pub fn find_previous_word_boundary(text: &str, current_col: usize) -> Option<usize> {
    let chars: Vec<char> = text.chars().collect();
    if current_col == 0 || chars.is_empty() {
        return None;
    }

    let mut pos = current_col.min(chars.len()).saturating_sub(1);

    // Skip whitespace and punctuation to find previous word/Japanese sequence
    while pos > 0 {
        let char_type = get_character_type(chars[pos]);
        if char_type == CharacterType::Word || char_type == CharacterType::Japanese {
            break;
        }
        pos = pos.saturating_sub(1);
    }

    // If we found a word/Japanese character, find the beginning of it
    if pos < chars.len() {
        let target_type = get_character_type(chars[pos]);
        if target_type == CharacterType::Word || target_type == CharacterType::Japanese {
            while pos > 0 {
                let prev_type = get_character_type(chars[pos.saturating_sub(1)]);
                if prev_type != CharacterType::Word && prev_type != CharacterType::Japanese {
                    break;
                }
                pos = pos.saturating_sub(1);
            }
            return Some(pos);
        }
    }

    None
}

/// Find the end of the current or next word from the current position
/// Handles Japanese characters as word characters
pub fn find_end_of_word(text: &str, current_col: usize) -> Option<usize> {
    let chars: Vec<char> = text.chars().collect();
    if current_col >= chars.len() {
        return None;
    }

    let mut pos = current_col;

    // Skip whitespace to find next word
    while pos < chars.len() && get_character_type(chars[pos]) == CharacterType::Whitespace {
        pos += 1;
    }

    // If we're at a word or Japanese character, find its end
    if pos < chars.len() {
        let current_type = get_character_type(chars[pos]);
        if current_type == CharacterType::Word || current_type == CharacterType::Japanese {
            while pos < chars.len() {
                let char_type = get_character_type(chars[pos]);
                if char_type != CharacterType::Word && char_type != CharacterType::Japanese {
                    break;
                }
                pos += 1;
            }
            return Some(pos.saturating_sub(1));
        } else if current_type == CharacterType::Punctuation {
            // Handle punctuation sequences
            while pos < chars.len() && get_character_type(chars[pos]) == CharacterType::Punctuation
            {
                pos += 1;
            }
            return Some(pos.saturating_sub(1));
        }
    }

    None
}

/// Get the display width of a character (1 for most, 2 for CJK)
pub fn get_char_display_width(ch: char) -> usize {
    UnicodeWidthChar::width(ch).unwrap_or(0)
}

/// Calculate the display width of a string
pub fn calculate_display_width(text: &str) -> usize {
    text.chars().map(get_char_display_width).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_japanese_character_detection() {
        assert!(is_japanese_character('こ')); // Hiragana
        assert!(is_japanese_character('ニ')); // Katakana
        assert!(is_japanese_character('私')); // Kanji
        assert!(!is_japanese_character('a')); // ASCII
        assert!(!is_japanese_character(' ')); // Space
    }

    #[test]
    fn test_character_type_classification() {
        assert_eq!(get_character_type('こ'), CharacterType::Japanese);
        assert_eq!(get_character_type('a'), CharacterType::Word);
        assert_eq!(get_character_type('_'), CharacterType::Word);
        assert_eq!(get_character_type('.'), CharacterType::Punctuation);
        assert_eq!(get_character_type(' '), CharacterType::Whitespace);
    }

    #[test]
    fn test_next_word_boundary_japanese() {
        let text = "こんにちは。私、名前 Borat です";
        assert_eq!(find_next_word_boundary(text, 0), Some(6)); // Skip "こんにちは。" to "私"
        assert_eq!(find_next_word_boundary(text, 6), Some(8)); // Skip "私、" to "名前"
        assert_eq!(find_next_word_boundary(text, 8), Some(11)); // Skip "名前 " to "Borat"
    }

    #[test]
    fn test_previous_word_boundary_japanese() {
        let text = "こんにちは。私、名前 Borat です";
        assert_eq!(find_previous_word_boundary(text, 11), Some(8)); // From "Borat" to "名前"
        assert_eq!(find_previous_word_boundary(text, 8), Some(6)); // From "名前" to "私"
        assert_eq!(find_previous_word_boundary(text, 6), Some(0)); // From "私" to "こんにちは"
    }

    #[test]
    fn test_display_width_calculation() {
        assert_eq!(calculate_display_width("hello"), 5);
        assert_eq!(calculate_display_width("きんようび"), 10); // 5 chars × 2 width each
        assert_eq!(calculate_display_width("hello き"), 8); // 5 + 1 + 2
    }

    #[test]
    fn test_move_left_right_by_character() {
        // Test with Japanese text "きんようび" (5 chars, 10 display columns)
        let text = "きんようび";

        // Moving right from start (display col 0) should go to display col 2 (after 'き')
        assert_eq!(move_right_by_character(text, 0), 2);

        // Moving left from display col 2 should go back to 0
        assert_eq!(move_left_by_character(text, 2), 0);

        // Moving right from display col 2 should go to display col 4 (after 'ん')
        assert_eq!(move_right_by_character(text, 2), 4);

        // Moving from middle of a character should work correctly
        assert_eq!(move_right_by_character(text, 1), 2); // From middle of 'き' to after 'き'
        assert_eq!(move_left_by_character(text, 3), 2); // From middle of 'ん' to start of 'ん'

        // Test with mixed text
        let mixed = "Hello きん";
        assert_eq!(move_right_by_character(mixed, 5), 6); // After space
        assert_eq!(move_right_by_character(mixed, 6), 8); // After 'き'
        assert_eq!(move_left_by_character(mixed, 8), 6); // Back to before 'き'
    }
}
