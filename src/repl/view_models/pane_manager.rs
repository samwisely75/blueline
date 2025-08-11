//! # Pane Management
//!
//! Handles pane switching, mode changes, and pane-related state management.
//! Contains the PaneManager struct that encapsulates all pane-related operations.
//!
//! HIGH-LEVEL ARCHITECTURE:
//! PaneManager implements the Manager pattern to encapsulate all pane-related operations:
//! - Manages Request and Response panes as an array with semantic operations
//! - Provides high-level pane switching without exposing internal array indices
//! - Handles terminal dimension updates and pane layout calculations
//! - Coordinates cursor management, scrolling, and text operations across panes
//!
//! CORE RESPONSIBILITIES:
//! 1. Pane State Management: Maintains mode, cursor position, scroll state for each pane
//! 2. Layout Calculation: Computes pane dimensions based on terminal size
//! 3. Semantic Operations: Provides request/response-specific operations without array access
//! 4. Event Coordination: Emits ViewEvents for selective rendering optimizations

use crate::repl::events::{EditorMode, LogicalPosition, LogicalRange, Pane, ViewEvent};
use crate::repl::geometry::Position;
use crate::repl::view_models::pane_state::PaneState;

/// Type alias for visual selection state to reduce complexity
type VisualSelectionState = (
    Option<LogicalPosition>,
    Option<LogicalPosition>,
    Option<Pane>,
);

/// PaneManager encapsulates all pane-related state and operations
/// This eliminates the need for array indexing operations throughout the codebase
///
/// HIGH-LEVEL DESIGN PATTERN:
/// Implements encapsulation by hiding the panes array and providing semantic operations.
/// All external access goes through method calls that handle array indexing internally,
/// improving type safety and preventing index-related bugs throughout the application.
#[derive(Debug)]
pub struct PaneManager {
    panes: [PaneState; 2], // Private - no external access
    current_pane: Pane,
    wrap_enabled: bool,
    pub terminal_dimensions: (u16, u16), // Public for ViewModel access
    request_pane_height: u16,
}

impl PaneManager {
    /// Create a new PaneManager with default state
    ///
    /// HIGH-LEVEL INITIALIZATION:
    /// Sets up the two-pane layout with calculated dimensions:
    /// 1. Computes content width accounting for line numbers (4 chars)
    /// 2. Splits terminal height between request and response panes
    /// 3. Reserves space for separator and status bar
    /// 4. Initializes both panes with proper display caches
    pub fn new(terminal_dimensions: (u16, u16)) -> Self {
        // Build initial display caches
        let content_width = (terminal_dimensions.0 as usize).saturating_sub(4); // Account for line numbers

        // Calculate pane heights
        let request_pane_height = (terminal_dimensions.1 / 2) as usize;
        let response_pane_height = (terminal_dimensions.1 as usize)
            .saturating_sub((terminal_dimensions.1 / 2) as usize)
            .saturating_sub(2) // -2 for separator and status
            .max(1); // Ensure minimum height of 1

        // Initialize pane array with proper display caches and dimensions
        let request_pane = PaneState::new(Pane::Request, content_width, request_pane_height, true);
        let response_pane =
            PaneState::new(Pane::Response, content_width, response_pane_height, true);

        Self {
            panes: [request_pane, response_pane],
            current_pane: Pane::Request,
            wrap_enabled: false,
            terminal_dimensions,
            request_pane_height: terminal_dimensions.1 / 2,
        }
    }

    /// Get current active pane type
    pub fn current_pane_type(&self) -> Pane {
        self.current_pane
    }

    /// Switch to other area (semantic operation - no pane exposure)
    pub fn switch_to_other_area(&mut self) -> Vec<ViewEvent> {
        let old_pane = self.current_pane;
        self.current_pane = match self.current_pane {
            Pane::Request => Pane::Response,
            Pane::Response => Pane::Request,
        };

        if old_pane != self.current_pane {
            vec![
                ViewEvent::FocusSwitched,
                ViewEvent::StatusBarUpdateRequired,
                ViewEvent::ActiveCursorUpdateRequired,
            ]
        } else {
            vec![]
        }
    }

    /// Switch to Request pane
    pub fn switch_to_request_pane(&mut self) -> Vec<ViewEvent> {
        if self.current_pane != Pane::Request {
            self.current_pane = Pane::Request;
            vec![
                ViewEvent::FocusSwitched,
                ViewEvent::StatusBarUpdateRequired,
                ViewEvent::ActiveCursorUpdateRequired,
            ]
        } else {
            vec![]
        }
    }

    /// Switch to Response pane
    pub fn switch_to_response_pane(&mut self) -> Vec<ViewEvent> {
        if self.current_pane != Pane::Response {
            self.current_pane = Pane::Response;
            vec![
                ViewEvent::FocusSwitched,
                ViewEvent::StatusBarUpdateRequired,
                ViewEvent::ActiveCursorUpdateRequired,
            ]
        } else {
            vec![]
        }
    }

    /// Check if currently in Request pane
    pub fn is_in_request_pane(&self) -> bool {
        self.current_pane == Pane::Request
    }

    /// Check if currently in Response pane
    pub fn is_in_response_pane(&self) -> bool {
        self.current_pane == Pane::Response
    }

    /// Get current cursor position (no indexing exposed)
    pub fn get_current_cursor_position(&self) -> LogicalPosition {
        self.panes[self.current_pane].buffer.cursor()
    }

    /// Get visual selection state for current pane
    pub fn get_visual_selection(&self) -> VisualSelectionState {
        let current_pane_state = &self.panes[self.current_pane];
        (
            current_pane_state.visual_selection_start,
            current_pane_state.visual_selection_end,
            if current_pane_state.visual_selection_start.is_some() {
                Some(self.current_pane)
            } else {
                None
            },
        )
    }

    /// Check if a position is within visual selection
    pub fn is_position_selected(&self, position: LogicalPosition, pane: Pane) -> bool {
        let pane_state = &self.panes[pane];
        if let (Some(start), Some(end)) = (
            pane_state.visual_selection_start,
            pane_state.visual_selection_end,
        ) {
            // Normalize selection range (start <= end)
            let (normalized_start, normalized_end) = if start.line < end.line
                || (start.line == end.line && start.column <= end.column)
            {
                (start, end)
            } else {
                (end, start)
            };

            tracing::trace!(
                "is_position_selected: checking position={:?} against selection start={:?} end={:?} (normalized: start={:?} end={:?})", 
                position, start, end, normalized_start, normalized_end
            );

            // Check if position is within selection range
            if position.line < normalized_start.line || position.line > normalized_end.line {
                tracing::trace!("is_position_selected: position outside line range");
                return false;
            }

            if position.line == normalized_start.line && position.line == normalized_end.line {
                // Single line selection
                let is_selected = position.column >= normalized_start.column
                    && position.column <= normalized_end.column;
                tracing::trace!(
                    "is_position_selected: single line selection, result={}",
                    is_selected
                );
                return is_selected;
            }

            if position.line == normalized_start.line {
                // First line of multi-line selection
                let is_selected = position.column >= normalized_start.column;
                tracing::trace!(
                    "is_position_selected: first line of multi-line selection, result={}",
                    is_selected
                );
                return is_selected;
            }

            if position.line == normalized_end.line {
                // Last line of multi-line selection
                let is_selected = position.column <= normalized_end.column;
                tracing::trace!(
                    "is_position_selected: last line of multi-line selection, result={}",
                    is_selected
                );
                return is_selected;
            }

            // Middle line of multi-line selection
            tracing::trace!(
                "is_position_selected: middle line of multi-line selection, result=true"
            );
            return true;
        }
        tracing::trace!("is_position_selected: no visual selection active");
        false
    }

