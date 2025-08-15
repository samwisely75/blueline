//! # Buffer Models
//!
//! Text buffer content and buffer model for MVVM architecture.
//! Handles text storage, cursor management, and basic editing operations.

use crate::repl::events::{LogicalPosition, LogicalRange, ModelEvent, Pane};
use crate::repl::models::buffer_char::CharacterBuffer;

/// Content of a text buffer with character-aware positioning
#[derive(Debug, Clone, PartialEq)]
pub struct BufferContent {
    buffer: CharacterBuffer,
}

impl BufferContent {
    /// Create new empty buffer
    pub fn new() -> Self {
        Self {
            buffer: CharacterBuffer::new(),
        }
    }

    /// Create buffer from existing lines
    pub fn from_lines(lines: Vec<String>) -> Self {
        Self {
            buffer: CharacterBuffer::from_lines(&lines),
        }
    }

    /// Get all lines as slice (for compatibility)
    pub fn lines(&self) -> Vec<String> {
        self.buffer.to_string_lines()
    }

    /// Get number of lines
    pub fn line_count(&self) -> usize {
        self.buffer.line_count()
    }

    /// Get specific line
    pub fn get_line(&self, index: usize) -> Option<String> {
        self.buffer.get_line(index).map(|line| line.to_string())
    }

    /// Get line length (logical character count)
    pub fn line_length(&self, index: usize) -> usize {
        self.buffer
            .get_line(index)
            .map_or(0, |line| line.char_count())
    }

    /// Get the position of the next character (for page scrolling)
    pub fn next_character_position(&self, current: LogicalPosition) -> LogicalPosition {
        let line_length = self.line_length(current.line);

        if current.column < line_length {
            // Move right within current line
            LogicalPosition::new(current.line, current.column + 1)
        } else if current.line + 1 < self.line_count() {
            // Move to start of next line
            LogicalPosition::new(current.line + 1, 0)
        } else {
            // Already at end of buffer
            current
        }
    }

    /// Get the position of the previous character (for page scrolling)
    pub fn previous_character_position(&self, current: LogicalPosition) -> LogicalPosition {
        if current.column > 0 {
            // Move left within current line
            LogicalPosition::new(current.line, current.column - 1)
        } else if current.line > 0 {
            // Move to end of previous line
            let prev_line = current.line - 1;
            let line_length = self.line_length(prev_line);
            LogicalPosition::new(prev_line, line_length)
        } else {
            // Already at start of buffer
            current
        }
    }

    /// Insert text at position, returning event
    pub fn insert_text(&mut self, pane: Pane, position: LogicalPosition, text: &str) -> ModelEvent {
        // Use CharacterBuffer's character-aware insertion
        let mut current_line = position.line;
        let mut current_col = position.column;

        for ch in text.chars() {
            self.buffer.insert_char(current_line, current_col, ch);
            if ch == '\n' {
                current_line += 1;
                current_col = 0;
            } else {
                current_col += 1;
            }
        }

        ModelEvent::TextInserted {
            pane,
            position,
            text: text.to_string(),
        }
    }

    /// Delete text in range, returning event if successful
    pub fn delete_range(&mut self, pane: Pane, range: LogicalRange) -> Option<ModelEvent> {
        if range.start.line >= self.buffer.line_count()
            || range.end.line >= self.buffer.line_count()
        {
            return None;
        }

        if range.start == range.end {
            return None; // Nothing to delete
        }

        // Use CharacterBuffer's character-aware deletion
        // Delete character by character from end to start of range

        // Handle single-line deletion separately to avoid incorrect line joining
        if range.start.line == range.end.line {
            // Deletion within a single line
            let mut pos = range.end.column;
            while pos > range.start.column {
                pos -= 1;
                self.buffer.delete_char(range.start.line, pos);
            }
        } else {
            // Multi-line deletion
            let mut current_pos = range.end;
            while current_pos != range.start {
                // Move backwards through the range
                if current_pos.column > 0 {
                    current_pos.column -= 1;
                    self.buffer
                        .delete_char(current_pos.line, current_pos.column);
                } else if current_pos.line > range.start.line {
                    // Move to end of previous line
                    current_pos.line -= 1;
                    if let Some(line) = self.buffer.get_line(current_pos.line) {
                        current_pos.column = line.char_count();
                        if current_pos.line + 1 < self.buffer.line_count() {
                            // Join the current line with the next line (removes the newline)
                            self.buffer
                                .join_lines(current_pos.line, current_pos.line + 1);
                        }
                    }
                } else {
                    break;
                }
            }
        }

        Some(ModelEvent::TextDeleted { pane, range })
    }

