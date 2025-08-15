//! # Core Event Types
//!
//! Common types used throughout the event system including positions,
//! ranges, panes, and editor modes.

/// Logical position in text content (line and column)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LogicalPosition {
    pub line: usize,
    pub column: usize,
}

impl LogicalPosition {
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }

    pub fn zero() -> Self {
        Self::new(0, 0)
    }
}

/// Range in logical coordinates for text operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LogicalRange {
    pub start: LogicalPosition,
    pub end: LogicalPosition,
}

impl LogicalRange {
    pub fn new(start: LogicalPosition, end: LogicalPosition) -> Self {
        Self { start, end }
    }

    pub fn single_char(position: LogicalPosition) -> Self {
        Self {
            start: position,
            end: LogicalPosition::new(position.line, position.column + 1),
        }
    }
}

/// Which pane is currently active
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Pane {
    Request,
    Response,
}

/// Editor mode (vim-style)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorMode {
    Normal,
    Insert,
    Command,
    /// G prefix mode - waiting for second character after 'g' press
    GPrefix,
    /// Visual mode - character-wise text selection mode (vim's 'v')
    Visual,
    /// Visual Line mode - line-wise text selection mode (vim's 'V')
    VisualLine,
    /// Visual Block mode - block-wise text selection mode (vim's Ctrl+V)
    VisualBlock,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logical_position_should_create_correctly() {
        let pos = LogicalPosition::new(5, 10);
        assert_eq!(pos.line, 5);
        assert_eq!(pos.column, 10);
    }

    #[test]
    fn logical_position_zero_should_be_origin() {
        let pos = LogicalPosition::zero();
        assert_eq!(pos.line, 0);
        assert_eq!(pos.column, 0);
    }

    #[test]
    fn logical_range_should_create_correctly() {
        let start = LogicalPosition::new(1, 2);
        let end = LogicalPosition::new(3, 4);
        let range = LogicalRange::new(start, end);

        assert_eq!(range.start, start);
        assert_eq!(range.end, end);
    }

    #[test]
    fn single_char_range_should_span_one_column() {
        let pos = LogicalPosition::new(1, 5);
        let range = LogicalRange::single_char(pos);

        assert_eq!(range.start, pos);
        assert_eq!(range.end, LogicalPosition::new(1, 6));
    }
}
