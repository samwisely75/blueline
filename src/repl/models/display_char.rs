//! # Display Character Management
//!
//! Provides display-aware character representation that extends BufferChar
//! with rendering, styling, and terminal-specific properties.

use crate::repl::models::buffer_char::BufferChar;
use unicode_width::UnicodeWidthChar;

/// A character with both logical (buffer) and display (rendering) properties
#[derive(Debug, Clone, PartialEq)]
pub struct DisplayChar {
    /// The underlying buffer character with logical properties
    pub buffer_char: BufferChar,

    // Display-specific properties for rendering (integrated directly)
    /// Whether this character is highlighted
    pub highlighted: bool,
    /// Whether this character is part of a search match
    pub search_match: bool,
    /// Background color (terminal color code)
    pub bg_color: Option<u8>,
    /// Foreground color (terminal color code)
    pub fg_color: Option<u8>,
    /// Whether the character should be rendered in bold
    pub bold: bool,
    /// Whether the character should be rendered with underline
    pub underline: bool,
    /// Whether the character should be rendered with italic
    pub italic: bool,
    /// Screen position where this character will be rendered (row, col)
    pub screen_position: (usize, usize),
    /// Display width of this character (1 for ASCII, 2 for CJK, etc.)
    pub display_width: usize,
}

impl DisplayChar {
    /// Create a new DisplayChar from a BufferChar
    pub fn from_buffer_char(buffer_char: BufferChar, screen_position: (usize, usize)) -> Self {
        // For backward compatibility, use default tab behavior (zero width)
        Self::from_buffer_char_with_tab_width(buffer_char, screen_position, 0)
    }

    /// Create a new DisplayChar from a BufferChar with tab width support
    pub fn from_buffer_char_with_tab_width(
        buffer_char: BufferChar,
        screen_position: (usize, usize),
        tab_width: usize,
    ) -> Self {
        // Calculate display width using Unicode width (terminal columns occupied)
        let display_width = match buffer_char.ch {
            '\t' if tab_width > 0 => {
                // Simple tab: always advance by tab_width characters
                tab_width
            }
            '\t' => 0, // Backward compatibility: zero width for tabs when tab_width = 0
            _ => UnicodeWidthChar::width(buffer_char.ch).unwrap_or(0),
        };

        Self {
            buffer_char,
            highlighted: false,
            search_match: false,
            bg_color: None,
            fg_color: None,
            bold: false,
            underline: false,
            italic: false,
            screen_position,
            display_width,
        }
    }

    /// Get the character
    pub fn ch(&self) -> char {
        self.buffer_char.ch
    }

    /// Get logical index in the buffer
    pub fn logical_index(&self) -> usize {
        self.buffer_char.logical_index
    }

    /// Get display width of this character
    pub fn display_width(&self) -> usize {
        self.display_width
    }

    /// Get screen position (row, col)
    pub fn screen_position(&self) -> (usize, usize) {
        self.screen_position
    }

    /// Get screen row position
    pub fn screen_row(&self) -> usize {
        self.screen_position.0
    }

    /// Get screen column position
    pub fn screen_col(&self) -> usize {
        self.screen_position.1
    }

    /// Check if this character is selected
    pub fn is_selected(&self) -> bool {
        self.buffer_char.selected
    }

    /// Check if this character is whitespace
    pub fn is_whitespace(&self) -> bool {
        self.buffer_char.is_whitespace()
    }

    /// Check if this character is a newline
    pub fn is_newline(&self) -> bool {
        self.buffer_char.is_newline()
    }

    /// Set highlight state
    pub fn set_highlighted(&mut self, highlighted: bool) {
        self.highlighted = highlighted;
    }

    /// Set search match state
    pub fn set_search_match(&mut self, search_match: bool) {
        self.search_match = search_match;
    }

    /// Set colors
    pub fn set_colors(&mut self, fg_color: Option<u8>, bg_color: Option<u8>) {
        self.fg_color = fg_color;
        self.bg_color = bg_color;
    }

    /// Set text styling
    pub fn set_styling(&mut self, bold: bool, underline: bool, italic: bool) {
        self.bold = bold;
        self.underline = underline;
        self.italic = italic;
    }

    /// Update screen position
    pub fn set_screen_position(&mut self, screen_position: (usize, usize)) {
        self.screen_position = screen_position;
    }

    /// Check if character has any styling applied
    pub fn has_styling(&self) -> bool {
        self.highlighted
            || self.search_match
            || self.bg_color.is_some()
            || self.fg_color.is_some()
            || self.bold
            || self.underline
            || self.italic
    }