    /// Check if position is valid within this buffer
    pub fn is_valid_position(&self, position: LogicalPosition) -> bool {
        position.line < self.buffer.line_count()
            && position.column <= self.line_length(position.line)
    }

    /// Clamp position to valid bounds
    pub fn clamp_position(&self, position: LogicalPosition) -> LogicalPosition {
        let line = position
            .line
            .min(self.buffer.line_count().saturating_sub(1));
        let column = position.column.min(self.line_length(line));
        LogicalPosition::new(line, column)
    }

    /// Get text content as single string
    pub fn get_text(&self) -> String {
        self.buffer.to_string_lines().join("\n")
    }

    /// Set entire content from string
    pub fn set_text(&mut self, text: &str) {
        let lines: Vec<String> = if text.is_empty() {
            vec![String::new()]
        } else {
            text.lines().map(|s| s.to_string()).collect()
        };
        self.buffer = CharacterBuffer::from_lines(&lines);
    }

    /// Get access to the underlying character buffer for display layer
    pub fn character_buffer(&self) -> &CharacterBuffer {
        &self.buffer
    }

    /// Get mutable access to the underlying character buffer for word navigation
    pub fn character_buffer_mut(&mut self) -> &mut CharacterBuffer {
        &mut self.buffer
    }

    /// Get word boundaries for a specific line (calculates if not cached)
    pub fn get_line_word_boundaries(
        &mut self,
        line_index: usize,
    ) -> Option<&crate::text::word_segmenter::WordBoundaries> {
        self.buffer.get_line_word_boundaries(line_index)
    }
}

impl Default for BufferContent {
    fn default() -> Self {
        Self::new()
    }
}

/// Text buffer model for a specific pane
#[derive(Debug, Clone)]
pub struct BufferModel {
    content: BufferContent,
    pane: Pane,
    cursor: LogicalPosition,
    scroll_offset: usize,
}

impl BufferModel {
    /// Create new buffer for pane
    pub fn new(pane: Pane) -> Self {
        Self {
            content: BufferContent::new(),
            pane,
            cursor: LogicalPosition::zero(),
            scroll_offset: 0,
        }
    }

    /// Get buffer content
    pub fn content(&self) -> &BufferContent {
        &self.content
    }

    /// Get mutable buffer content
    pub fn content_mut(&mut self) -> &mut BufferContent {
        &mut self.content
    }

    /// Get the pane this buffer belongs to
    pub fn pane(&self) -> Pane {
        self.pane
    }

    /// Get current cursor position
    pub fn cursor(&self) -> LogicalPosition {
        self.cursor
    }

    /// Set cursor position (clamped to valid bounds)
    pub fn set_cursor(&mut self, position: LogicalPosition) -> Option<ModelEvent> {
        let old_pos = self.cursor;
        let new_pos = self.content.clamp_position(position);

        if old_pos != new_pos {
            self.cursor = new_pos;
            Some(ModelEvent::CursorMoved {
                pane: self.pane,
                old_pos,
                new_pos,
            })
        } else {
            None
        }
    }

    /// Get scroll offset
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// Set scroll offset
    pub fn set_scroll_offset(&mut self, offset: usize) {
        self.scroll_offset = offset;
    }

