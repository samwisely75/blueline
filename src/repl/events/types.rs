//! # Core Event Types
//!
//! Common types used throughout the event system including positions,
//! ranges, panes, editor modes, and pane capabilities.

use bitflags::bitflags;

/// Logical position in text content (line and column)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
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
    /// D prefix mode - waiting for second character after 'd' press
    DPrefix,
    /// Y prefix mode - waiting for second character after 'y' press
    YPrefix,
    /// Visual mode - character-wise text selection mode (vim's 'v')
    Visual,
    /// Visual Line mode - line-wise text selection mode (vim's 'V')
    VisualLine,
    /// Visual Block mode - block-wise text selection mode (vim's Ctrl+V)
    VisualBlock,
    /// Visual Block Insert mode - special insert mode for Visual Block 'I' and 'A' commands
    VisualBlockInsert,
}

bitflags! {
    /// Capabilities that control what operations are allowed on a pane
    ///
    /// This bitflag enum provides fine-grained control over pane functionality,
    /// allowing for flexible configuration without hardcoding pane-specific behavior.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use blueline::repl::events::PaneCapabilities;
    ///
    /// // Request pane with full access
    /// let request_caps = PaneCapabilities::FULL_ACCESS;
    ///
    /// // Response pane (read-only)
    /// let response_caps = PaneCapabilities::READ_ONLY;
    ///
    /// // Custom configuration
    /// let custom_caps = PaneCapabilities::FOCUSABLE | PaneCapabilities::NAVIGABLE;
    ///
    /// // Check capabilities
    /// if request_caps.contains(PaneCapabilities::EDITABLE) {
    ///     // Allow editing operations
    /// }
    /// ```
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct PaneCapabilities: u32 {
        /// No capabilities - pane is completely inactive
        const NONE         = 0b00000000;

        /// Can receive focus and become the active pane
        const FOCUSABLE    = 0b00000001;

        /// Can edit content (insert, delete, modify text)
        const EDITABLE     = 0b00000010;

        /// Can select text for visual operations
        const SELECTABLE   = 0b00000100;

        /// Can scroll content vertically and horizontally
        const SCROLLABLE   = 0b00001000;

        /// Can navigate with cursor movement commands
        const NAVIGABLE    = 0b00010000;

        /// Standard read-only configuration for display panes
        /// Allows focus, navigation, selection, and scrolling but not editing
        const READ_ONLY = Self::FOCUSABLE.bits()
                        | Self::SCROLLABLE.bits()
                        | Self::NAVIGABLE.bits()
                        | Self::SELECTABLE.bits();

        /// Full access configuration for editable panes
        /// Enables all capabilities for complete pane functionality
        const FULL_ACCESS = Self::FOCUSABLE.bits()
                          | Self::EDITABLE.bits()
                          | Self::SELECTABLE.bits()
                          | Self::SCROLLABLE.bits()
                          | Self::NAVIGABLE.bits();
    }
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