    /// Start visual selection in current area
    pub fn start_visual_selection(&mut self) -> Vec<ViewEvent> {
        let current_cursor = self.get_current_cursor_position();
        let current_pane_state = &mut self.panes[self.current_pane];

        current_pane_state.visual_selection_start = Some(current_cursor);
        current_pane_state.visual_selection_end = Some(current_cursor);

        tracing::info!(
            "Entered visual mode, selection starts at {:?}",
            current_cursor
        );

        vec![
            ViewEvent::StatusBarUpdateRequired,
            ViewEvent::ActiveCursorUpdateRequired,
        ]
    }

    /// End visual selection in current area
    pub fn end_visual_selection(&mut self) -> Vec<ViewEvent> {
        let current_pane_state = &mut self.panes[self.current_pane];
        current_pane_state.visual_selection_start = None;
        current_pane_state.visual_selection_end = None;

        tracing::info!("Exited visual mode, cleared selection state");

        vec![
            ViewEvent::CurrentAreaRedrawRequired,
            ViewEvent::StatusBarUpdateRequired,
            ViewEvent::ActiveCursorUpdateRequired,
        ]
    }

    /// Update visual selection end position
    pub fn update_visual_selection(&mut self, position: LogicalPosition) -> Vec<ViewEvent> {
        let current_pane_state = &mut self.panes[self.current_pane];
        if current_pane_state.visual_selection_start.is_some() {
            current_pane_state.visual_selection_end = Some(position);
            vec![ViewEvent::CurrentAreaRedrawRequired]
        } else {
            vec![]
        }
    }

    /// Get request pane height
    pub fn request_pane_height(&self) -> u16 {
        self.request_pane_height
    }

    /// Get response pane height
    pub fn response_pane_height(&self, has_response: bool) -> u16 {
        if has_response {
            self.terminal_dimensions
                .1
                .saturating_sub(self.request_pane_height)
                .saturating_sub(2) // -2 for separator and status
        } else {
            0
        }
    }

    /// Get word wrap enabled state
    pub fn is_wrap_enabled(&self) -> bool {
        self.wrap_enabled
    }

    /// Set word wrap enabled state
    pub fn set_wrap_enabled(&mut self, enabled: bool) {
        tracing::debug!(
            "🔧 PaneManager::set_wrap_enabled: changing from {} to {}",
            self.wrap_enabled,
            enabled
        );
        self.wrap_enabled = enabled;
        tracing::debug!(
            "✅ PaneManager::set_wrap_enabled: wrap_enabled is now {}",
            self.wrap_enabled
        );
    }

    /// Update terminal size and recalculate pane dimensions
    pub fn update_terminal_size(&mut self, width: u16, height: u16, has_response: bool) {
        self.terminal_dimensions = (width, height);

        // Calculate request pane height (split screen when response exists)
        self.request_pane_height = if has_response {
            height / 2
        } else {
            height - 1 // Reserve space for status bar
        };

        // Recalculate pane dimensions
        let content_width = (width as usize).saturating_sub(4); // Account for line numbers
        let request_pane_height = self.request_pane_height as usize;
        let response_pane_height = (height as usize)
            .saturating_sub(self.request_pane_height as usize)
            .saturating_sub(2) // -2 for separator and status
            .max(1); // Ensure minimum height of 1

        // Update pane dimensions
        self.panes[Pane::Request].update_dimensions(content_width, request_pane_height);
        self.panes[Pane::Response].update_dimensions(content_width, response_pane_height);

        // Invalidate and rebuild display caches for both panes
        // CRITICAL FIX: After invalidating caches, we must rebuild them immediately
        // Otherwise rendering will show empty panes when caches are invalid
        self.panes[Pane::Request].display_cache.invalidate();
        self.panes[Pane::Response].display_cache.invalidate();

        // Rebuild both caches with the new dimensions
        self.panes[Pane::Request].build_display_cache(content_width, self.wrap_enabled);
        self.panes[Pane::Response].build_display_cache(content_width, self.wrap_enabled);

        tracing::debug!(
            "Terminal size updated: {}x{}, pane dimensions: Request={}x{}, Response={}x{}",
            width,
            height,
            content_width,
            request_pane_height,
            content_width,
            response_pane_height
        );
    }

    /// Rebuild display caches for both panes with provided content width
    pub fn rebuild_display_caches(&mut self, content_width: usize) {
        self.panes[Pane::Request].build_display_cache(content_width, self.wrap_enabled);
        self.panes[Pane::Response].build_display_cache(content_width, self.wrap_enabled);
    }

    /// Rebuild display caches for both panes and sync cursors (complete rebuild process)
    pub fn rebuild_display_caches_and_sync(&mut self) -> Vec<ViewEvent> {
        tracing::debug!(
            "🔄 PaneManager::rebuild_display_caches_and_sync: starting with wrap_enabled={}",
            self.wrap_enabled
        );
        let content_width = self.get_content_width();

        // Rebuild display caches
        self.rebuild_display_caches(content_width);

        // Sync display cursors to ensure they're still valid after cache rebuild
        self.sync_display_cursors();

        // Ensure current cursor is visible after potential layout changes

        self.ensure_current_cursor_visible(content_width)
    }

    /// Sync display cursors for both panes
    pub fn sync_display_cursors(&mut self) {
        for pane in [Pane::Request, Pane::Response] {
            let logical = self.panes[pane].buffer.cursor();
            if let Some(display_pos) = self.panes[pane]
                .display_cache
                .logical_to_display_position(logical.line, logical.column)
            {
                self.panes[pane].display_cursor = display_pos;
            } else {
                // BUGFIX Issue #89: If logical_to_display_position fails, ensure cursor tracking doesn't break
                tracing::warn!(
                    "sync_display_cursors: logical_to_display_position failed for {:?} pane at {:?} - using fallback", 
                    pane, logical
                );
                // Fallback: Use logical position as display position (works for non-wrapped content)
                self.panes[pane].display_cursor = Position::new(logical.line, logical.column);
            }
        }
    }

    /// Get display cursor position for current pane
    pub fn get_current_display_cursor(&self) -> Position {
        self.panes[self.current_pane].display_cursor
    }

    /// Get scroll offset for current pane
    pub fn get_current_scroll_offset(&self) -> Position {
        self.panes[self.current_pane].scroll_offset
    }

    /// Ensure cursor is visible in current area
    pub fn ensure_current_cursor_visible(&mut self, content_width: usize) -> Vec<ViewEvent> {
        let result = self.panes[self.current_pane].ensure_cursor_visible(content_width);

        if result.vertical_changed || result.horizontal_changed {
            // For horizontal scrolling, use horizontal offsets; for vertical scrolling, use vertical offsets
            // If both changed, prioritize horizontal since it's more common in response navigation
            let (old_offset, new_offset) = if result.horizontal_changed {
                (result.old_horizontal_offset, result.new_horizontal_offset)
            } else {
                (result.old_vertical_offset, result.new_vertical_offset)
            };

            vec![ViewEvent::CurrentAreaScrollChanged {
                old_offset,
                new_offset,
            }]
        } else {
            vec![]
        }
    }

