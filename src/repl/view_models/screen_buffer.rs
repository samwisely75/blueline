//! # Screen Buffer for Double Buffering
//!
//! Implements terminal screen state representation and diff-based rendering.
//! This enables flicker-free updates by only rendering cells that have changed.

use crossterm::style::{Attribute, Color};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Type alias for a row of buffer cells to reduce complexity
pub type BufferRow = [BufferCell];

/// A single terminal cell with character and styling information
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BufferCell {
    /// The character to display
    pub character: char,
    /// Foreground color
    pub fg_color: Option<Color>,
    /// Background color  
    pub bg_color: Option<Color>,
    /// Text attributes (bold, italic, etc.)
    pub attributes: Vec<Attribute>,
}

impl BufferCell {
    /// Create a new empty cell (space with default styling)
    pub fn empty() -> Self {
        Self {
            character: ' ',
            fg_color: None,
            bg_color: None,
            attributes: Vec::new(),
        }
    }

    /// Create a cell with just a character
    pub fn with_char(ch: char) -> Self {
        Self {
            character: ch,
            fg_color: None,
            bg_color: None,
            attributes: Vec::new(),
        }
    }

    /// Create a cell with character and foreground color
    pub fn with_color(ch: char, fg: Color) -> Self {
        Self {
            character: ch,
            fg_color: Some(fg),
            bg_color: None,
            attributes: Vec::new(),
        }
    }

    /// Check if this cell represents "empty" content
    pub fn is_empty(&self) -> bool {
        self.character == ' '
            && self.fg_color.is_none()
            && self.bg_color.is_none()
            && self.attributes.is_empty()
    }
}

impl Default for BufferCell {
    fn default() -> Self {
        Self::empty()
    }
}

/// Terminal screen buffer for double buffering
#[derive(Debug, Clone)]
pub struct ScreenBuffer {
    /// 2D grid of cells [row][col]
    cells: Vec<Vec<BufferCell>>,
    /// Terminal width
    width: usize,
    /// Terminal height
    height: usize,
    /// Hash of each row for fast comparison
    row_hashes: Vec<u64>,
}

impl ScreenBuffer {
    /// Create a new screen buffer with given dimensions
    pub fn new(width: usize, height: usize) -> Self {
        let cells = vec![vec![BufferCell::empty(); width]; height];
        let row_hashes = vec![0u64; height];

        Self {
            cells,
            width,
            height,
            row_hashes,
        }
    }

