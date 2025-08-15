//! # PaneState Module
//!
//! Contains the PaneState struct and its implementations for managing individual pane state.
//! This includes scrolling, cursor positioning, word navigation, and display cache management.
//!
//! HIGH-LEVEL ARCHITECTURE:
//! PaneState encapsulates all state and operations for a single editor pane:
//! - Manages cursor position in both logical and display coordinates
//! - Handles horizontal and vertical scrolling with bounds checking
//! - Coordinates text operations with DisplayCache for proper wrapping
//! - Maintains editor mode state and line number width calculations
//!
//! CORE RESPONSIBILITIES:
//! 1. Cursor Management: Tracks logical position and converts to display coordinates
//! 2. Scroll Coordination: Maintains viewport position and cursor visibility
//! 3. Text Operations: Handles character insertion/deletion with proper event emission
//! 4. Display Integration: Works with DisplayCache for text wrapping and rendering
//!
//! CRITICAL ARCHITECTURAL DECISION:
//! PaneState eliminates feature envy by keeping all pane-specific logic centralized.
//! Previously scattered across multiple classes, this consolidation improves maintainability
//! and follows the Single Responsibility Principle.

use crate::repl::events::{
    EditorMode, LogicalPosition, Pane, PaneCapabilities,
};
use crate::repl::geometry::{Dimensions, Position};
use crate::repl::models::{BufferModel, DisplayCache};
use std::ops::{Index, IndexMut};

// Re-export all modules
pub mod capabilities;
pub mod cursor_basic;
pub mod cursor_line;
pub mod display;
pub mod scrolling;
pub mod text_operations;
pub mod visual_selection;
pub mod word_navigation;

// Re-export key types for external use

/// Minimum width for line number column as specified in requirements
const MIN_LINE_NUMBER_WIDTH: usize = 3;

/// Information about a wrapped line segment
#[derive(Debug, Clone)]
struct WrappedSegment {
    #[allow(dead_code)] // Used for debug/display purposes
    content: String,
    logical_start: usize,
    logical_end: usize,
}

/// Type alias for optional position
pub type OptionalPosition = Option<Position>;

/// Result of a scrolling operation, contains information needed for event emission
#[derive(Debug, Clone)]
pub struct ScrollResult {
    pub old_offset: usize,
    pub new_offset: usize,
    pub cursor_moved: bool,
}

/// Result of a cursor movement operation, contains information needed for event emission
#[derive(Debug, Clone)]
pub struct CursorMoveResult {
    pub cursor_moved: bool,
    pub old_display_pos: Position,
    pub new_display_pos: Position,
}

/// Result of a scroll adjustment for cursor visibility
#[derive(Debug, Clone)]
pub struct ScrollAdjustResult {
    pub vertical_changed: bool,
    pub horizontal_changed: bool,
    pub old_vertical_offset: usize,
    pub new_vertical_offset: usize,
    pub old_horizontal_offset: usize,
    pub new_horizontal_offset: usize,
}

/// State container for a single pane (Request or Response)
///
/// HIGH-LEVEL DESIGN:
/// This struct aggregates all state needed for a single editor pane:
/// - BufferModel: Contains the actual text content and logical operations
/// - DisplayCache: Handles text wrapping and display line calculations  
/// - Position tracking: Maintains both logical and display cursor coordinates
/// - Scroll management: Tracks viewport offset for large content navigation
/// - Visual selection: Supports Vim-style visual mode selections
/// - Mode state: Each pane maintains its own editor mode independently
#[derive(Debug, Clone)]
pub struct PaneState {
    pub buffer: BufferModel,
    pub display_cache: DisplayCache,
    pub display_cursor: Position, // (display_line, display_column)
    pub scroll_offset: Position,  // (vertical, horizontal)
    pub visual_selection_start: Option<LogicalPosition>,
    pub visual_selection_end: Option<LogicalPosition>,
    pub pane_dimensions: Dimensions,    // (width, height)
    pub editor_mode: EditorMode,        // Current editor mode for this pane
    pub line_number_width: usize,       // Width needed for line numbers display
    pub virtual_column: usize,          // Vim-style virtual column - desired cursor position
    pub capabilities: PaneCapabilities, // What operations are allowed on this pane
}

impl PaneState {
    pub fn new(
        pane: Pane,
        pane_width: usize,
        pane_height: usize,
        wrap_enabled: bool,
        capabilities: PaneCapabilities,
    ) -> Self {
        let mut pane_state = Self {
            buffer: BufferModel::new(pane),
            display_cache: DisplayCache::new(),
            display_cursor: Position::origin(),
            scroll_offset: Position::origin(),
            visual_selection_start: None,
            visual_selection_end: None,
            pane_dimensions: Dimensions::new(pane_width, pane_height),
            editor_mode: EditorMode::Normal, // Start in Normal mode
            line_number_width: MIN_LINE_NUMBER_WIDTH, // Start with minimum width
            virtual_column: 0,               // Start at column 0
            capabilities,                    // Set capabilities based on pane type
        };
        pane_state.build_display_cache(pane_width, wrap_enabled, 4); // Default tab width, will be updated later
                                                                     // Calculate initial line number width based on content
        pane_state.update_line_number_width();
        pane_state
    }
}

/// Array indexing for panes to enable clean access patterns
impl Index<Pane> for [PaneState; 2] {
    type Output = PaneState;
    fn index(&self, pane: Pane) -> &Self::Output {
        match pane {
            Pane::Request => &self[0],
            Pane::Response => &self[1],
        }
    }
}

impl IndexMut<Pane> for [PaneState; 2] {
    fn index_mut(&mut self, pane: Pane) -> &mut Self::Output {
        match pane {
            Pane::Request => &mut self[0],
            Pane::Response => &mut self[1],
        }
    }
}