    /// Get text content for current pane
    pub fn get_current_text(&self) -> String {
        self.panes[self.current_pane]
            .buffer
            .content()
            .lines()
            .join("\n")
    }

    /// Get text content for request pane
    pub fn get_request_text(&self) -> String {
        self.panes[Pane::Request]
            .buffer
            .content()
            .lines()
            .join("\n")
    }

    /// Get text content for response pane
    pub fn get_response_text(&self) -> String {
        self.panes[Pane::Response]
            .buffer
            .content()
            .lines()
            .join("\n")
    }

    /// Insert character in Request pane content
    pub fn insert_char_in_request(&mut self, ch: char) -> Vec<ViewEvent> {
        if self.is_in_request_pane() {
            let _event = self.panes[Pane::Request].buffer.insert_char(ch);

            // Rebuild display cache to ensure rendering sees the updated content
            let content_width = self.get_content_width();
            self.panes[Pane::Request].build_display_cache(content_width, self.wrap_enabled);

            // Sync display cursor after cache rebuild
            let logical = self.panes[Pane::Request].buffer.cursor();
            if let Some(display_pos) = self.panes[Pane::Request]
                .display_cache
                .logical_to_display_position(logical.line, logical.column)
            {
                self.panes[Pane::Request].display_cursor = display_pos;
            } else {
                // BUGFIX Issue #89: If logical_to_display_position fails, ensure cursor tracking doesn't break
                // This can happen with empty lines or edge cases after multiple newlines in Insert mode
                tracing::warn!(
                    "logical_to_display_position failed for cursor at {:?} - using fallback display position", 
                    logical
                );
                // Fallback: Use logical position as display position (works for non-wrapped content)
                self.panes[Pane::Request].display_cursor =
                    Position::new(logical.line, logical.column);
            }

            let mut events = vec![
                ViewEvent::RequestContentChanged,
                ViewEvent::ActiveCursorUpdateRequired,
                ViewEvent::PositionIndicatorUpdateRequired,
            ];

            // Ensure cursor is visible after insertion
            let visibility_events = self.ensure_current_cursor_visible(content_width);
            events.extend(visibility_events);

            events
        } else {
            vec![] // Can't edit in display area
        }
    }

    /// Delete character before cursor in Request pane
    pub fn delete_char_before_cursor_in_request(&mut self) -> Vec<ViewEvent> {
        tracing::debug!("🗑️  PaneManager::delete_char_before_cursor_in_request called");

        if !self.is_in_request_pane() {
            tracing::debug!("🗑️  Not in request pane, skipping deletion");
            return vec![]; // Can't edit in display area
        }

        tracing::debug!("🗑️  In request pane, performing actual deletion");

        let request_pane = &mut self.panes[Pane::Request];
        let current_cursor = request_pane.buffer.cursor();

        tracing::debug!("🗑️  Current cursor position: {:?}", current_cursor);

        if current_cursor.column > 0 {
            // Delete character before cursor in the same line
            tracing::debug!("🗑️  Deleting character before cursor in same line");

            let delete_start = LogicalPosition::new(current_cursor.line, current_cursor.column - 1);
            let delete_end = LogicalPosition::new(current_cursor.line, current_cursor.column);
            let delete_range = LogicalRange::new(delete_start, delete_end);

            // Use the existing delete_range method
            if let Some(_event) = request_pane
                .buffer
                .content_mut()
                .delete_range(self.current_pane, delete_range)
            {
                // Move cursor left
                let new_cursor =
                    LogicalPosition::new(current_cursor.line, current_cursor.column - 1);
                request_pane.buffer.set_cursor(new_cursor);

                tracing::debug!(
                    "🗑️  Deleted character in line, new cursor: {:?}",
                    new_cursor
                );

                // Rebuild display cache since content changed
                let content_width = (self.terminal_dimensions.0 as usize).saturating_sub(4);
                request_pane.build_display_cache(content_width, self.wrap_enabled);

                // Sync display cursor with new logical position after cache rebuild
                if let Some(display_pos) = request_pane
                    .display_cache
                    .logical_to_display_position(new_cursor.line, new_cursor.column)
                {
                    request_pane.display_cursor = display_pos;
                } else {
                    // BUGFIX Issue #89: If logical_to_display_position fails, ensure cursor tracking doesn't break
                    tracing::warn!(
                        "delete_char_before_cursor: logical_to_display_position failed at {:?} - using fallback", 
                        new_cursor
                    );
                    // Fallback: Use logical position as display position (works for non-wrapped content)
                    request_pane.display_cursor = Position::new(new_cursor.line, new_cursor.column);
                }

                let mut events = vec![
                    ViewEvent::RequestContentChanged,
                    ViewEvent::ActiveCursorUpdateRequired,
                    ViewEvent::CurrentAreaRedrawRequired,
                ];

                // Ensure cursor is visible after deletion
                let content_width = self.get_content_width();
                let visibility_events = self.ensure_current_cursor_visible(content_width);
                events.extend(visibility_events);

                return events;
            }
        } else if current_cursor.line > 0 {
            // At beginning of line, join with previous line (backspace at line start)
            tracing::debug!("🗑️  At line start, joining with previous line");

            // Get length of previous line to position cursor correctly
            let prev_line_length = if let Some(prev_line) = request_pane
                .buffer
                .content()
                .get_line(current_cursor.line - 1)
            {
                prev_line.len()
            } else {
                0
            };

            // Create range to delete the newline character (join lines)
            // We delete from end of previous line to start of current line
            let delete_start = LogicalPosition::new(current_cursor.line - 1, prev_line_length);
            let delete_end = LogicalPosition::new(current_cursor.line, 0);
            let delete_range = LogicalRange::new(delete_start, delete_end);

            // Use the existing delete_range method
            if let Some(_event) = request_pane
                .buffer
                .content_mut()
                .delete_range(self.current_pane, delete_range)
            {
                // Move cursor to end of previous line (where the join happened)
                let new_cursor = LogicalPosition::new(current_cursor.line - 1, prev_line_length);
                request_pane.buffer.set_cursor(new_cursor);

                tracing::debug!("🗑️  Joined lines, new cursor: {:?}", new_cursor);

                // Rebuild display cache since content structure changed
                let content_width = (self.terminal_dimensions.0 as usize).saturating_sub(4);
                request_pane.build_display_cache(content_width, self.wrap_enabled);

                // Sync display cursor with new logical position after cache rebuild
                if let Some(display_pos) = request_pane
                    .display_cache
                    .logical_to_display_position(new_cursor.line, new_cursor.column)
                {
                    request_pane.display_cursor = display_pos;
                } else {
                    // BUGFIX Issue #89: If logical_to_display_position fails, ensure cursor tracking doesn't break
                    tracing::warn!(
                        "delete_char_before_cursor: logical_to_display_position failed at {:?} - using fallback", 
                        new_cursor
                    );
                    // Fallback: Use logical position as display position (works for non-wrapped content)
                    request_pane.display_cursor = Position::new(new_cursor.line, new_cursor.column);
                }

                return vec![
                    ViewEvent::RequestContentChanged,
                    ViewEvent::ActiveCursorUpdateRequired,
                    ViewEvent::CurrentAreaRedrawRequired,
                ];
            }
        }

        tracing::debug!("🗑️  No deletion performed - at start of buffer or invalid state");
        vec![] // Nothing to delete (at start of first line)
    }