    /// Get buffer dimensions
    pub fn dimensions(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    /// Resize the buffer (preserving content where possible)
    pub fn resize(&mut self, new_width: usize, new_height: usize) {
        // Resize existing rows
        for row in &mut self.cells {
            row.resize(new_width, BufferCell::empty());
        }

        // Add or remove rows
        self.cells
            .resize(new_height, vec![BufferCell::empty(); new_width]);
        self.row_hashes.resize(new_height, 0);

        self.width = new_width;
        self.height = new_height;
    }

    /// Clear the entire buffer
    pub fn clear(&mut self) {
        for row in &mut self.cells {
            for cell in row {
                *cell = BufferCell::empty();
            }
        }
        self.update_all_row_hashes();
    }

    /// Set a cell at specific position
    pub fn set_cell(&mut self, row: usize, col: usize, cell: BufferCell) -> bool {
        if row >= self.height || col >= self.width {
            return false;
        }

        self.cells[row][col] = cell;
        self.update_row_hash(row);
        true
    }

    /// Get a cell at specific position
    pub fn get_cell(&self, row: usize, col: usize) -> Option<&BufferCell> {
        if row >= self.height || col >= self.width {
            return None;
        }
        Some(&self.cells[row][col])
    }

    /// Set text at a specific position (left-to-right)
    pub fn set_text(&mut self, row: usize, start_col: usize, text: &str, fg_color: Option<Color>) {
        if row >= self.height {
            return;
        }

        let mut col = start_col;
        for ch in text.chars() {
            if col >= self.width {
                break;
            }

            let cell = if let Some(color) = fg_color {
                BufferCell::with_color(ch, color)
            } else {
                BufferCell::with_char(ch)
            };

            self.cells[row][col] = cell;
            col += 1;
        }

        self.update_row_hash(row);
    }

    /// Clear a specific row
    pub fn clear_row(&mut self, row: usize) {
        if row >= self.height {
            return;
        }

        for cell in &mut self.cells[row] {
            *cell = BufferCell::empty();
        }
        self.update_row_hash(row);
    }

    /// Clear from a position to end of row
    pub fn clear_row_from(&mut self, row: usize, start_col: usize) {
        if row >= self.height {
            return;
        }

        for col in start_col..self.width {
            self.cells[row][col] = BufferCell::empty();
        }
        self.update_row_hash(row);
    }

    /// Calculate hash for a specific row
    fn calculate_row_hash(&self, row: usize) -> u64 {
        if row >= self.height {
            return 0;
        }

        let mut hasher = DefaultHasher::new();
        self.cells[row].hash(&mut hasher);
        hasher.finish()
    }

    /// Update hash for a specific row
    fn update_row_hash(&mut self, row: usize) {
        if row < self.height {
            self.row_hashes[row] = self.calculate_row_hash(row);
        }
    }

    /// Update all row hashes
    fn update_all_row_hashes(&mut self) {
        for row in 0..self.height {
            self.row_hashes[row] = self.calculate_row_hash(row);
        }
    }

    /// Compare with another buffer and return changed rows
    pub fn diff(&self, other: &ScreenBuffer) -> Vec<usize> {
        let mut changed_rows = Vec::new();

        // Different dimensions means everything changed
        if self.width != other.width || self.height != other.height {
            return (0..self.height.max(other.height)).collect();
        }

        // Compare row hashes for fast detection
        for row in 0..self.height {
            if row >= other.row_hashes.len() || self.row_hashes[row] != other.row_hashes[row] {
                changed_rows.push(row);
            }
        }

        changed_rows
    }

    /// Get a reference to a row
    pub fn get_row(&self, row: usize) -> Option<&BufferRow> {
        if row >= self.height {
            return None;
        }
        Some(&self.cells[row])
    }

    /// Check if buffer is completely empty
    pub fn is_empty(&self) -> bool {
        self.cells
            .iter()
            .all(|row| row.iter().all(|cell| cell.is_empty()))
    }

    /// Get content of a row as a string (for debugging)
    pub fn row_to_string(&self, row: usize) -> String {
        if row >= self.height {
            return String::new();
        }

        self.cells[row]
            .iter()
            .map(|cell| cell.character)
            .collect::<String>()
            .trim_end()
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buffer_cell_should_create_empty() {
        let cell = BufferCell::empty();
        assert_eq!(cell.character, ' ');
        assert_eq!(cell.fg_color, None);
        assert!(cell.is_empty());
    }

    #[test]
    fn buffer_cell_should_create_with_char() {
        let cell = BufferCell::with_char('A');
        assert_eq!(cell.character, 'A');
        assert!(!cell.is_empty());
    }

    #[test]
    fn screen_buffer_should_create() {
        let buffer = ScreenBuffer::new(80, 24);
        assert_eq!(buffer.dimensions(), (80, 24));
        assert!(buffer.is_empty());
    }

    #[test]
    fn screen_buffer_should_set_and_get_cell() {
        let mut buffer = ScreenBuffer::new(10, 5);
        let cell = BufferCell::with_char('X');

        assert!(buffer.set_cell(2, 3, cell.clone()));
        assert_eq!(buffer.get_cell(2, 3), Some(&cell));

        // Out of bounds should fail
        assert!(!buffer.set_cell(10, 10, cell));
    }

    #[test]
    fn screen_buffer_should_set_text() {
        let mut buffer = ScreenBuffer::new(20, 5);
        buffer.set_text(1, 5, "Hello", Some(Color::Red));

        assert_eq!(buffer.get_cell(1, 5).unwrap().character, 'H');
        assert_eq!(buffer.get_cell(1, 6).unwrap().character, 'e');
        assert_eq!(buffer.get_cell(1, 9).unwrap().character, 'o');
        assert_eq!(buffer.get_cell(1, 5).unwrap().fg_color, Some(Color::Red));
    }

    #[test]
    fn screen_buffer_should_detect_changes() {
        let mut buffer1 = ScreenBuffer::new(10, 5);
        let mut buffer2 = ScreenBuffer::new(10, 5);

        // Initially same
        assert_eq!(buffer1.diff(&buffer2), Vec::<usize>::new());

        // Change row 2
        buffer1.set_text(2, 0, "test", None);
        let changes = buffer1.diff(&buffer2);
        assert_eq!(changes, vec![2]);

        // Make buffer2 match
        buffer2.set_text(2, 0, "test", None);
        assert_eq!(buffer1.diff(&buffer2), Vec::<usize>::new());
    }

    #[test]
    fn screen_buffer_should_clear_row() {
        let mut buffer = ScreenBuffer::new(10, 5);
        buffer.set_text(1, 0, "test", None);

        assert!(!buffer.get_cell(1, 0).unwrap().is_empty());

        buffer.clear_row(1);
        assert!(buffer.get_cell(1, 0).unwrap().is_empty());
    }

    #[test]
    fn screen_buffer_should_resize() {
        let mut buffer = ScreenBuffer::new(5, 3);
        buffer.set_text(1, 1, "Hi", None);

        buffer.resize(10, 6);
        assert_eq!(buffer.dimensions(), (10, 6));

        // Previous content should be preserved
        assert_eq!(buffer.get_cell(1, 1).unwrap().character, 'H');
    }
}