    /// Move cursor left, returning new position and event
    pub fn move_cursor_left(&mut self) -> Option<ModelEvent> {
        let current = self.cursor;

        if current.column > 0 {
            // Move left within current line
            self.set_cursor(LogicalPosition::new(current.line, current.column - 1))
        } else if current.line > 0 {
            // Move to end of previous line
            let prev_line = current.line - 1;
            let line_length = self.content.line_length(prev_line);
            self.set_cursor(LogicalPosition::new(prev_line, line_length))
        } else {
            None // Already at start of buffer
        }
    }

    /// Move cursor right, returning new position and event
    pub fn move_cursor_right(&mut self) -> Option<ModelEvent> {
        let current = self.cursor;
        let line_length = self.content.line_length(current.line);

        if current.column < line_length {
            // Move right within current line
            self.set_cursor(LogicalPosition::new(current.line, current.column + 1))
        } else if current.line + 1 < self.content.line_count() {
            // Move to start of next line
            self.set_cursor(LogicalPosition::new(current.line + 1, 0))
        } else {
            None // Already at end of buffer
        }
    }

    /// Insert character at cursor, returning event
    pub fn insert_char(&mut self, ch: char) -> ModelEvent {
        let event = self
            .content
            .insert_text(self.pane, self.cursor, &ch.to_string());

        // Move cursor forward - handle newlines properly
        if ch == '\n' {
            self.cursor = LogicalPosition::new(self.cursor.line + 1, 0);
        } else {
            self.cursor = LogicalPosition::new(self.cursor.line, self.cursor.column + 1);
        }

        event
    }

    /// Insert text at cursor, returning event
    pub fn insert_text(&mut self, text: &str) -> ModelEvent {
        let event = self.content.insert_text(self.pane, self.cursor, text);

        // Update cursor position based on inserted text (character-aware)
        if text.contains('\n') {
            let lines: Vec<&str> = text.split('\n').collect();
            let new_line = self.cursor.line + lines.len() - 1;
            let new_column = if lines.len() > 1 {
                lines.last().unwrap().chars().count() // Character count, not byte count
            } else {
                self.cursor.column + text.chars().count()
            };
            self.cursor = LogicalPosition::new(new_line, new_column);
        } else {
            self.cursor =
                LogicalPosition::new(self.cursor.line, self.cursor.column + text.chars().count());
        }

        event
    }

    /// Move cursor to next word boundary, returning new position and event
    pub fn move_cursor_to_next_word(&mut self) -> Option<ModelEvent> {
        let current = self.cursor;

        if let Some(buffer_line) = self.content.character_buffer().get_line(current.line) {
            if let Some(next_pos) = buffer_line.find_next_word_start(current.column) {
                return self.set_cursor(LogicalPosition::new(current.line, next_pos));
            }
        }

        // If no word boundary found on current line, move to beginning of next line
        if current.line + 1 < self.content.line_count() {
            self.set_cursor(LogicalPosition::new(current.line + 1, 0))
        } else {
            None
        }
    }

    /// Move cursor to previous word boundary, returning new position and event
    pub fn move_cursor_to_previous_word(&mut self) -> Option<ModelEvent> {
        let current = self.cursor;

        if let Some(buffer_line) = self.content.character_buffer().get_line(current.line) {
            if let Some(prev_pos) = buffer_line.find_previous_word_start(current.column) {
                return self.set_cursor(LogicalPosition::new(current.line, prev_pos));
            }
        }

        // If no word boundary found on current line, move to end of previous line
        if current.line > 0 {
            let prev_line = current.line - 1;
            let line_length = self.content.line_length(prev_line);
            self.set_cursor(LogicalPosition::new(prev_line, line_length))
        } else {
            None
        }
    }

    /// Move cursor to end of current or next word, returning new position and event
    pub fn move_cursor_to_end_of_word(&mut self) -> Option<ModelEvent> {
        let current = self.cursor;

        if let Some(buffer_line) = self.content.character_buffer().get_line(current.line) {
            if let Some(end_pos) = buffer_line.find_next_word_end(current.column) {
                return self.set_cursor(LogicalPosition::new(current.line, end_pos));
            }
        }

        None // If no end of word found, stay at current position
    }