    /// Delete character after cursor in Request pane
    pub fn delete_char_after_cursor_in_request(&mut self) -> Vec<ViewEvent> {
        tracing::debug!("🗑️  PaneManager::delete_char_after_cursor_in_request called");

        if !self.is_in_request_pane() {
            tracing::debug!("🗑️  Not in request pane, skipping deletion");
            return vec![]; // Can't edit in display area
        }

        tracing::debug!("🗑️  In request pane, performing actual deletion");

        let request_pane = &mut self.panes[Pane::Request];
        let current_cursor = request_pane.buffer.cursor();

        tracing::debug!("🗑️  Current cursor position: {:?}", current_cursor);

        // Get current line to check if we can delete within the line
        if let Some(current_line) = request_pane.buffer.content().get_line(current_cursor.line) {
            if current_cursor.column < current_line.len() {
                // Delete character at cursor position (same line)
                tracing::debug!("🗑️  Deleting character at cursor position in same line");

                let delete_start = LogicalPosition::new(current_cursor.line, current_cursor.column);
                let delete_end =
                    LogicalPosition::new(current_cursor.line, current_cursor.column + 1);
                let delete_range = LogicalRange::new(delete_start, delete_end);

                // Use the existing delete_range method
                if let Some(_event) = request_pane
                    .buffer
                    .content_mut()
                    .delete_range(self.current_pane, delete_range)
                {
                    // Cursor stays in same position
                    tracing::debug!("🗑️  Deleted character at cursor, cursor position unchanged");

                    // Rebuild display cache since content changed
                    let content_width = (self.terminal_dimensions.0 as usize).saturating_sub(4);
                    request_pane.build_display_cache(content_width, self.wrap_enabled);

                    // Sync display cursor with logical position after cache rebuild
                    let current_logical = request_pane.buffer.cursor();
                    if let Some(display_pos) = request_pane
                        .display_cache
                        .logical_to_display_position(current_logical.line, current_logical.column)
                    {
                        request_pane.display_cursor = display_pos;
                    } else {
                        // BUGFIX Issue #89: If logical_to_display_position fails, ensure cursor tracking doesn't break
                        tracing::warn!(
                            "delete_char_after_cursor: logical_to_display_position failed at {:?} - using fallback", 
                            current_logical
                        );
                        // Fallback: Use logical position as display position (works for non-wrapped content)
                        request_pane.display_cursor =
                            Position::new(current_logical.line, current_logical.column);
                    }

                    return vec![
                        ViewEvent::RequestContentChanged,
                        ViewEvent::ActiveCursorUpdateRequired,
                        ViewEvent::CurrentAreaRedrawRequired,
                    ];
                }
            } else if current_cursor.line + 1 < request_pane.buffer.content().line_count() {
                // At end of line, join with next line (delete key at line end)
                tracing::debug!("🗑️  At line end, joining with next line");

                // Create range to delete the newline character (join lines)
                // We delete from cursor position to start of next line
                let delete_start = LogicalPosition::new(current_cursor.line, current_cursor.column);
                let delete_end = LogicalPosition::new(current_cursor.line + 1, 0);
                let delete_range = LogicalRange::new(delete_start, delete_end);

                // Use the existing delete_range method
                if let Some(_event) = request_pane
                    .buffer
                    .content_mut()
                    .delete_range(self.current_pane, delete_range)
                {
                    // Cursor stays at current position (end of merged line)
                    tracing::debug!("🗑️  Joined lines, cursor position unchanged");

                    // Rebuild display cache since content structure changed
                    let content_width = (self.terminal_dimensions.0 as usize).saturating_sub(4);
                    request_pane.build_display_cache(content_width, self.wrap_enabled);

                    // Sync display cursor with logical position after cache rebuild
                    let current_logical = request_pane.buffer.cursor();
                    if let Some(display_pos) = request_pane
                        .display_cache
                        .logical_to_display_position(current_logical.line, current_logical.column)
                    {
                        request_pane.display_cursor = display_pos;
                    } else {
                        // BUGFIX Issue #89: If logical_to_display_position fails, ensure cursor tracking doesn't break
                        tracing::warn!(
                            "delete_char_after_cursor: logical_to_display_position failed at {:?} - using fallback", 
                            current_logical
                        );
                        // Fallback: Use logical position as display position (works for non-wrapped content)
                        request_pane.display_cursor =
                            Position::new(current_logical.line, current_logical.column);
                    }

                    return vec![
                        ViewEvent::RequestContentChanged,
                        ViewEvent::ActiveCursorUpdateRequired,
                        ViewEvent::CurrentAreaRedrawRequired,
                    ];
                }
            }
        }

        tracing::debug!("🗑️  No deletion performed - at end of buffer or invalid state");
        vec![] // Nothing to delete (at end of buffer)
    }

    /// Set cursor position in current area
    pub fn set_current_cursor_position(&mut self, position: LogicalPosition) -> Vec<ViewEvent> {
        let clamped_position = self.panes[self.current_pane]
            .buffer
            .content()
            .clamp_position(position);
        self.panes[self.current_pane]
            .buffer
            .set_cursor(clamped_position);

        // Sync display cursor with new logical position
        if let Some(display_pos) = self.panes[self.current_pane]
            .display_cache
            .logical_to_display_position(clamped_position.line, clamped_position.column)
        {
            self.panes[self.current_pane].display_cursor = display_pos;
        } else {
            // BUGFIX Issue #89: If logical_to_display_position fails, ensure cursor tracking doesn't break
            tracing::warn!(
                "set_current_cursor_position: logical_to_display_position failed at {:?} - using fallback", 
                clamped_position
            );
            // Fallback: Use logical position as display position (works for non-wrapped content)
            self.panes[self.current_pane].display_cursor =
                Position::new(clamped_position.line, clamped_position.column);
        }

        // Update visual selection if active
        if self.panes[self.current_pane]
            .visual_selection_start
            .is_some()
        {
            self.panes[self.current_pane].visual_selection_end = Some(clamped_position);
        }

        let mut events = vec![
            ViewEvent::ActiveCursorUpdateRequired,
            ViewEvent::PositionIndicatorUpdateRequired,
        ];

        // Ensure cursor is visible and add visibility events
        let content_width = self.get_content_width();
        let visibility_events = self.ensure_current_cursor_visible(content_width);
        events.extend(visibility_events);

        events
    }

    /// Clear editable content (semantic operation)
    pub fn clear_editable_content(&mut self) -> Vec<ViewEvent> {
        self.panes[Pane::Request].buffer = crate::repl::models::BufferModel::new(Pane::Request);
        vec![ViewEvent::RequestContentChanged]
    }

    /// Set Request pane content
    pub fn set_request_content(&mut self, text: &str) -> Vec<ViewEvent> {
        self.panes[Pane::Request].buffer = crate::repl::models::BufferModel::new(Pane::Request);
        self.panes[Pane::Request]
            .buffer
            .content_mut()
            .set_text(text);

        // Update line number width after content changes
        self.panes[Pane::Request].update_line_number_width();

        vec![ViewEvent::RequestContentChanged]
    }