    /// Generate ANSI escape sequence for this character's styling
    pub fn ansi_style_start(&self) -> String {
        if !self.has_styling() {
            return String::new();
        }

        let mut codes = Vec::new();

        // Text styling
        if self.bold {
            codes.push("1".to_string());
        }
        if self.underline {
            codes.push("4".to_string());
        }
        if self.italic {
            codes.push("3".to_string());
        }

        // Colors
        if let Some(fg) = self.fg_color {
            codes.push(format!("38;5;{fg}"));
        }
        if let Some(bg) = self.bg_color {
            codes.push(format!("48;5;{bg}"));
        }

        // Special highlighting
        if self.highlighted {
            codes.push("7".to_string()); // Reverse video
        }
        if self.search_match {
            codes.push("48;5;11".to_string()); // Yellow background
        }

        if codes.is_empty() {
            String::new()
        } else {
            format!("\x1b[{}m", codes.join(";"))
        }
    }

    /// Generate ANSI reset sequence (only if styling was applied)
    pub fn ansi_style_end(&self) -> String {
        if self.has_styling() {
            "\x1b[0m".to_string()
        } else {
            String::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::models::buffer_char::BufferLine;

    #[test]
    fn display_char_should_wrap_buffer_char() {
        let buffer_line = BufferLine::from_string("hello こ");
        let buffer_chars = buffer_line.chars();

        let display_char = DisplayChar::from_buffer_char(buffer_chars[0].clone(), (0, 0));
        assert_eq!(display_char.ch(), 'h');
        assert_eq!(display_char.logical_index(), 0);
        assert_eq!(display_char.display_width(), 1);
        assert_eq!(display_char.screen_position(), (0, 0));

        let japanese_display_char = DisplayChar::from_buffer_char(buffer_chars[6].clone(), (0, 6));
        assert_eq!(japanese_display_char.ch(), 'こ');
        assert_eq!(japanese_display_char.display_width(), 2);
        assert_eq!(japanese_display_char.screen_position(), (0, 6));
    }

    #[test]
    fn display_char_should_handle_styling() {
        let buffer_line = BufferLine::from_string("test");
        let mut display_char =
            DisplayChar::from_buffer_char(buffer_line.chars()[0].clone(), (0, 0));

        assert!(!display_char.has_styling());
        assert_eq!(display_char.ansi_style_start(), "");

        display_char.set_highlighted(true);
        display_char.set_styling(true, false, false);

        assert!(display_char.has_styling());
        assert_eq!(display_char.ansi_style_start(), "\x1b[1;7m");
        assert_eq!(display_char.ansi_style_end(), "\x1b[0m");
    }

    #[test]
    fn display_char_should_handle_tab_width_correctly() {
        use crate::repl::models::buffer_char::BufferChar;

        // Create a tab character
        let buffer_char = BufferChar::new('\t', 0, 0);

        // Test tab at column 0 with tab width 4
        let display_char = DisplayChar::from_buffer_char_with_tab_width(
            buffer_char.clone(),
            (0, 0), // screen position (row, col)
            4,      // tab width
        );
        assert_eq!(display_char.ch(), '\t');
        assert_eq!(display_char.display_width(), 4); // Always 4 spaces

        // Test tab at column 1 with tab width 4
        let display_char = DisplayChar::from_buffer_char_with_tab_width(
            buffer_char.clone(),
            (0, 1), // screen position (row, col)
            4,      // tab width
        );
        assert_eq!(display_char.display_width(), 4); // Always 4 spaces

        // Test tab at column 3 with tab width 4
        let display_char = DisplayChar::from_buffer_char_with_tab_width(
            buffer_char.clone(),
            (0, 3), // screen position (row, col)
            4,      // tab width
        );
        assert_eq!(display_char.display_width(), 4); // Always 4 spaces

        // Test tab at column 4 with tab width 4
        let display_char = DisplayChar::from_buffer_char_with_tab_width(
            buffer_char.clone(),
            (0, 4), // screen position (row, col)
            4,      // tab width
        );
        assert_eq!(display_char.display_width(), 4); // Always 4 spaces

        // Test tab with tab width 8
        let display_char = DisplayChar::from_buffer_char_with_tab_width(
            buffer_char.clone(),
            (0, 0), // screen position (row, col)
            8,      // tab width
        );
        assert_eq!(display_char.display_width(), 8); // Always 8 spaces

        // Test backward compatibility: tab width 0 should give zero width
        let display_char = DisplayChar::from_buffer_char_with_tab_width(
            buffer_char.clone(),
            (0, 0), // screen position (row, col)
            0,      // tab width (backward compatibility mode)
        );
        assert_eq!(display_char.display_width(), 0); // Zero width for backward compatibility
    }
}
