//! # App State Types
//!
//! Common types used throughout the app_state module, replacing complex type aliases
//! with clear, explicit structs that provide better type safety and maintainability.

use crate::repl::models::buffer::YankType;
use crate::repl::models::pane_state::Pane;
use crate::repl::models::LogicalPosition;

/// Visual selection state for tracking selected regions across panes
///
/// Replaces the complex type alias:
/// `type VisualSelectionState = (Option<LogicalPosition>, Option<LogicalPosition>, Option<Pane>)`
///
/// Provides clear field names and better encapsulation for visual selection management.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisualSelectionState {
    /// Start position of the visual selection
    pub start: Option<LogicalPosition>,

    /// End position of the visual selection
    pub end: Option<LogicalPosition>,

    /// Pane where the selection is active
    pub pane: Option<Pane>,
}

impl VisualSelectionState {
    /// Create a new empty visual selection
    pub fn new() -> Self {
        Self {
            start: None,
            end: None,
            pane: None,
        }
    }

    /// Create a visual selection with start, end, and pane
    pub fn with_range(start: LogicalPosition, end: LogicalPosition, pane: Pane) -> Self {
        Self {
            start: Some(start),
            end: Some(end),
            pane: Some(pane),
        }
    }

    /// Check if the selection is active (has both start and end)
    pub fn is_active(&self) -> bool {
        self.start.is_some() && self.end.is_some()
    }

    /// Clear the visual selection
    pub fn clear(&mut self) {
        self.start = None;
        self.end = None;
        self.pane = None;
    }

    /// Get the selection as a tuple for backward compatibility
    #[allow(clippy::type_complexity)]
    pub fn as_tuple(
        &self,
    ) -> (
        Option<LogicalPosition>,
        Option<LogicalPosition>,
        Option<Pane>,
    ) {
        (self.start, self.end, self.pane)
    }

    /// Create from tuple for backward compatibility
    #[allow(clippy::type_complexity)]
    pub fn from_tuple(
        tuple: (
            Option<LogicalPosition>,
            Option<LogicalPosition>,
            Option<Pane>,
        ),
    ) -> Self {
        Self {
            start: tuple.0,
            end: tuple.1,
            pane: tuple.2,
        }
    }
}

impl Default for VisualSelectionState {
    fn default() -> Self {
        Self::new()
    }
}

/// Yank selection containing text and its yank type
///
/// Replaces the complex type alias:
/// `type SelectionWithType = (String, YankType)`
///
/// Provides clear field names for yank operations with type information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YankSelection {
    /// The selected text content
    pub text: String,

    /// Type of yank operation (Character, Line, or Block)
    pub yank_type: YankType,
}

impl YankSelection {
    /// Create a new yank selection
    pub fn new(text: String, yank_type: YankType) -> Self {
        Self { text, yank_type }
    }

    /// Create a character-wise yank selection
    pub fn character(text: String) -> Self {
        Self::new(text, YankType::Character)
    }

    /// Create a line-wise yank selection
    pub fn line(text: String) -> Self {
        Self::new(text, YankType::Line)
    }

    /// Create a block-wise yank selection
    pub fn block(text: String) -> Self {
        Self::new(text, YankType::Block)
    }

    /// Get the selection as a tuple for backward compatibility
    pub fn as_tuple(&self) -> (String, YankType) {
        (self.text.clone(), self.yank_type)
    }

    /// Create from tuple for backward compatibility
    pub fn from_tuple(tuple: (String, YankType)) -> Self {
        Self {
            text: tuple.0,
            yank_type: tuple.1,
        }
    }
}

/// Display line data for rendering
///
/// Replaces the complex type alias:
/// `type DisplayLineData = (String, Option<usize>, bool, usize, usize)`
///
/// Provides clear field names for display rendering operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayLine {
    /// The text content of the line
    pub content: String,

    /// Line number for display (None for continuation lines)
    pub line_number: Option<usize>,

    /// Whether this is a continuation of a wrapped line
    pub is_continuation: bool,

    /// Logical starting column for this display line
    pub logical_start_col: usize,

    /// Logical line number this display line belongs to
    pub logical_line: usize,
}

impl DisplayLine {
    /// Create a new display line
    pub fn new(
        content: String,
        line_number: Option<usize>,
        is_continuation: bool,
        logical_start_col: usize,
        logical_line: usize,
    ) -> Self {
        Self {
            content,
            line_number,
            is_continuation,
            logical_start_col,
            logical_line,
        }
    }

    /// Create a simple display line (non-continuation)
    pub fn simple(content: String, line_number: usize, logical_line: usize) -> Self {
        Self::new(content, Some(line_number), false, 0, logical_line)
    }

    /// Create a continuation line
    pub fn continuation(content: String, logical_start_col: usize, logical_line: usize) -> Self {
        Self::new(content, None, true, logical_start_col, logical_line)
    }

    /// Get the display line as a tuple for backward compatibility
    #[allow(clippy::type_complexity)]
    pub fn as_tuple(&self) -> (String, Option<usize>, bool, usize, usize) {
        (
            self.content.clone(),
            self.line_number,
            self.is_continuation,
            self.logical_start_col,
            self.logical_line,
        )
    }

    /// Create from tuple for backward compatibility
    #[allow(clippy::type_complexity)]
    pub fn from_tuple(tuple: (String, Option<usize>, bool, usize, usize)) -> Self {
        Self {
            content: tuple.0,
            line_number: tuple.1,
            is_continuation: tuple.2,
            logical_start_col: tuple.3,
            logical_line: tuple.4,
        }
    }
}