    /// Set Response pane content
    pub fn set_response_content(&mut self, text: &str) -> Vec<ViewEvent> {
        self.panes[Pane::Response].buffer = crate::repl::models::BufferModel::new(Pane::Response);

        self.panes[Pane::Response]
            .buffer
            .content_mut()
            .set_text(text);

        // Update line number width after content changes
        self.panes[Pane::Response].update_line_number_width();

        // Reset cursor and scroll positions to avoid out-of-bounds issues
        self.panes[Pane::Response].display_cursor = Position::origin();
        self.panes[Pane::Response].scroll_offset = Position::origin();

        // Clear any visual selection in the response pane
        self.panes[Pane::Response].visual_selection_start = None;
        self.panes[Pane::Response].visual_selection_end = None;

        // Rebuild display cache to ensure rendering sees the updated content
        let content_width = (self.terminal_dimensions.0 as usize).saturating_sub(4); // Same as Request pane
        self.panes[Pane::Response].build_display_cache(content_width, self.wrap_enabled);

        vec![ViewEvent::ResponseContentChanged]
    }

    /// Get display cache for current pane
    pub fn get_current_display_cache(&self) -> &crate::repl::models::DisplayCache {
        &self.panes[self.current_pane].display_cache
    }

    /// Get display cache for specific pane (rare usage)
    pub fn get_display_cache(&self, pane: Pane) -> &crate::repl::models::DisplayCache {
        &self.panes[pane].display_cache
    }

    /// Get line number width for current pane
    pub fn get_current_line_number_width(&self) -> usize {
        self.panes[self.current_pane].get_line_number_width()
    }

    /// Get line number width for specific pane
    pub fn get_line_number_width(&self, pane: Pane) -> usize {
        self.panes[pane].get_line_number_width()
    }

    /// Sync display cursor with logical cursor for current pane
    pub fn sync_current_display_cursor_with_logical(&mut self) -> Vec<ViewEvent> {
        let _result = self.panes[self.current_pane].sync_display_cursor_with_logical();
        vec![]
    }

    /// Set display cursor position for current area
    pub fn set_current_display_cursor(&mut self, position: Position) -> Vec<ViewEvent> {
        let _result = self.panes[self.current_pane].set_display_cursor(position);

        let mut events = vec![
            ViewEvent::ActiveCursorUpdateRequired,
            ViewEvent::PositionIndicatorUpdateRequired,
        ];

        // CRITICAL FIX: Update visual selection end if in visual mode (same pattern as other cursor movements)
        if self.panes[self.current_pane]
            .visual_selection_start
            .is_some()
        {
            let new_cursor_pos = self.panes[self.current_pane].buffer.cursor();
            self.panes[self.current_pane].visual_selection_end = Some(new_cursor_pos);
            events.push(ViewEvent::CurrentAreaRedrawRequired); // Redraw for visual selection
            tracing::debug!(
                "Display cursor movement updated visual selection end to {:?}",
                new_cursor_pos
            );
        }

        events
    }

    /// Handle horizontal scrolling in current area
    pub fn scroll_current_horizontally(&mut self, direction: i32, amount: usize) -> Vec<ViewEvent> {
        let result = self.panes[self.current_pane].scroll_horizontally(direction, amount);

        let mut events = vec![ViewEvent::CurrentAreaScrollChanged {
            old_offset: result.old_offset,
            new_offset: result.new_offset,
        }];

        if result.cursor_moved {
            events.push(ViewEvent::ActiveCursorUpdateRequired);
        }

        events
    }

    /// Move cursor to next word in current pane
    pub fn move_cursor_to_next_word(&mut self) -> Vec<ViewEvent> {
        let current_display_pos = self.get_current_display_cursor();

        if let Some(new_pos) =
            self.panes[self.current_pane].find_next_word_position(current_display_pos)
        {
            let events = self.set_current_display_cursor(new_pos);
            let mut all_events = events;
            all_events.extend(self.ensure_current_cursor_visible(self.get_content_width()));
            all_events
        } else {
            vec![]
        }
    }

    /// Move cursor to previous word in current pane
    pub fn move_cursor_to_previous_word(&mut self) -> Vec<ViewEvent> {
        let current_display_pos = self.get_current_display_cursor();

        if let Some(new_pos) =
            self.panes[self.current_pane].find_previous_word_position(current_display_pos)
        {
            let events = self.set_current_display_cursor(new_pos);
            let mut all_events = events;
            all_events.extend(self.ensure_current_cursor_visible(self.get_content_width()));
            all_events
        } else {
            vec![]
        }
    }

    /// Move cursor to end of word in current pane
    pub fn move_cursor_to_end_of_word(&mut self) -> Vec<ViewEvent> {
        let current_display_pos = self.get_current_display_cursor();

        if let Some(new_pos) =
            self.panes[self.current_pane].find_end_of_word_position(current_display_pos)
        {
            let events = self.set_current_display_cursor(new_pos);
            let mut all_events = events;
            all_events.extend(self.ensure_current_cursor_visible(self.get_content_width()));
            all_events
        } else {
            vec![]
        }
    }

    /// Get content width for current pane (temporary - will be moved to internal calculation)
    pub fn get_content_width(&self) -> usize {
        // Use current pane's line number width calculation
        // This is a simplified version - should be improved later
        (self.terminal_dimensions.0 as usize).saturating_sub(4)
    }

    /// Move cursor left in current area
    pub fn move_cursor_left(&mut self) -> Vec<ViewEvent> {
        let current_display_pos = self.get_current_display_cursor();
        let display_cache = &self.panes[self.current_pane].display_cache;

        let mut moved = false;

        // Check if we can move left within current display line
        if current_display_pos.col > 0 {
            // Use character-aware left movement
            if let Some(current_line) = display_cache.get_display_line(current_display_pos.row) {
                let new_col = current_line.move_left_by_character(current_display_pos.col);
                let new_display_pos = Position::new(current_display_pos.row, new_col);
                self.panes[self.current_pane].display_cursor = new_display_pos;
                moved = true;
            }
        } else if current_display_pos.row > 0 {
            // Move to end of previous display line
            let prev_display_line = current_display_pos.row - 1;
            if let Some(prev_line) = display_cache.get_display_line(prev_display_line) {
                // FIXED: Use display width instead of character count for proper multibyte character support
                let new_col = prev_line.display_width().saturating_sub(1);
                let new_display_pos = Position::new(prev_display_line, new_col);
                self.panes[self.current_pane].display_cursor = new_display_pos;
                moved = true;
            }
        }

        if moved {
            // Sync logical cursor with new display position
            let new_display_pos = self.get_current_display_cursor();
            if let Some(logical_pos) = self.panes[self.current_pane]
                .display_cache
                .display_to_logical_position(new_display_pos.row, new_display_pos.col)
            {
                let new_logical_pos = LogicalPosition::new(logical_pos.row, logical_pos.col);
                self.panes[self.current_pane]
                    .buffer
                    .set_cursor(new_logical_pos);

                // CRITICAL FIX: Update visual selection if active (similar to set_current_cursor_position)
                if self.panes[self.current_pane]
                    .visual_selection_start
                    .is_some()
                {
                    self.panes[self.current_pane].visual_selection_end = Some(new_logical_pos);
                    tracing::debug!("Updated visual selection end to {:?}", new_logical_pos);
                }
            }

            let mut events = vec![
                ViewEvent::ActiveCursorUpdateRequired,
                ViewEvent::PositionIndicatorUpdateRequired,
                ViewEvent::CurrentAreaRedrawRequired, // Add redraw for visual selection
            ];

            // Ensure cursor is visible and add visibility events
            let content_width = self.get_content_width();
            let visibility_events = self.ensure_current_cursor_visible(content_width);
            events.extend(visibility_events);

            events
        } else {
            vec![]
        }
    }