    /// Get word boundaries for a specific line (calculates if not cached)
    pub fn get_line_word_boundaries(
        &mut self,
        line_index: usize,
    ) -> Option<&crate::text::word_segmenter::WordBoundaries> {
        self.content.get_line_word_boundaries(line_index)
    }

    /// Get mutable access to the underlying character buffer for word navigation
    pub fn character_buffer_mut(&mut self) -> &mut CharacterBuffer {
        self.content.character_buffer_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buffer_content_should_create_with_empty_line() {
        let content = BufferContent::new();
        assert_eq!(content.line_count(), 1);
        assert_eq!(content.get_line(0), Some(String::new()));
    }

    #[test]
    fn buffer_content_should_insert_single_char() {
        let mut content = BufferContent::new();
        let pos = LogicalPosition::new(0, 0);

        content.insert_text(Pane::Request, pos, "a");

        assert_eq!(content.get_text(), "a");
    }

    #[test]
    fn buffer_content_should_insert_multiline_text() {
        let mut content = BufferContent::new();
        let pos = LogicalPosition::new(0, 0);

        content.insert_text(Pane::Request, pos, "hello\nworld");

        assert_eq!(content.line_count(), 2);
        assert_eq!(content.get_line(0), Some("hello".to_string()));
        assert_eq!(content.get_line(1), Some("world".to_string()));
    }

    #[test]
    fn buffer_model_should_move_cursor_left() {
        let mut buffer = BufferModel::new(Pane::Request);
        buffer.insert_text("hello");
        // Now cursor is at (0, 5), move it left

        let event = buffer.move_cursor_left();

        assert!(event.is_some());
        assert_eq!(buffer.cursor(), LogicalPosition::new(0, 4));
    }

    #[test]
    fn buffer_model_should_move_cursor_right() {
        let mut buffer = BufferModel::new(Pane::Request);
        buffer.insert_text("hello");
        buffer.cursor = LogicalPosition::new(0, 2);

        let event = buffer.move_cursor_right();

        assert!(event.is_some());
        assert_eq!(buffer.cursor(), LogicalPosition::new(0, 3));
    }

    #[test]
    fn buffer_content_should_insert_japanese_hiragana_text() {
        let mut content = BufferContent::new();
        let pos = LogicalPosition::new(0, 0);

        content.insert_text(Pane::Request, pos, "こんにちは");

        assert_eq!(content.get_text(), "こんにちは");
        assert_eq!(content.line_count(), 1);
    }

    #[test]
    fn buffer_content_should_insert_japanese_katakana_text() {
        let mut content = BufferContent::new();
        let pos = LogicalPosition::new(0, 0);

        content.insert_text(Pane::Request, pos, "カタカナ");

        assert_eq!(content.get_text(), "カタカナ");
        assert_eq!(content.line_count(), 1);
    }

    #[test]
    fn buffer_content_should_insert_japanese_kanji_text() {
        let mut content = BufferContent::new();
        let pos = LogicalPosition::new(0, 0);

        content.insert_text(Pane::Request, pos, "日本語");

        assert_eq!(content.get_text(), "日本語");
        assert_eq!(content.line_count(), 1);
    }

    #[test]
    fn buffer_content_should_insert_mixed_japanese_and_ascii_text() {
        let mut content = BufferContent::new();
        let pos = LogicalPosition::new(0, 0);

        content.insert_text(Pane::Request, pos, "Hello こんにちは World 世界");

        assert_eq!(content.get_text(), "Hello こんにちは World 世界");
        assert_eq!(content.line_count(), 1);
    }

    #[test]
    fn buffer_content_should_insert_multiline_japanese_text() {
        let mut content = BufferContent::new();
        let pos = LogicalPosition::new(0, 0);

        content.insert_text(Pane::Request, pos, "こんにちは\n世界");

        assert_eq!(content.line_count(), 2);
        assert_eq!(content.get_line(0), Some("こんにちは".to_string()));
        assert_eq!(content.get_line(1), Some("世界".to_string()));
    }

    #[test]
    fn buffer_model_should_insert_japanese_character() {
        let mut buffer = BufferModel::new(Pane::Request);

        let event = buffer.insert_char('あ');

        assert_eq!(buffer.content().get_text(), "あ");
        assert_eq!(buffer.cursor(), LogicalPosition::new(0, 1));
        if let ModelEvent::TextInserted { text, .. } = event {
            assert_eq!(text, "あ");
        } else {
            panic!("Expected TextInserted event");
        }
    }

    #[test]
    fn buffer_content_should_handle_long_japanese_text_with_line_counting() {
        let mut content = BufferContent::new();
        let pos = LogicalPosition::new(0, 0);

        // Create a long Japanese text with multiple lines (original content, not copyrighted)
        let long_japanese_text = "これはとても長い日本語のテストです。\n\
            プログラミングにおいて、文字エンコーディングは重要な概念です。\n\
            UTF-8は現在最も広く使用されている文字エンコーディングの一つです。\n\
            日本語、中国語、韓国語などの東アジアの言語を適切に表示するためには、\n\
            ダブルバイト文字の処理が必要になります。\n\
            このテストでは、長いテキストが正しく処理されることを確認します。\n\
            各行の文字数や表示幅を正確に計算することは、\n\
            ターミナルアプリケーションにとって非常に重要です。\n\
            スクロール機能やカーソル移動も、\n\
            ダブルバイト文字で正しく動作する必要があります。";

        content.insert_text(Pane::Request, pos, long_japanese_text);

        assert_eq!(content.line_count(), 10);
        assert_eq!(
            content.get_line(0),
            Some("これはとても長い日本語のテストです。".to_string())
        );
        assert_eq!(
            content.get_line(1),
            Some("プログラミングにおいて、文字エンコーディングは重要な概念です。".to_string())
        );
        assert_eq!(
            content.get_line(9),
            Some("ダブルバイト文字で正しく動作する必要があります。".to_string())
        );
    }

    #[test]
    fn buffer_model_should_navigate_cursor_in_long_japanese_text() {
        let mut buffer = BufferModel::new(Pane::Request);

        // Insert multi-line Japanese text
        buffer.insert_text("日本語の一行目です。\n二行目はここです。\n三行目も日本語です。");

        // Test cursor movement to specific positions
        buffer.set_cursor(LogicalPosition::new(1, 5)); // Move to middle of second line
        assert_eq!(buffer.cursor(), LogicalPosition::new(1, 5));

        // Test moving right from Japanese characters
        let event = buffer.move_cursor_right();
        assert!(event.is_some());
        assert_eq!(buffer.cursor(), LogicalPosition::new(1, 6));

        // Test moving left from Japanese characters
        let event = buffer.move_cursor_left();
        assert!(event.is_some());
        assert_eq!(buffer.cursor(), LogicalPosition::new(1, 5));
    }

    #[test]
    fn buffer_content_should_handle_very_long_single_japanese_line() {
        let mut content = BufferContent::new();
        let pos = LogicalPosition::new(0, 0);

        // Create a very long single line with Japanese text for wrapping test
        let long_line = "あいうえおかきくけこさしすせそたちつてとなにぬねのはひふへほまみむめもやゆよらりるれろわをん".repeat(5);

        content.insert_text(Pane::Request, pos, &long_line);

        assert_eq!(content.line_count(), 1);
        assert_eq!(content.line_length(0), long_line.chars().count()); // Character count, not byte length
        assert_eq!(content.get_text(), long_line);
    }

    #[test]
    fn buffer_content_should_handle_mixed_ascii_japanese_long_line() {
        let mut content = BufferContent::new();
        let pos = LogicalPosition::new(0, 0);

        // Create a very long line mixing ASCII and Japanese (realistic scenario)
        let mixed_long_line = "Anthropic Claude は現時点ではコーディング作業に対して最も優れた LLM であると言われている。しかし、GitHub Copilot や ChatGPT などの他の AI ツールも非常に有用である。特に VS Code の拡張機能として利用する場合、開発者の生産性を大幅に向上させることができる。日本語のコメントや変数名を使用する場合でも、適切に処理される必要がある。".repeat(3);

        content.insert_text(Pane::Request, pos, &mixed_long_line);

        assert_eq!(content.line_count(), 1);
        assert_eq!(content.line_length(0), mixed_long_line.chars().count()); // Character count, not byte length
        assert_eq!(content.get_text(), mixed_long_line);

        // Test that mixed content is preserved correctly
        let text = content.get_text();
        assert!(text.contains("Anthropic Claude"));
        assert!(text.contains("は現時点では"));
        assert!(text.contains("LLM"));
        assert!(text.contains("であると言われている"));
    }

    #[test]
    fn buffer_model_should_navigate_in_mixed_ascii_japanese_text() {
        let mut buffer = BufferModel::new(Pane::Request);

        // Insert mixed text
        let mixed_text = "Hello こんにちは World 世界 API エンドポイント";
        buffer.insert_text(mixed_text);

        // Move cursor to different positions in mixed text
        buffer.set_cursor(LogicalPosition::new(0, 6)); // After "Hello "
        assert_eq!(buffer.cursor(), LogicalPosition::new(0, 6));

        // Move right through Japanese characters
        for _ in 0..5 {
            // Move through "こんにちは"
            buffer.move_cursor_right();
        }
        assert_eq!(buffer.cursor(), LogicalPosition::new(0, 11)); // After "Hello こんにちは"

        // Move left back through Japanese characters
        for _ in 0..2 {
            buffer.move_cursor_left();
        }
        assert_eq!(buffer.cursor(), LogicalPosition::new(0, 9)); // In middle of "こんにちは"
    }

    #[test]
    fn buffer_content_should_handle_extremely_long_wrapped_mixed_line() {
        let mut content = BufferContent::new();
        let pos = LogicalPosition::new(0, 0);

        // Create an extremely long line that will definitely wrap (no \n)
        let base_text = "Programming プログラミング is とても楽しい activity アクティビティ for developers 開発者 who love コードを書くこと and creating アプリケーション applications. ";
        let extremely_long_line = base_text.repeat(20); // Very long single line

        content.insert_text(Pane::Request, pos, &extremely_long_line);

        // Should still be one logical line
        assert_eq!(content.line_count(), 1);
        assert_eq!(content.get_text(), extremely_long_line);

        // Verify mixed content integrity
        let text = content.get_text();
        assert!(text.contains("Programming プログラミング"));
        assert!(text.contains("developers 開発者"));
        assert!(text.contains("アプリケーション applications"));
    }

    #[test]
    fn buffer_content_should_handle_get_space_enter_enter_sequence() {
        let mut content = BufferContent::new();

        // Simulate the exact sequence: "GET " + Enter + Enter
        content.insert_text(Pane::Request, LogicalPosition::new(0, 0), "G");
        content.insert_text(Pane::Request, LogicalPosition::new(0, 1), "E");
        content.insert_text(Pane::Request, LogicalPosition::new(0, 2), "T");
        content.insert_text(Pane::Request, LogicalPosition::new(0, 3), " ");
        content.insert_text(Pane::Request, LogicalPosition::new(0, 4), "\n");
        content.insert_text(Pane::Request, LogicalPosition::new(1, 0), "\n");

        // Should have 3 lines: "GET ", "", ""
        assert_eq!(content.line_count(), 3);
        assert_eq!(content.get_line(0), Some("GET ".to_string()));
        assert_eq!(content.get_line(1), Some("".to_string()));
        assert_eq!(content.get_line(2), Some("".to_string()));

        let full_text = content.get_text();
        assert_eq!(full_text, "GET \n\n");
    }

    #[test]
    fn buffer_content_should_handle_unicode_character_boundary_insertion() {
        let mut content = BufferContent::new();

        // Insert Japanese characters first
        content.insert_text(Pane::Request, LogicalPosition::new(0, 0), "こんにちは");
        assert_eq!(content.get_line(0), Some("こんにちは".to_string()));

        // Insert text in the middle (character index 2, which is "に")
        content.insert_text(Pane::Request, LogicalPosition::new(0, 2), "X");
        assert_eq!(content.get_line(0), Some("こんXにちは".to_string()));

        // Insert newline in the middle of Unicode text (after position 3, which is "に")
        content.insert_text(Pane::Request, LogicalPosition::new(0, 4), "\n");
        assert_eq!(content.line_count(), 2);
        assert_eq!(content.get_line(0), Some("こんXに".to_string()));
        assert_eq!(content.get_line(1), Some("ちは".to_string()));
    }

    #[test]
    fn buffer_content_should_handle_mixed_ascii_unicode_insertion() {
        let mut content = BufferContent::new();

        // Insert mixed ASCII and Unicode
        content.insert_text(Pane::Request, LogicalPosition::new(0, 0), "Hello");
        content.insert_text(Pane::Request, LogicalPosition::new(0, 5), " こんにちは");
        assert_eq!(content.get_line(0), Some("Hello こんにちは".to_string()));

        // Insert text after Japanese characters (character position 11)
        content.insert_text(Pane::Request, LogicalPosition::new(0, 11), " World");
        assert_eq!(
            content.get_line(0),
            Some("Hello こんにちは World".to_string())
        );
    }

    #[test]
    fn buffer_model_should_navigate_by_word_boundaries() {
        let mut buffer = BufferModel::new(Pane::Request);
        buffer.insert_text("hello こんにちは world");
        buffer.set_cursor(LogicalPosition::new(0, 0));

        // Move to next word (should go to "こんにちは" at position 6)
        let event = buffer.move_cursor_to_next_word();
        assert!(event.is_some());
        assert_eq!(buffer.cursor(), LogicalPosition::new(0, 6));

        // Move to next word (should go to "world" at position 12)
        let event = buffer.move_cursor_to_next_word();
        assert!(event.is_some());
        assert_eq!(buffer.cursor(), LogicalPosition::new(0, 12));

        // Move to previous word (should go back to "こんにちは" at position 6)
        let event = buffer.move_cursor_to_previous_word();
        assert!(event.is_some());
        assert_eq!(buffer.cursor(), LogicalPosition::new(0, 6));
    }

    #[test]
    fn buffer_model_should_find_next_word_end() {
        let mut buffer = BufferModel::new(Pane::Request);
        buffer.insert_text("hello こんにちは world");
        buffer.set_cursor(LogicalPosition::new(0, 0));

        // Move to end of "hello" (should go to position 4)
        let event = buffer.move_cursor_to_end_of_word();
        assert!(event.is_some());
        assert_eq!(buffer.cursor(), LogicalPosition::new(0, 4));

        // Move cursor to start of Japanese word and find its end
        buffer.set_cursor(LogicalPosition::new(0, 6));
        let event = buffer.move_cursor_to_end_of_word();
        assert!(event.is_some());
        assert_eq!(buffer.cursor(), LogicalPosition::new(0, 10)); // End of "こんにちは"
    }

    #[test]
    fn buffer_model_cursor_position_after_tab_insertion() {
        let mut buffer = BufferModel::new(Pane::Request);

        // Insert "TEST"
        buffer.insert_text("TEST");
        let cursor_after_test = buffer.cursor();
        println!("Cursor after 'TEST': {cursor_after_test:?}");
        assert_eq!(cursor_after_test, LogicalPosition::new(0, 4));

        // Insert tab
        buffer.insert_text("\t");
        let cursor_after_tab = buffer.cursor();
        println!("Cursor after tab: {cursor_after_tab:?}");
        assert_eq!(cursor_after_tab, LogicalPosition::new(0, 5));

        // Insert "TEST"
        buffer.insert_text("TEST");
        let cursor_after_second_test = buffer.cursor();
        println!("Cursor after second 'TEST': {cursor_after_second_test:?}");
        assert_eq!(cursor_after_second_test, LogicalPosition::new(0, 9));

        // Check buffer content
        let content = buffer.content().get_text();
        println!("Buffer content: '{content}'");
        assert_eq!(content, "TEST\tTEST");
    }
}
