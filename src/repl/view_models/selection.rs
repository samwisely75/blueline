//! # Selection
//!
//! Encapsulated selection object with pure positional data.
//! Contains only start and end positions without mode or pane references.
//! Methods are pure functions that work with provided context.

use crate::repl::events::LogicalPosition;

/// Represents a text selection with start and end positions
///
/// This is a pure data structure that contains only positional information.
/// It does not know about editor modes, panes, or buffer content.
/// All operations that need mode or buffer context are provided
/// by the owning component (PaneState).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selection {
    /// Start position of the selection
    pub start: LogicalPosition,
    /// End position of the selection
    pub end: LogicalPosition,
}

impl Selection {
    /// Create a new selection between two positions
    ///
    /// The positions will be automatically normalized so that
    /// start <= end in document order.
    pub fn new(start: LogicalPosition, end: LogicalPosition) -> Self {
        let (normalized_start, normalized_end) = Self::normalize_positions(start, end);
        Self {
            start: normalized_start,
            end: normalized_end,
        }
    }

    /// Create a selection from a single position (cursor position)
    ///
    /// This creates a zero-width selection at the cursor position,
    /// which is useful for starting a selection.
    pub fn from_cursor(position: LogicalPosition) -> Self {
        Self {
            start: position,
            end: position,
        }
    }

    /// Get the normalized positions (start <= end in document order)
    ///
    /// Returns (start, end) where start is guaranteed to be before
    /// or equal to end in document order.
    pub fn normalize(&self) -> (LogicalPosition, LogicalPosition) {
        Self::normalize_positions(self.start, self.end)
    }

    /// Check if a position is within this selection
    ///
    /// Returns true if the position is between start and end (inclusive).
    /// This is a pure positional check without mode-specific logic.
    pub fn contains(&self, position: LogicalPosition) -> bool {
        let (start, end) = self.normalize();
        position >= start && position <= end
    }

    /// Check if this selection is empty (zero-width)
    ///
    /// Returns true if start and end positions are the same.
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// Extend the selection to include a new position
    ///
    /// Updates the end position while keeping the start position fixed.
    /// This is typically used when extending a selection with cursor movement.
    pub fn extend_to(&self, position: LogicalPosition) -> Self {
        Self {
            start: self.start,
            end: position,
        }
    }

    /// Get the anchor position (start of selection)
    ///
    /// This is the position where the selection was started,
    /// which remains fixed while extending the selection.
    pub fn anchor(&self) -> LogicalPosition {
        self.start
    }

    /// Get the cursor position (end of selection)
    ///
    /// This is the current cursor position, which moves when
    /// extending the selection.
    pub fn cursor(&self) -> LogicalPosition {
        self.end
    }

    /// Normalize two positions to ensure start <= end
    ///
    /// Private helper that handles the position ordering logic.
    /// Compares by line first, then by column.
    fn normalize_positions(
        pos1: LogicalPosition,
        pos2: LogicalPosition,
    ) -> (LogicalPosition, LogicalPosition) {
        if pos1.line < pos2.line || (pos1.line == pos2.line && pos1.column <= pos2.column) {
            (pos1, pos2)
        } else {
            (pos2, pos1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selection_new_should_normalize_positions() {
        let start = LogicalPosition::new(2, 5);
        let end = LogicalPosition::new(1, 3);
        let selection = Selection::new(start, end);

        // Positions should be swapped since end < start
        assert_eq!(selection.start, LogicalPosition::new(1, 3));
        assert_eq!(selection.end, LogicalPosition::new(2, 5));
    }

    #[test]
    fn selection_new_should_keep_order_when_already_normalized() {
        let start = LogicalPosition::new(1, 3);
        let end = LogicalPosition::new(2, 5);
        let selection = Selection::new(start, end);

        assert_eq!(selection.start, start);
        assert_eq!(selection.end, end);
    }

    #[test]
    fn selection_from_cursor_should_create_zero_width_selection() {
        let position = LogicalPosition::new(5, 10);
        let selection = Selection::from_cursor(position);

        assert_eq!(selection.start, position);
        assert_eq!(selection.end, position);
        assert!(selection.is_empty());
    }

    #[test]
    fn selection_contains_should_check_position_within_bounds() {
        let selection = Selection::new(LogicalPosition::new(1, 5), LogicalPosition::new(3, 10));

        // Position before selection
        assert!(!selection.contains(LogicalPosition::new(0, 5)));

        // Position at start
        assert!(selection.contains(LogicalPosition::new(1, 5)));

        // Position in middle
        assert!(selection.contains(LogicalPosition::new(2, 0)));

        // Position at end
        assert!(selection.contains(LogicalPosition::new(3, 10)));

        // Position after selection
        assert!(!selection.contains(LogicalPosition::new(4, 0)));
    }

    #[test]
    fn selection_extend_to_should_update_end_position() {
        let start = LogicalPosition::new(1, 5);
        let selection = Selection::from_cursor(start);

        let extended = selection.extend_to(LogicalPosition::new(3, 10));

        assert_eq!(extended.start, start);
        assert_eq!(extended.end, LogicalPosition::new(3, 10));
    }

    #[test]
    fn selection_normalize_should_return_ordered_positions() {
        let selection = Selection::new(LogicalPosition::new(5, 15), LogicalPosition::new(2, 8));

        let (start, end) = selection.normalize();

        assert_eq!(start, LogicalPosition::new(2, 8));
        assert_eq!(end, LogicalPosition::new(5, 15));
    }

    #[test]
    fn selection_anchor_and_cursor_should_return_correct_positions() {
        let start = LogicalPosition::new(1, 5);
        let end = LogicalPosition::new(3, 10);
        let selection = Selection::new(start, end);

        assert_eq!(selection.anchor(), start);
        assert_eq!(selection.cursor(), end);
    }

    #[test]
    fn selection_is_empty_should_detect_zero_width_selection() {
        let position = LogicalPosition::new(2, 7);
        let empty_selection = Selection::from_cursor(position);
        let non_empty_selection =
            Selection::new(LogicalPosition::new(1, 0), LogicalPosition::new(2, 0));

        assert!(empty_selection.is_empty());
        assert!(!non_empty_selection.is_empty());
    }

    #[test]
    fn selection_normalize_positions_should_handle_same_line_ordering() {
        let pos1 = LogicalPosition::new(2, 10);
        let pos2 = LogicalPosition::new(2, 5);

        let (start, end) = Selection::normalize_positions(pos1, pos2);

        assert_eq!(start, LogicalPosition::new(2, 5));
        assert_eq!(end, LogicalPosition::new(2, 10));
    }
}