    /// Move cursor right in current area
    ///
    /// HIGH-LEVEL LOGIC:
    /// 1. Check if cursor can move right within current line (mode-aware boundary check)
    /// 2. If not, check if cursor can move to next line (line wrap navigation)
    /// 3. Perform the actual cursor movement using character-aware positioning
    /// 4. Sync display cursor with logical cursor and update visual selections
    pub fn move_cursor_right(&mut self) -> Vec<ViewEvent> {
        let current_display_pos = self.get_current_display_cursor();

        let mut moved = false;
        let mut new_display_pos = current_display_pos;

        // PHASE 1: Check if cursor can move right within current line
        // Uses mode-aware boundary checking for proper Insert vs Normal behavior
        let can_move_right_in_line = if let Some(current_line) = self.panes[self.current_pane]
            .display_cache
            .get_display_line(current_display_pos.row)
        {
            // MULTIBYTE FIX: Use display width instead of character count for proper CJK support
            // MODE-AWARE: Different boundary behavior for Insert vs Normal mode
            let line_display_width = current_line.display_width();
            let current_mode = self.get_current_pane_mode();

            match current_mode {
                EditorMode::Insert => {
                    // Insert mode: Allow cursor to go one position past end of line (for typing new chars)
                    current_display_pos.col < line_display_width
                }
                _ => {
                    // Normal/Visual mode: Stop at last character position (Vim behavior)
                    if line_display_width == 0 {
                        false // Empty line - no movement allowed
                    } else {
                        // Check if moving right would keep us within the line
                        // We simulate the movement to see if it would go past the end
                        let next_pos =
                            current_line.move_right_by_character(current_display_pos.col);
                        next_pos < line_display_width
                    }
                }
            }
        } else {
            false
        };

        // PHASE 2: Check if cursor can move to next line (when right movement in current line fails)
        let can_move_to_next_line = if !can_move_right_in_line {
            let next_display_line = current_display_pos.row + 1;
            self.panes[self.current_pane]
                .display_cache
                .get_display_line(next_display_line)
                .is_some()
        } else {
            false
        };

        // PHASE 3: Perform the actual cursor movement
        if can_move_right_in_line {
            // CASE 1: Move right within current line using character-aware positioning
            if let Some(current_line) = self.panes[self.current_pane]
                .display_cache
                .get_display_line(current_display_pos.row)
            {
                let new_col = current_line.move_right_by_character(current_display_pos.col);

                // When wrap is enabled, check if we've moved past the visible width
                // If so, wrap to the next line instead of staying on the current line
                let content_width = self.get_content_width();
                if self.wrap_enabled && new_col >= content_width {
                    // Check if there's a next line to wrap to
                    let next_display_line = current_display_pos.row + 1;
                    if self.panes[self.current_pane]
                        .display_cache
                        .get_display_line(next_display_line)
                        .is_some()
                    {
                        new_display_pos = Position::new(next_display_line, 0);
                    } else {
                        // No next line, stay at current position
                        new_display_pos =
                            Position::new(current_display_pos.row, current_display_pos.col);
                        moved = false;
                    }
                } else {
                    new_display_pos = Position::new(current_display_pos.row, new_col);
                }

                if moved || new_display_pos != current_display_pos {
                    self.panes[self.current_pane].display_cursor = new_display_pos;
                    moved = true;
                }
            }
        } else if can_move_to_next_line {
            // CASE 2: Move to beginning of next line (line wrap navigation)
            new_display_pos = Position::new(current_display_pos.row + 1, 0);
            self.panes[self.current_pane].display_cursor = new_display_pos;
            moved = true;
        }

        // PHASE 4: Synchronize cursor position and update related state
        if moved {
            // Sync logical cursor with new display position for buffer operations
            if let Some(logical_pos) = self.panes[self.current_pane]
                .display_cache
                .display_to_logical_position(new_display_pos.row, new_display_pos.col)
            {
                let new_logical_pos = LogicalPosition::new(logical_pos.row, logical_pos.col);
                self.panes[self.current_pane]
                    .buffer
                    .set_cursor(new_logical_pos);

                // VISUAL MODE: Update visual selection end point if in visual mode
                if self.panes[self.current_pane]
                    .visual_selection_start
                    .is_some()
                {
                    self.panes[self.current_pane].visual_selection_end = Some(new_logical_pos);
                    tracing::debug!("Updated visual selection end to {:?}", new_logical_pos);
                }
            }

            // EVENTS: Generate view events for UI updates
            let mut events = vec![
                ViewEvent::ActiveCursorUpdateRequired,
                ViewEvent::PositionIndicatorUpdateRequired,
                ViewEvent::CurrentAreaRedrawRequired, // Add redraw for visual selection
            ];

            // VISIBILITY: Ensure cursor remains visible after movement (scrolling if needed)
            let content_width = self.get_content_width();
            let visibility_events = self.ensure_current_cursor_visible(content_width);
            events.extend(visibility_events);

            events
        } else {
            // NO MOVEMENT: Return empty events if cursor couldn't move
            vec![]
        }
    }

    /// Move cursor up in current area
    pub fn move_cursor_up(&mut self) -> Vec<ViewEvent> {
        let current_display_pos = self.get_current_display_cursor();
        let current_mode = self.panes[self.current_pane].editor_mode;

        if current_display_pos.row > 0 {
            let new_line = current_display_pos.row - 1;

            // Clamp column to the length of the new line to prevent cursor going beyond line end
            // Mode-dependent: Normal/Visual stops at last char, Insert can go one past
            let new_col = if let Some(display_line) = self.panes[self.current_pane]
                .display_cache
                .get_display_line(new_line)
            {
                let line_char_count = display_line.char_count();
                let max_col = if current_mode == EditorMode::Insert {
                    line_char_count // Insert mode: can be positioned after last character
                } else {
                    line_char_count.saturating_sub(1) // Normal/Visual: stop at last character
                };
                current_display_pos.col.min(max_col)
            } else {
                current_display_pos.col
            };

            let new_display_pos = Position::new(new_line, new_col);
            self.panes[self.current_pane].display_cursor = new_display_pos;

            // Sync logical cursor with new display position
            if let Some(logical_pos) = self.panes[self.current_pane]
                .display_cache
                .display_to_logical_position(new_display_pos.row, new_display_pos.col)
            {
                let new_logical_pos = LogicalPosition::new(logical_pos.row, logical_pos.col);
                self.panes[self.current_pane]
                    .buffer
                    .set_cursor(new_logical_pos);

                // CRITICAL FIX: Update visual selection if active (similar to set_current_cursor_position)
                if self.panes[self.current_pane]
                    .visual_selection_start
                    .is_some()
                {
                    self.panes[self.current_pane].visual_selection_end = Some(new_logical_pos);
                    tracing::debug!("Updated visual selection end to {:?}", new_logical_pos);
                }
            }

            let mut events = vec![
                ViewEvent::ActiveCursorUpdateRequired,
                ViewEvent::PositionIndicatorUpdateRequired,
                ViewEvent::CurrentAreaRedrawRequired, // Add redraw for visual selection
            ];

            // Ensure cursor is visible and add visibility events
            let content_width = self.get_content_width();
            let visibility_events = self.ensure_current_cursor_visible(content_width);
            events.extend(visibility_events);

            events
        } else {
            vec![]
        }
    }

    /// Move cursor down in current area
    pub fn move_cursor_down(&mut self) -> Vec<ViewEvent> {
        let current_display_pos = self.get_current_display_cursor();
        let current_mode = self.panes[self.current_pane].editor_mode;

        let next_display_line = current_display_pos.row + 1;

        // Check if the next display line actually exists in the display cache
        // This prevents cursor from moving beyond actual content
        if let Some(display_line) = self.panes[self.current_pane]
            .display_cache
            .get_display_line(next_display_line)
        {
            // Only move if there's actual content at the next line
            // Clamp column to the length of the new line to prevent cursor going beyond line end
            // Mode-dependent: Normal/Visual stops at last char, Insert can go one past
            let line_char_count = display_line.char_count();
            let max_col = if current_mode == EditorMode::Insert {
                line_char_count // Insert mode: can be positioned after last character
            } else {
                line_char_count.saturating_sub(1) // Normal/Visual: stop at last character
            };
            let new_col = current_display_pos.col.min(max_col);
            let new_display_pos = Position::new(next_display_line, new_col);

            self.panes[self.current_pane].display_cursor = new_display_pos;

            // Sync logical cursor with new display position
            if let Some(logical_pos) = self.panes[self.current_pane]
                .display_cache
                .display_to_logical_position(new_display_pos.row, new_display_pos.col)
            {
                let new_logical_pos = LogicalPosition::new(logical_pos.row, logical_pos.col);
                self.panes[self.current_pane]
                    .buffer
                    .set_cursor(new_logical_pos);

                // CRITICAL FIX: Update visual selection if active (similar to set_current_cursor_position)
                if self.panes[self.current_pane]
                    .visual_selection_start
                    .is_some()
                {
                    self.panes[self.current_pane].visual_selection_end = Some(new_logical_pos);
                    tracing::debug!("Updated visual selection end to {:?}", new_logical_pos);
                }
            }

            let mut events = vec![
                ViewEvent::ActiveCursorUpdateRequired,
                ViewEvent::PositionIndicatorUpdateRequired,
                ViewEvent::CurrentAreaRedrawRequired, // Add redraw for visual selection
            ];

            // Ensure cursor is visible and add visibility events
            let content_width = self.get_content_width();
            let visibility_events = self.ensure_current_cursor_visible(content_width);
            events.extend(visibility_events);

            events
        } else {
            vec![]
        }
    }

    /// Move cursor to start of current line
    pub fn move_cursor_to_start_of_line(&mut self) -> Vec<ViewEvent> {
        // Get current logical position
        let current_logical = self.panes[self.current_pane].buffer.cursor();

        // Create new logical position at start of current line (column 0)
        let new_logical = LogicalPosition::new(current_logical.line, 0);

        // Update logical cursor first
        self.panes[self.current_pane].buffer.set_cursor(new_logical);

        // Sync display cursor with logical cursor
        self.panes[self.current_pane].sync_display_cursor_with_logical();

        // CRITICAL FIX: Update visual selection end if in visual mode (same pattern as other cursor movements)
        let mut events = vec![
            ViewEvent::ActiveCursorUpdateRequired,
            ViewEvent::PositionIndicatorUpdateRequired,
        ];

        if self.panes[self.current_pane]
            .visual_selection_start
            .is_some()
        {
            self.panes[self.current_pane].visual_selection_end = Some(new_logical);
            events.push(ViewEvent::CurrentAreaRedrawRequired); // Redraw for visual selection
            tracing::debug!(
                "Line start movement updated visual selection end to {:?}",
                new_logical
            );
        }

        // Ensure cursor is visible and add visibility events
        let content_width = self.get_content_width();
        let visibility_events = self.ensure_current_cursor_visible(content_width);
        events.extend(visibility_events);

        events
    }

    /// Move cursor to end of current line for append (A command)
    /// This positions the cursor AFTER the last character for insert mode
    pub fn move_cursor_to_line_end_for_append(&mut self) -> Vec<ViewEvent> {
        // Get current logical position
        let current_logical = self.panes[self.current_pane].buffer.cursor();

        let mut events = vec![
            ViewEvent::ActiveCursorUpdateRequired,
            ViewEvent::PositionIndicatorUpdateRequired,
        ];

        // Get the current line content to find its length
        if let Some(line) = self.panes[self.current_pane]
            .buffer
            .content()
            .get_line(current_logical.line)
        {
            let line_length = line.chars().count();

            // For the 'A' command, position cursor AFTER the last character
            // This allows inserting at the end of the line
            let end_position = line_length; // Position after last character
            let new_logical = LogicalPosition::new(current_logical.line, end_position);

            // Update logical cursor first
            self.panes[self.current_pane].buffer.set_cursor(new_logical);

            // Sync display cursor with logical cursor
            self.panes[self.current_pane].sync_display_cursor_with_logical();

            // CRITICAL FIX: Update visual selection end if in visual mode (same pattern as other cursor movements)
            if self.panes[self.current_pane]
                .visual_selection_start
                .is_some()
            {
                self.panes[self.current_pane].visual_selection_end = Some(new_logical);
                events.push(ViewEvent::CurrentAreaRedrawRequired); // Redraw for visual selection
                tracing::debug!(
                    "Line end append movement updated visual selection end to {:?}",
                    new_logical
                );
            }
        }

        // Ensure cursor is visible with Insert-mode scrolling logic
        // The A command will immediately switch to Insert mode, so we need to use
        // Insert mode scrolling behavior here to ensure proper horizontal scrolling
        let content_width = self.get_content_width();
        let original_mode = self.panes[self.current_pane].editor_mode;

        // Temporarily set to Insert mode for proper scrolling calculation
        self.panes[self.current_pane].editor_mode = EditorMode::Insert;
        let visibility_events = self.ensure_current_cursor_visible(content_width);

        // Restore original mode
        self.panes[self.current_pane].editor_mode = original_mode;

        events.extend(visibility_events);

        events
    }

    /// Move cursor to end of current line
    pub fn move_cursor_to_end_of_line(&mut self) -> Vec<ViewEvent> {
        // Get current logical position
        let current_logical = self.panes[self.current_pane].buffer.cursor();
        let current_mode = self.panes[self.current_pane].editor_mode;

        let mut events = vec![
            ViewEvent::ActiveCursorUpdateRequired,
            ViewEvent::PositionIndicatorUpdateRequired,
        ];

        // Get the current line content to find its length
        if let Some(line) = self.panes[self.current_pane]
            .buffer
            .content()
            .get_line(current_logical.line)
        {
            let line_length = line.chars().count();

            // Create new logical position at end of current line
            // Mode-dependent positioning:
            // - Normal/Visual mode: cursor stops at the last character (index n-1)
            // - Insert mode: cursor can be positioned after last character (index n)
            // This is used for the 'A' command which should position after last char
            let end_position = if current_mode == EditorMode::Insert && line_length > 0 {
                line_length // Position after last character for insert mode
            } else if line_length > 0 {
                line_length - 1 // Stop at last character for normal/visual mode
            } else {
                0 // Empty line, stay at position 0
            };
            let new_logical = LogicalPosition::new(current_logical.line, end_position);

            // Update logical cursor first
            self.panes[self.current_pane].buffer.set_cursor(new_logical);

            // Sync display cursor with logical cursor
            self.panes[self.current_pane].sync_display_cursor_with_logical();

            // CRITICAL FIX: Update visual selection end if in visual mode (same pattern as other cursor movements)
            if self.panes[self.current_pane]
                .visual_selection_start
                .is_some()
            {
                self.panes[self.current_pane].visual_selection_end = Some(new_logical);
                events.push(ViewEvent::CurrentAreaRedrawRequired); // Redraw for visual selection
                tracing::debug!(
                    "Line end movement updated visual selection end to {:?}",
                    new_logical
                );
            }
        }

        // Ensure cursor is visible and add visibility events
        let content_width = self.get_content_width();
        let visibility_events = self.ensure_current_cursor_visible(content_width);
        events.extend(visibility_events);

        events
    }

    /// Move cursor to start of document
    pub fn move_cursor_to_document_start(&mut self) -> Vec<ViewEvent> {
        // Use proper cursor positioning method to ensure logical/display sync
        let start_position = Position::origin();
        let _result = self.panes[self.current_pane].set_display_cursor(start_position);

        let mut events = vec![
            ViewEvent::ActiveCursorUpdateRequired,
            ViewEvent::PositionIndicatorUpdateRequired,
        ];

        // CRITICAL FIX: Update visual selection end if in visual mode (same pattern as other cursor movements)
        if self.panes[self.current_pane]
            .visual_selection_start
            .is_some()
        {
            let new_cursor_pos = self.panes[self.current_pane].buffer.cursor();
            self.panes[self.current_pane].visual_selection_end = Some(new_cursor_pos);
            events.push(ViewEvent::CurrentAreaRedrawRequired); // Redraw for visual selection
            tracing::debug!(
                "Document start movement updated visual selection end to {:?}",
                new_cursor_pos
            );
        }

        // Ensure cursor is visible and add visibility events
        let content_width = self.get_content_width();
        let visibility_events = self.ensure_current_cursor_visible(content_width);
        events.extend(visibility_events);

        events
    }

    /// Move cursor to end of document
    pub fn move_cursor_to_document_end(&mut self) -> Vec<ViewEvent> {
        let display_cache = &self.panes[self.current_pane].display_cache;
        // Find the last valid display line by iterating
        let mut last_line_idx = 0;
        let mut idx = 0;
        while display_cache.get_display_line(idx).is_some() {
            last_line_idx = idx;
            idx += 1;
        }

        let mut events = vec![
            ViewEvent::ActiveCursorUpdateRequired,
            ViewEvent::PositionIndicatorUpdateRequired,
        ];

        if let Some(last_line) = display_cache.get_display_line(last_line_idx) {
            // FIXED: Use display width instead of character count for proper multibyte character support
            let line_display_width = last_line.display_width();
            let end_position = Position::new(last_line_idx, line_display_width);

            // Use proper cursor positioning method to ensure logical/display sync
            let _result = self.panes[self.current_pane].set_display_cursor(end_position);

            // CRITICAL FIX: Update visual selection end if in visual mode (same pattern as other cursor movements)
            if self.panes[self.current_pane]
                .visual_selection_start
                .is_some()
            {
                let new_cursor_pos = self.panes[self.current_pane].buffer.cursor();
                self.panes[self.current_pane].visual_selection_end = Some(new_cursor_pos);
                events.push(ViewEvent::CurrentAreaRedrawRequired); // Redraw for visual selection
                tracing::debug!(
                    "Document end movement updated visual selection end to {:?}",
                    new_cursor_pos
                );
            }
        }

        // Ensure cursor is visible and add visibility events
        let content_width = self.get_content_width();
        let visibility_events = self.ensure_current_cursor_visible(content_width);
        events.extend(visibility_events);

        events
    }

    /// Move cursor to specific line number (1-based)
    /// If line_number is out of bounds, clamps to the last available line (vim behavior)
    pub fn move_cursor_to_line(&mut self, line_number: usize) -> Vec<ViewEvent> {
        if line_number == 0 {
            return vec![];
        }

        let display_cache = &self.panes[self.current_pane].display_cache;
        let max_line_count = display_cache.display_line_count();

        if max_line_count == 0 {
            return vec![]; // No lines to navigate to
        }

        // Clamp to valid range (1 to max_line_count)
        let clamped_line_number = line_number.min(max_line_count);
        let target_line_idx = clamped_line_number - 1; // Convert to 0-based

        // Set cursor position
        self.panes[self.current_pane].display_cursor = Position::new(target_line_idx, 0);
        let mut events = vec![
            ViewEvent::ActiveCursorUpdateRequired,
            ViewEvent::PositionIndicatorUpdateRequired,
        ];

        // Ensure cursor is visible and add visibility events
        let content_width = self.get_content_width();
        let visibility_events = self.ensure_current_cursor_visible(content_width);
        events.extend(visibility_events);

        events
    }

    /// Calculate pane boundaries for rendering
    /// Returns (request_height, response_start, response_height)
    #[allow(clippy::type_complexity)]
    pub fn get_pane_boundaries(&self, has_response: bool) -> (u16, u16, u16) {
        if has_response {
            // When response exists, split the space
            let request_height = self.request_pane_height();
            let response_start = request_height + 1; // +1 for separator
            let response_height = self.response_pane_height(true);
            (request_height, response_start, response_height)
        } else {
            // When no response, request pane uses full available space
            let request_height = self.terminal_dimensions.1 - 1; // -1 for status bar
            let response_start = request_height + 1; // Won't be used
            let response_height = 0; // Hidden
            (request_height, response_start, response_height)
        }
    }

    // Per-pane mode management methods
    /// Get current editor mode for the currently active pane
    pub fn get_current_pane_mode(&self) -> EditorMode {
        self.panes[self.current_pane].get_mode()
    }

    /// Set editor mode for the currently active pane
    pub fn set_current_pane_mode(&mut self, mode: EditorMode) {
        self.panes[self.current_pane].set_mode(mode);
    }

    /// Get editor mode for a specific pane
    pub fn get_pane_mode(&self, pane: Pane) -> EditorMode {
        self.panes[pane].get_mode()
    }

    /// Set editor mode for a specific pane
    pub fn set_pane_mode(&mut self, pane: Pane, mode: EditorMode) {
        self.panes[pane].set_mode(mode);
    }

    /// Get reference to the currently active pane state
    pub fn get_current_pane_state(&self) -> Option<&PaneState> {
        Some(&self.panes[self.current_pane])
    }
}
