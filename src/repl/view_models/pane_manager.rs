//! # Pane Management
//!
//! Handles pane switching, mode changes, and pane-related state management.
//! Contains the PaneManager struct that encapsulates all pane-related operations.
//!
//! ⚠️ **CRITICAL ARCHITECTURAL GUIDELINE** ⚠️
//!
//! **DO NOT IMPLEMENT BUSINESS LOGIC IN PANE_MANAGER**
//!
//! PaneManager is a pure layout manager. All business logic (text editing, cursor movement,
//! visual selection, etc.) should be implemented in PaneState. PaneManager should only:
//! - Switch between panes
//! - Calculate layout dimensions  
//! - Delegate operations to the appropriate PaneState
//! - Emit view events for rendering
//!
//! If you find yourself implementing text operations, cursor logic, or edit functionality
//! in PaneManager, move it to PaneState instead. Use the PaneCapabilities system to
//! control what operations are allowed on each pane.
//!
//! HIGH-LEVEL ARCHITECTURE:
//! PaneManager implements the Manager pattern to encapsulate all pane-related operations:
//! - Manages Request and Response panes as an array with semantic operations
//! - Provides high-level pane switching without exposing internal array indices
//! - Handles terminal dimension updates and pane layout calculations
//! - Delegates cursor management, scrolling, and text operations to PaneState instances
//!
//! CORE RESPONSIBILITIES (Post-Refactoring):
//! 1. Layout Management: Computes pane dimensions and content width based on terminal size
//! 2. Pane Switching: Manages current pane state and provides semantic pane operations
//! 3. Pure Delegation: Forwards business logic operations to appropriate PaneState instances
//! 4. Event Coordination: Aggregates ViewEvents from PaneState operations for rendering
//! 5. Settings Management: Handles display settings (wrap, line numbers, tab width) that affect all panes

use crate::repl::events::{EditorMode, LogicalPosition, Pane, PaneCapabilities, ViewEvent};
use crate::repl::geometry::Position;
use crate::repl::view_models::pane_state::{PaneState, VisualSelectionRestoreResult};

/// Type alias for visual selection state to reduce complexity
type VisualSelectionState = (
    Option<LogicalPosition>,
    Option<LogicalPosition>,
    Option<Pane>,
);

/// Type alias for delete operation result to reduce complexity
type DeleteResult = Option<(String, Vec<ViewEvent>)>;

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
    show_line_numbers: bool,
    tab_width: usize,                    // Number of spaces per tab stop (default 4)
    expand_tab: bool,                    // If true, insert spaces instead of tab character
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
        let content_width = if true {
            // Default to showing line numbers
            (terminal_dimensions.0 as usize).saturating_sub(4) // Account for line numbers
        } else {
            terminal_dimensions.0 as usize
        };

        // Calculate pane heights
        let request_pane_height = (terminal_dimensions.1 / 2) as usize;
        let response_pane_height = (terminal_dimensions.1 as usize)
            .saturating_sub((terminal_dimensions.1 / 2) as usize)
            .saturating_sub(2) // -2 for separator and status
            .max(1); // Ensure minimum height of 1

        // Initialize pane array with proper display caches and dimensions
        // Initialize capabilities based on pane type - Request pane allows all operations, Response pane is read-only
        let request_pane = PaneState::new(
            Pane::Request,
            content_width,
            request_pane_height,
            true,
            PaneCapabilities::FULL_ACCESS,
        );
        let response_pane = PaneState::new(
            Pane::Response,
            content_width,
            response_pane_height,
            true,
            PaneCapabilities::READ_ONLY,
        );

        Self {
            panes: [request_pane, response_pane],
            current_pane: Pane::Request,
            wrap_enabled: false,
            show_line_numbers: true, // Default to showing line numbers
            tab_width: 4,            // Default tab width of 4 spaces
            expand_tab: false,       // Default to inserting real tabs, not spaces
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
        let (start, end) = self.panes[self.current_pane].get_visual_selection();
        (
            start,
            end,
            if start.is_some() {
                Some(self.current_pane)
            } else {
                None
            },
        )
    }

    /// Check if a position is within visual selection
    pub fn is_position_selected(&self, position: LogicalPosition, pane: Pane) -> bool {
        // Delegate to specified pane
        self.panes[pane].is_position_selected(position)
    }

    /// Check if current pane has an active visual selection
    pub fn has_visual_selection(&self) -> bool {
        let (start, _end) = self.panes[self.current_pane].get_visual_selection();
        start.is_some()
    }

    /// Start visual selection in current area
    pub fn start_visual_selection(&mut self) -> Vec<ViewEvent> {
        // Delegate to current pane
        self.panes[self.current_pane].start_visual_selection()
    }

    /// End visual selection in current area
    pub fn end_visual_selection(&mut self) -> Vec<ViewEvent> {
        // Delegate to current pane
        self.panes[self.current_pane].end_visual_selection()
    }

    /// Update visual selection end position
    pub fn update_visual_selection(&mut self, position: LogicalPosition) -> Vec<ViewEvent> {
        // Delegate to current pane
        self.panes[self.current_pane].update_visual_selection(position)
    }

    /// Get selected text from the current pane
    pub fn get_selected_text(&self) -> Option<String> {
        self.panes[self.current_pane].get_selected_text()
    }

    /// Update visual selection during cursor movement if active
    /// Helper method to be called from cursor movement operations
    pub fn update_visual_selection_on_cursor_move(
        &mut self,
        new_position: LogicalPosition,
    ) -> Option<ViewEvent> {
        self.panes[self.current_pane].update_visual_selection_on_cursor_move(new_position)
    }

    /// Restore the last visual selection (for 'gv' command)
    /// Returns the mode and view events if restoration successful
    pub fn restore_last_visual_selection(&mut self) -> VisualSelectionRestoreResult {
        self.panes[self.current_pane].restore_last_visual_selection()
    }

    /// Delete selected text from the current pane
    /// Returns (deleted_text, view_events) if successful
    pub fn delete_selected_text(&mut self) -> DeleteResult {
        if let Some((deleted_text, model_event)) =
            self.panes[self.current_pane].delete_selected_text()
        {
            // Process the model event and return appropriate view events
            let view_events = match model_event {
                crate::repl::events::ModelEvent::TextDeleted { .. } => {
                    // Rebuild display cache for the affected pane
                    let visibility_events = self.rebuild_display_caches_and_sync();
                    let mut events = vec![ViewEvent::CurrentAreaRedrawRequired];
                    events.extend(visibility_events);
                    events
                }
                _ => vec![ViewEvent::CurrentAreaRedrawRequired],
            };

            Some((deleted_text, view_events))
        } else {
            // No selection to delete
            None
        }
    }

    /// Insert text block-wise at specific positions (for block paste operations)
    pub fn insert_block_wise(
        &mut self,
        start_position: LogicalPosition,
        block_lines: &[&str],
    ) -> Vec<ViewEvent> {
        self.panes[self.current_pane].insert_block_wise(start_position, block_lines)
    }

    /// Get the length of the current line in the current pane
    pub fn get_current_line_length(&self) -> usize {
        let current_pane = &self.panes[self.current_pane];
        let cursor_pos = self.get_current_cursor_position();
        current_pane
            .buffer
            .content()
            .get_line(cursor_pos.line)
            .map(|line| line.len())
            .unwrap_or(0)
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

    /// Get line number visibility state
    pub fn is_line_numbers_visible(&self) -> bool {
        self.show_line_numbers
    }

    /// Set line number visibility state
    pub fn set_line_numbers_visible(&mut self, visible: bool) {
        tracing::debug!(
            "🔧 PaneManager::set_line_numbers_visible: changing from {} to {}",
            self.show_line_numbers,
            visible
        );
        self.show_line_numbers = visible;
        tracing::debug!(
            "✅ PaneManager::set_line_numbers_visible: show_line_numbers is now {}",
            self.show_line_numbers
        );
    }

    /// Get tab width (number of spaces per tab stop)
    pub fn get_tab_width(&self) -> usize {
        self.tab_width
    }

    /// Set tab width (number of spaces per tab stop)
    pub fn set_tab_width(&mut self, width: usize) {
        // Ensure tab width is at least 1 to prevent division by zero or infinite loops
        let tab_width = width.max(1);
        tracing::debug!(
            "🔧 PaneManager::set_tab_width: changing from {} to {}",
            self.tab_width,
            tab_width
        );
        self.tab_width = tab_width;
        tracing::debug!(
            "✅ PaneManager::set_tab_width: tab_width is now {}",
            self.tab_width
        );
        // TODO: Invalidate display caches since tab width affects text layout
    }

    /// Get expand tab setting (whether to insert spaces instead of tab character)
    pub fn get_expand_tab(&self) -> bool {
        self.expand_tab
    }

    /// Set expand tab setting (whether to insert spaces instead of tab character)
    pub fn set_expand_tab(&mut self, expand: bool) {
        tracing::debug!(
            "🔧 PaneManager::set_expand_tab: changing from {} to {}",
            self.expand_tab,
            expand
        );
        self.expand_tab = expand;
        tracing::debug!(
            "✅ PaneManager::set_expand_tab: expand_tab is now {}",
            self.expand_tab
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
        let content_width = if self.show_line_numbers {
            (width as usize).saturating_sub(4) // Account for line numbers
        } else {
            width as usize
        };
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
        self.panes[Pane::Request].build_display_cache(
            content_width,
            self.wrap_enabled,
            self.tab_width,
        );
        self.panes[Pane::Response].build_display_cache(
            content_width,
            self.wrap_enabled,
            self.tab_width,
        );

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
        self.panes[Pane::Request].build_display_cache(
            content_width,
            self.wrap_enabled,
            self.tab_width,
        );
        self.panes[Pane::Response].build_display_cache(
            content_width,
            self.wrap_enabled,
            self.tab_width,
        );
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

    /// Insert character at current cursor position using generic delegation
    ///
    /// This method delegates to the current pane's insert_char() method,
    /// which handles capability checking and text insertion logic.
    pub fn insert_char(&mut self, ch: char) -> Vec<ViewEvent> {
        let content_width = self.get_content_width();

        // Delegate to current pane with capability checking
        let mut events = self.panes[self.current_pane].insert_char(
            ch,
            content_width,
            self.wrap_enabled,
            self.tab_width,
        );

        // Ensure cursor is visible after insertion if events were generated
        if !events.is_empty() {
            let visibility_events = self.ensure_current_cursor_visible(content_width);
            events.extend(visibility_events);
        }

        events
    }

    /// Delete character before cursor using generic delegation
    ///
    /// This method delegates to the current pane's delete_char_before_cursor() method,
    /// which handles capability checking and deletion logic.
    pub fn delete_char_before_cursor(&mut self) -> Vec<ViewEvent> {
        let content_width = self.get_content_width();

        // Delegate to current pane with capability checking
        self.panes[self.current_pane].delete_char_before_cursor(
            content_width,
            self.wrap_enabled,
            self.tab_width,
        )
    }

    /// Delete character after cursor (generic method for any pane)
    pub fn delete_char_after_cursor(&mut self) -> Vec<ViewEvent> {
        let content_width = self.get_content_width();

        // Delegate to current pane with capability checking
        self.panes[self.current_pane].delete_char_after_cursor(
            content_width,
            self.wrap_enabled,
            self.tab_width,
        )
    }

    /// Delete character after cursor without line joining (safe for Visual Block Insert mode)
    pub fn delete_char_after_cursor_visual_block_safe(&mut self) -> Vec<ViewEvent> {
        let content_width = self.get_content_width();

        // Delegate to current pane with line joining disabled
        self.panes[self.current_pane].delete_char_after_cursor_no_join(
            content_width,
            self.wrap_enabled,
            self.tab_width,
        )
    }

    /// Cut (delete and yank) character at cursor position, returning deleted character
    pub fn cut_char_at_cursor(&mut self) -> Option<String> {
        let content_width = self.get_content_width();

        // Delegate to current pane with capability checking
        self.panes[self.current_pane].delete_char_at_cursor_with_return(
            content_width,
            self.wrap_enabled,
            self.tab_width,
        )
    }

    /// Cut (delete and yank) from cursor to end of line, returning deleted text
    pub fn cut_to_end_of_line(&mut self) -> Option<String> {
        let content_width = self.get_content_width();

        // Delegate to current pane with capability checking
        self.panes[self.current_pane].cut_to_end_of_line_with_return(
            content_width,
            self.wrap_enabled,
            self.tab_width,
        )
    }

    /// Set cursor position in current area
    pub fn set_current_cursor_position(&mut self, position: LogicalPosition) -> Vec<ViewEvent> {
        self.panes[self.current_pane].set_current_cursor_position(position)
    }

    /// Clear editable content (semantic operation)
    pub fn clear_editable_content(&mut self) -> Vec<ViewEvent> {
        self.panes[Pane::Request].clear_editable_content()
    }

    /// Set Request pane content
    pub fn set_request_content(&mut self, text: &str) -> Vec<ViewEvent> {
        self.panes[Pane::Request].set_request_content(text)
    }

    /// Set Response pane content
    pub fn set_response_content(&mut self, text: &str) -> Vec<ViewEvent> {
        let events = self.panes[Pane::Response].set_response_content(text);

        // Rebuild display cache to ensure rendering sees the updated content
        let content_width = if self.show_line_numbers {
            (self.terminal_dimensions.0 as usize).saturating_sub(4) // Same as Request pane
        } else {
            self.terminal_dimensions.0 as usize
        };
        self.panes[Pane::Response].build_display_cache(
            content_width,
            self.wrap_enabled,
            self.tab_width,
        );

        events
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
        if self.show_line_numbers {
            self.panes[self.current_pane].get_line_number_width()
        } else {
            0 // Return 0 when line numbers are hidden
        }
    }

    /// Get line number width for specific pane
    pub fn get_line_number_width(&self, pane: Pane) -> usize {
        if self.show_line_numbers {
            self.panes[pane].get_line_number_width()
        } else {
            0 // Return 0 when line numbers are hidden
        }
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
        // Delegate to current pane with capability checking
        let content_width = self.get_content_width();
        self.panes[self.current_pane].move_cursor_to_next_word(content_width)
    }

    /// Move cursor to previous word in current pane
    pub fn move_cursor_to_previous_word(&mut self) -> Vec<ViewEvent> {
        // Delegate to current pane with capability checking
        let content_width = self.get_content_width();
        self.panes[self.current_pane].move_cursor_to_previous_word(content_width)
    }

    /// Move cursor to end of word in current pane
    pub fn move_cursor_to_end_of_word(&mut self) -> Vec<ViewEvent> {
        // Delegate to current pane with capability checking
        let content_width = self.get_content_width();
        self.panes[self.current_pane].move_cursor_to_end_of_word(content_width)
    }

    /// Get content width for current pane (temporary - will be moved to internal calculation)
    pub fn get_content_width(&self) -> usize {
        // Use current pane's line number width calculation
        // This is a simplified version - should be improved later
        if self.show_line_numbers {
            (self.terminal_dimensions.0 as usize).saturating_sub(4) // Account for line numbers
        } else {
            self.terminal_dimensions.0 as usize // Full width when line numbers are hidden
        }
    }

    /// Move cursor left in current area
    ///
    /// Delegates to PaneState for business logic with capability checking.
    pub fn move_cursor_left(&mut self) -> Vec<ViewEvent> {
        let content_width = self.get_content_width();
        self.panes[self.current_pane].move_cursor_left(content_width)
    }

    /// Move cursor right in current area
    ///
    /// HIGH-LEVEL LOGIC:
    /// 1. Check if cursor can move right within current line (mode-aware boundary check)
    /// 2. If not, check if cursor can move to next line (line wrap navigation)
    /// 3. Perform the actual cursor movement using character-aware positioning
    /// 4. Sync display cursor with logical cursor and update visual selections
    pub fn move_cursor_right(&mut self) -> Vec<ViewEvent> {
        let content_width = self.get_content_width();
        self.panes[self.current_pane].move_cursor_right(content_width)
    }

    /// Move cursor up in current area
    ///
    /// Delegates to PaneState for business logic with capability checking.
    pub fn move_cursor_up(&mut self) -> Vec<ViewEvent> {
        let content_width = self.get_content_width();
        self.panes[self.current_pane].move_cursor_up(content_width)
    }

    /// Move cursor down in current area
    ///
    /// Delegates to PaneState for business logic with capability checking.
    /// Use PaneState::move_cursor_down() directly for new code.
    pub fn move_cursor_down(&mut self) -> Vec<ViewEvent> {
        let content_width = self.get_content_width();
        self.panes[self.current_pane].move_cursor_down(content_width)
    }

    /// Move cursor to start of current line
    ///
    /// Delegates to PaneState for business logic with capability checking.
    /// Use PaneState::move_cursor_to_start_of_line() directly for new code.
    pub fn move_cursor_to_start_of_line(&mut self) -> Vec<ViewEvent> {
        let content_width = self.get_content_width();
        self.panes[self.current_pane].move_cursor_to_start_of_line(content_width)
    }

    /// Move cursor to end of current line for append (A command)
    /// This positions the cursor AFTER the last character for insert mode
    ///
    /// Delegates to PaneState for business logic with capability checking.
    /// Use PaneState::move_cursor_to_line_end_for_append() directly for new code.
    pub fn move_cursor_to_line_end_for_append(&mut self) -> Vec<ViewEvent> {
        let content_width = self.get_content_width();
        self.panes[self.current_pane].move_cursor_to_line_end_for_append(content_width)
    }

    /// Move cursor to end of current line
    ///
    /// Delegates to PaneState for business logic with capability checking.
    /// Use PaneState::move_cursor_to_end_of_line() directly for new code.
    pub fn move_cursor_to_end_of_line(&mut self) -> Vec<ViewEvent> {
        let content_width = self.get_content_width();
        self.panes[self.current_pane].move_cursor_to_end_of_line(content_width)
    }

    /// Move cursor to start of document
    ///
    /// Delegates to PaneState for business logic with capability checking.
    /// Use PaneState::move_cursor_to_document_start() directly for new code.
    pub fn move_cursor_to_document_start(&mut self) -> Vec<ViewEvent> {
        let content_width = self.get_content_width();
        self.panes[self.current_pane].move_cursor_to_document_start(content_width)
    }

    /// Move cursor to end of document
    ///
    /// Delegates to PaneState for business logic with capability checking.
    /// Use PaneState::move_cursor_to_document_end() directly for new code.
    pub fn move_cursor_to_document_end(&mut self) -> Vec<ViewEvent> {
        let content_width = self.get_content_width();
        self.panes[self.current_pane].move_cursor_to_document_end(content_width)
    }

    /// Move cursor to specific line number (1-based)
    /// If line_number is out of bounds, clamps to the last available line (vim behavior)
    ///
    /// Delegates to PaneState for business logic with capability checking.
    /// Use PaneState::move_cursor_to_line() directly for new code.
    pub fn move_cursor_to_line(&mut self, line_number: usize) -> Vec<ViewEvent> {
        let content_width = self.get_content_width();
        self.panes[self.current_pane].move_cursor_to_line(line_number, content_width)
    }

    /// Move cursor down one page (Ctrl+f)
    pub fn move_cursor_page_down(&mut self) -> Vec<ViewEvent> {
        self.panes[self.current_pane].move_cursor_page_down()
    }

    /// Move cursor up one page (Ctrl+b)
    pub fn move_cursor_page_up(&mut self) -> Vec<ViewEvent> {
        self.panes[self.current_pane].move_cursor_page_up()
    }

    /// Move cursor down half a page (Ctrl+d)
    pub fn move_cursor_half_page_down(&mut self) -> Vec<ViewEvent> {
        self.panes[self.current_pane].move_cursor_half_page_down()
    }

    /// Move cursor up half a page (Ctrl+u)
    pub fn move_cursor_half_page_up(&mut self) -> Vec<ViewEvent> {
        self.panes[self.current_pane].move_cursor_half_page_up()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_cursor_page_down_should_work() {
        let mut manager = PaneManager::new((80, 24));

        // Set up some content in the current pane (Request pane by default)
        let content = "Line 1\nLine 2\nLine 3\nLine 4\nLine 5\nLine 6\nLine 7\nLine 8\nLine 9\nLine 10\nLine 11\nLine 12\nLine 13\nLine 14\nLine 15\nLine 16\nLine 17\nLine 18\nLine 19\nLine 20\nLine 21\nLine 22\nLine 23\nLine 24\nLine 25";
        manager.set_request_content(content);

        // Rebuild display caches to ensure content is processed
        let content_width = manager.get_content_width();
        manager.rebuild_display_caches(content_width);

        // Get initial cursor position
        let initial_cursor = manager.get_current_cursor_position();
        assert_eq!(initial_cursor.line, 0);
        assert_eq!(initial_cursor.column, 0);

        // Debug: check pane dimensions and line count
        let pane_height = manager.panes[manager.current_pane].pane_dimensions.height;
        let line_count = manager.panes[manager.current_pane]
            .display_cache
            .display_line_count();
        tracing::debug!(
            "Test: pane_height={}, line_count={}",
            pane_height,
            line_count
        );

        // Perform page down
        let events = manager.move_cursor_page_down();

        // Debug: print events if empty
        if events.is_empty() {
            tracing::warn!("Test: Page down returned empty events");
        }

        tracing::debug!("Test: events.len()={}", events.len());

        // Check if there's actually room to page down
        if line_count > pane_height {
            // Should have generated events for cursor update
            assert!(
                !events.is_empty(),
                "Expected events for page down but got none. pane_height={pane_height}, line_count={line_count}"
            );
            assert!(events.iter().any(|e| matches!(
                e,
                crate::repl::events::ViewEvent::ActiveCursorUpdateRequired
            )));

            // Cursor should have moved down by page size (pane height)
            let new_cursor = manager.get_current_cursor_position();
            tracing::debug!("Test: new_cursor.line={}, expected > 0", new_cursor.line);
            assert!(
                new_cursor.line > 0,
                "Cursor should have moved from line 0 to line > 0, but got line {new_cursor_line}",
                new_cursor_line = new_cursor.line
            );
            assert_eq!(new_cursor.column, 0); // Column should remain at 0
        } else {
            // If there's not enough content to page down, it should return empty events
            tracing::debug!("Not enough content to page down, this is expected");
        }
    }

    #[test]
    fn move_cursor_page_down_should_not_move_beyond_last_line() {
        let mut manager = PaneManager::new((80, 24));

        // Set up limited content (less than a page)
        let content = "Line 1\nLine 2\nLine 3";
        manager.set_request_content(content);

        // Try to page down - should not move since we're already at the last possible position
        let events = manager.move_cursor_page_down();

        // Should return empty events since no movement occurred
        assert!(events.is_empty());

        // Cursor should stay at line 0
        let cursor = manager.get_current_cursor_position();
        assert_eq!(cursor.line, 0);
    }

    #[test]
    fn move_cursor_page_down_should_handle_empty_content() {
        let mut manager = PaneManager::new((80, 24));

        // Empty content (use default empty content)

        // Try to page down
        let events = manager.move_cursor_page_down();

        // Should return empty events since there's no content
        assert!(events.is_empty());

        // Cursor should remain at origin
        let cursor = manager.get_current_cursor_position();
        assert_eq!(cursor.line, 0);
        assert_eq!(cursor.column, 0);
    }

    #[test]
    fn move_cursor_page_down_should_work_with_doublebyte_characters() {
        let mut manager = PaneManager::new((80, 24));

        // Set up content with Japanese characters (doublebyte)
        let content = "こんにちは世界\nこれは日本語のテストです\nページダウンのテスト\n今日はいい天気ですね\n昨日は雨でした\n明日は晴れるでしょう\nもう一行追加します\nさらにもう一行\nテストデータ続行\n最後の行まで\n確認中です\n動作テスト\n最終確認行\nこれで終わりです\nもう少し続けます\nほぼ終了です\n本当の最後\n完了です\nテスト完了\n最終行";
        manager.set_request_content(content);

        // Rebuild display caches to ensure content is processed
        let content_width = manager.get_content_width();
        manager.rebuild_display_caches(content_width);

        // Get initial cursor position
        let initial_cursor = manager.get_current_cursor_position();
        assert_eq!(initial_cursor.line, 0);
        assert_eq!(initial_cursor.column, 0);

        // Debug: check line count
        let line_count = manager.panes[manager.current_pane]
            .display_cache
            .display_line_count();
        tracing::debug!("Test (doublebyte): line_count={}", line_count);

        // Perform page down
        let events = manager.move_cursor_page_down();

        // Should have moved the cursor
        assert!(!events.is_empty());

        // Cursor should have moved to a new position
        let new_cursor = manager.get_current_cursor_position();
        tracing::debug!(
            "Test (doublebyte): moved from line {} to line {}",
            initial_cursor.line,
            new_cursor.line
        );
        assert!(
            new_cursor.line > 0,
            "Cursor should have moved down from line 0 with doublebyte content"
        );
        assert_eq!(new_cursor.column, 0); // Should be at start of new line
    }

    #[test]
    fn move_cursor_should_maintain_virtual_column_vim_style() {
        let mut manager = PaneManager::new((80, 24));

        // Set up content with varying line lengths - demonstrates Vim behavior
        let content = "This is a very long line that extends beyond most other lines in this test\nShort\nAnother medium length line here\nX\nThis is again a very long line that should restore the cursor to original position\nTiny";
        manager.set_request_content(content);

        // Rebuild display caches
        let content_width = manager.get_content_width();
        manager.rebuild_display_caches(content_width);

        // Position cursor near the end of the first long line (column 50)
        let target_column = 50;
        manager.panes[manager.current_pane].display_cursor = Position::new(0, target_column);

        // Update virtual column to this position (simulates user moving horizontally)
        manager.panes[manager.current_pane].set_virtual_column(target_column);

        // Sync buffer cursor with display cursor
        if let Some(logical_pos) = manager.panes[manager.current_pane]
            .display_cache
            .display_to_logical_position(0, target_column)
        {
            manager.panes[manager.current_pane]
                .buffer
                .set_cursor(LogicalPosition::new(logical_pos.row, logical_pos.col));
        }

        tracing::debug!("Starting position: line 0, column {}", target_column);
        tracing::debug!(
            "Virtual column: {}",
            manager.panes[manager.current_pane].get_virtual_column()
        );

        // Move cursor down to "Short" line (line 1) - should clamp to end of short line
        let events = manager.move_cursor_down();
        assert!(!events.is_empty());

        let cursor_after_short = manager.get_current_cursor_position();
        tracing::debug!("After moving to short line: {:?}", cursor_after_short);

        // Cursor should be at the end of "Short" line (much less than 50)
        assert_eq!(cursor_after_short.line, 1);
        assert!(
            cursor_after_short.column < target_column,
            "Cursor should be clamped to shorter line"
        );

        // But virtual column should still be 50
        assert_eq!(
            manager.panes[manager.current_pane].get_virtual_column(),
            target_column,
            "Virtual column should be preserved"
        );

        // Move down again to medium line - should be positioned further right than on short line
        let events = manager.move_cursor_down();
        assert!(!events.is_empty());

        let cursor_after_medium = manager.get_current_cursor_position();
        tracing::debug!("After moving to medium line: {:?}", cursor_after_medium);

        // Should be on medium line (line 2) and positioned further right than on short line
        assert_eq!(cursor_after_medium.line, 2);
        assert!(
            cursor_after_medium.column > cursor_after_short.column,
            "Cursor should be positioned further right on longer line"
        );

        // Move down to very short line "X" - should clamp to position 0 (only one character)
        let events = manager.move_cursor_down();
        assert!(!events.is_empty());

        let cursor_after_x = manager.get_current_cursor_position();
        tracing::debug!("After moving to 'X' line: {:?}", cursor_after_x);

        // Should be on "X" line (line 3) and at position 0 (clamped)
        assert_eq!(cursor_after_x.line, 3);
        assert_eq!(
            cursor_after_x.column, 0,
            "Cursor should be clamped to single character line"
        );

        // Virtual column should still be preserved
        assert_eq!(
            manager.panes[manager.current_pane].get_virtual_column(),
            target_column,
            "Virtual column should still be preserved"
        );

        // Move down to the last long line - should restore to near original position
        let events = manager.move_cursor_down();
        assert!(!events.is_empty());

        let cursor_after_long = manager.get_current_cursor_position();
        tracing::debug!("After moving to long line: {:?}", cursor_after_long);

        // Should be on long line (line 4) and restored to target column or close to it
        assert_eq!(cursor_after_long.line, 4);
        assert!(
            cursor_after_long.column >= target_column - 5, // Allow some tolerance
            "Cursor should be restored to near original position: expected ~{}, got {}",
            target_column,
            cursor_after_long.column
        );

        // Virtual column should still be preserved
        assert_eq!(
            manager.panes[manager.current_pane].get_virtual_column(),
            target_column,
            "Virtual column should be preserved throughout navigation"
        );

        tracing::debug!("Virtual column behavior test completed successfully");
    }

    // Tests for move_cursor_page_up functionality
    #[test]
    fn move_cursor_page_up_should_work() {
        let mut manager = PaneManager::new((80, 24));

        // Set up content with many lines to test page up functionality
        let content = "Line 1\nLine 2\nLine 3\nLine 4\nLine 5\nLine 6\nLine 7\nLine 8\nLine 9\nLine 10\nLine 11\nLine 12\nLine 13\nLine 14\nLine 15\nLine 16\nLine 17\nLine 18\nLine 19\nLine 20\nLine 21\nLine 22\nLine 23\nLine 24\nLine 25\nLine 26\nLine 27\nLine 28\nLine 29\nLine 30";
        manager.set_request_content(content);

        // Rebuild display caches to ensure content is processed
        let content_width = manager.get_content_width();
        manager.rebuild_display_caches(content_width);

        // Position cursor near the end of content (line 25)
        let start_line = 25;
        manager.panes[manager.current_pane].display_cursor = Position::new(start_line, 0);

        // Update virtual column and sync logical cursor
        manager.panes[manager.current_pane].update_virtual_column();
        if let Some(logical_pos) = manager.panes[manager.current_pane]
            .display_cache
            .display_to_logical_position(start_line, 0)
        {
            let logical_position = LogicalPosition::new(logical_pos.row, logical_pos.col);
            manager.panes[manager.current_pane]
                .buffer
                .set_cursor(logical_position);
        }

        let initial_cursor = manager.get_current_cursor_position();
        assert_eq!(initial_cursor.line, start_line);
        assert_eq!(initial_cursor.column, 0);

        // Debug: check line count and pane height
        let line_count = manager.panes[manager.current_pane]
            .display_cache
            .display_line_count();
        let pane_height = manager.panes[manager.current_pane].pane_dimensions.height;
        tracing::debug!(
            "Test page up: start_line={}, pane_height={}, line_count={}",
            start_line,
            pane_height,
            line_count
        );

        // Perform page up
        let events = manager.move_cursor_page_up();

        // Should have generated events for cursor update
        assert!(
            !events.is_empty(),
            "Expected events for page up but got none. pane_height={pane_height}, line_count={line_count}"
        );
        assert!(events.iter().any(|e| matches!(
            e,
            crate::repl::events::ViewEvent::ActiveCursorUpdateRequired
        )));

        // Cursor should have moved up by page size (pane height)
        let new_cursor = manager.get_current_cursor_position();
        let expected_new_line = start_line.saturating_sub(pane_height);
        tracing::debug!(
            "Test page up: moved from line {} to line {}, expected line {}",
            initial_cursor.line,
            new_cursor.line,
            expected_new_line
        );

        assert_eq!(
            new_cursor.line, expected_new_line,
            "Cursor should have moved from line {} to line {}, but got line {}",
            start_line, expected_new_line, new_cursor.line
        );
        assert_eq!(new_cursor.column, 0); // Column should remain at 0
    }

    #[test]
    fn move_cursor_page_up_should_not_move_above_first_line() {
        let mut manager = PaneManager::new((80, 24));

        // Set up some content
        let content = "Line 1\nLine 2\nLine 3";
        manager.set_request_content(content);

        // Rebuild display caches
        let content_width = manager.get_content_width();
        manager.rebuild_display_caches(content_width);

        // Start at line 0 (already at top)
        let initial_cursor = manager.get_current_cursor_position();
        assert_eq!(initial_cursor.line, 0);

        // Try to page up - should not move since we're already at the top
        let events = manager.move_cursor_page_up();

        // Should return empty events since no movement occurred
        assert!(events.is_empty());

        // Cursor should stay at line 0
        let cursor = manager.get_current_cursor_position();
        assert_eq!(cursor.line, 0);
    }

    #[test]
    fn move_cursor_page_up_should_handle_empty_content() {
        let mut manager = PaneManager::new((80, 24));

        // Empty content (use default empty content)

        // Try to page up
        let events = manager.move_cursor_page_up();

        // Should return empty events since there's no content
        assert!(events.is_empty());

        // Cursor should remain at origin
        let cursor = manager.get_current_cursor_position();
        assert_eq!(cursor.line, 0);
        assert_eq!(cursor.column, 0);
    }

    #[test]
    fn move_cursor_page_up_should_maintain_virtual_column() {
        let mut manager = PaneManager::new((80, 10)); // Small height for easier testing

        // Create content with varying line lengths
        let mut content_lines = Vec::new();
        for i in 0..30 {
            if i % 4 == 0 {
                content_lines.push("This is a very long line that extends well beyond most other lines in this test content");
            } else if i % 3 == 0 {
                content_lines.push("Short");
            } else {
                content_lines.push("Medium length line here");
            }
        }

        let content = content_lines.join("\n");
        manager.set_request_content(&content);

        // Rebuild display caches
        let content_width = manager.get_content_width();
        manager.rebuild_display_caches(content_width);

        // Position cursor at column 30 on line 25 (a long line)
        let start_line = 25;
        let target_virtual_column = 30;
        manager.panes[manager.current_pane].display_cursor =
            Position::new(start_line, target_virtual_column);

        // Set virtual column to remember this position
        manager.panes[manager.current_pane].set_virtual_column(target_virtual_column);

        // Sync logical cursor
        if let Some(logical_pos) = manager.panes[manager.current_pane]
            .display_cache
            .display_to_logical_position(start_line, target_virtual_column)
        {
            let logical_position = LogicalPosition::new(logical_pos.row, logical_pos.col);
            manager.panes[manager.current_pane]
                .buffer
                .set_cursor(logical_position);
        }

        // Perform page up
        let events = manager.move_cursor_page_up();
        assert!(!events.is_empty());

        // Virtual column should be preserved
        assert_eq!(
            manager.panes[manager.current_pane].get_virtual_column(),
            target_virtual_column,
            "Virtual column should be preserved after page up"
        );

        // Cursor should have moved up by page size
        let pane_height = manager.panes[manager.current_pane].pane_dimensions.height;
        let new_cursor = manager.get_current_cursor_position();
        let expected_line = start_line.saturating_sub(pane_height);

        assert_eq!(
            new_cursor.line, expected_line,
            "Cursor should be on line {expected_line} after page up from line {start_line}"
        );
    }

    #[test]
    fn move_cursor_page_down_should_maintain_virtual_column() {
        let mut manager = PaneManager::new((80, 10)); // Small height for easier page down testing

        // Create content with many lines of varying lengths
        let mut content_lines = Vec::new();
        content_lines.push("This is a very long first line that extends well beyond most other lines in this test content");
        for i in 1..20 {
            if i % 4 == 0 {
                content_lines.push("X"); // Very short line
            } else if i % 3 == 0 {
                content_lines.push("Medium length line here");
            } else {
                content_lines.push("Short");
            }
        }
        // Add another long line at the end
        content_lines.push("This is another very long line that should allow restoration of the virtual column position when reached");

        let content = content_lines.join("\n");
        manager.set_request_content(&content);

        // Rebuild display caches
        let content_width = manager.get_content_width();
        manager.rebuild_display_caches(content_width);

        // Position cursor at column 60 on the first long line
        let target_virtual_column = 60;
        manager.panes[manager.current_pane].display_cursor =
            Position::new(0, target_virtual_column);
        manager.panes[manager.current_pane].set_virtual_column(target_virtual_column);

        // Sync buffer cursor
        if let Some(logical_pos) = manager.panes[manager.current_pane]
            .display_cache
            .display_to_logical_position(0, target_virtual_column)
        {
            manager.panes[manager.current_pane]
                .buffer
                .set_cursor(LogicalPosition::new(logical_pos.row, logical_pos.col));
        }

        tracing::debug!(
            "Starting page down test: line 0, column {}",
            target_virtual_column
        );
        tracing::debug!(
            "Virtual column: {}",
            manager.panes[manager.current_pane].get_virtual_column()
        );

        // Perform page down - this should jump multiple lines down
        let events = manager.move_cursor_page_down();
        assert!(!events.is_empty(), "Page down should produce events");

        let cursor_after_page_down = manager.get_current_cursor_position();
        tracing::debug!("After page down: {:?}", cursor_after_page_down);

        // Should have moved to a different line
        assert!(
            cursor_after_page_down.line > 0,
            "Should have moved to a lower line"
        );

        // Virtual column should still be preserved
        assert_eq!(
            manager.panes[manager.current_pane].get_virtual_column(),
            target_virtual_column,
            "Virtual column should be preserved after page down"
        );

        // The cursor column may be clamped on shorter lines, but should try to get as close as possible
        // to the virtual column on any lines that are long enough

        // If we're on a short line, cursor should be clamped
        let target_line = manager.panes[manager.current_pane].display_cursor.row;
        if let Some(display_line) = manager.panes[manager.current_pane]
            .display_cache
            .get_display_line(target_line)
        {
            let line_length = display_line.char_count();
            if line_length < target_virtual_column {
                // On a short line, cursor should be clamped
                assert!(
                    cursor_after_page_down.column <= line_length.saturating_sub(1),
                    "Cursor should be clamped on short line"
                );
            } else {
                // On a long enough line, cursor should be restored to virtual column
                assert_eq!(
                    cursor_after_page_down.column, target_virtual_column,
                    "Cursor should be restored to virtual column on long line"
                );
            }
        }

        tracing::debug!("Page down virtual column behavior test completed successfully");
    }

    #[test]
    fn move_cursor_page_down_should_handle_dbcs_character_boundaries() {
        let mut manager = PaneManager::new((80, 10));

        // Create content with mixed ASCII and DBCS characters
        // The key is to have lines where the virtual column would land in the middle of DBCS chars
        let content = "This is a normal ASCII line with some characters here for positioning\n日本語の文字列です。これはダブルバイト文字のテストです。\nShort line\n中文字符测试，包含一些双字节字符用于测试光标定位\nEnd";
        manager.set_request_content(content);

        // Rebuild display caches
        let content_width = manager.get_content_width();
        manager.rebuild_display_caches(content_width);

        // Position cursor at column 25 on the ASCII line - this should land in middle of DBCS char on Japanese line
        let target_virtual_column = 25;
        manager.panes[manager.current_pane].display_cursor =
            Position::new(0, target_virtual_column);
        manager.panes[manager.current_pane].set_virtual_column(target_virtual_column);

        // Sync buffer cursor
        if let Some(logical_pos) = manager.panes[manager.current_pane]
            .display_cache
            .display_to_logical_position(0, target_virtual_column)
        {
            manager.panes[manager.current_pane]
                .buffer
                .set_cursor(LogicalPosition::new(logical_pos.row, logical_pos.col));
        }

        tracing::debug!(
            "Starting DBCS test: line 0, column {}",
            target_virtual_column
        );

        // Perform page down - should jump to the Japanese line and snap to character boundary
        let events = manager.move_cursor_page_down();
        assert!(!events.is_empty(), "Page down should produce events");

        let cursor_after_page_down = manager.get_current_cursor_position();
        tracing::debug!("After page down with DBCS: {:?}", cursor_after_page_down);

        // Should have moved to a different line
        assert!(
            cursor_after_page_down.line > 0,
            "Should have moved to a lower line"
        );

        // Virtual column should still be preserved
        assert_eq!(
            manager.panes[manager.current_pane].get_virtual_column(),
            target_virtual_column,
            "Virtual column should be preserved after page down"
        );

        // Most importantly: the cursor should be positioned at a valid character boundary
        // We can verify this by checking that the cursor position makes logical sense
        // (i.e., it's not in the middle of a DBCS character)

        // Get the display line we landed on
        let target_line = manager.panes[manager.current_pane].display_cursor.row;
        if let Some(display_line) = manager.panes[manager.current_pane]
            .display_cache
            .get_display_line(target_line)
        {
            // The cursor column should be valid (not in middle of DBCS char)
            // We can test this by ensuring snap_to_character_boundary returns the same position
            let current_col = cursor_after_page_down.column;
            let snapped_col = display_line.snap_to_character_boundary(current_col);

            assert_eq!(
                current_col, snapped_col,
                "Cursor should already be at a valid character boundary, but was at {current_col} and should be at {snapped_col}"
            );

            tracing::debug!(
                "DBCS boundary test: cursor at column {}, snapped to column {} ✓",
                current_col,
                snapped_col
            );
        }

        tracing::debug!("DBCS character boundary test completed successfully");
    }

    #[test]
    fn move_cursor_page_down_should_clamp_column_to_line_width() {
        let mut manager = PaneManager::new((80, 24));

        // Set up content with varying line lengths
        let content = "This is a very long line that extends beyond most other lines\nShort\nMedium line here\nX\nAnother long line that should test column clamping behavior properly\nTiny\nNormal length line\nA\nMore content for testing\nEnd";
        manager.set_request_content(content);

        // Rebuild display caches
        let content_width = manager.get_content_width();
        manager.rebuild_display_caches(content_width);

        // Position cursor at the end of the first long line
        let long_line_length =
            "This is a very long line that extends beyond most other lines".len();
        manager.panes[manager.current_pane].display_cursor = Position::new(0, long_line_length - 1);

        // Sync buffer cursor with display cursor
        if let Some(logical_pos) = manager.panes[manager.current_pane]
            .display_cache
            .display_to_logical_position(0, long_line_length - 1)
        {
            manager.panes[manager.current_pane]
                .buffer
                .set_cursor(LogicalPosition::new(logical_pos.row, logical_pos.col));
        }

        tracing::debug!(
            "Starting cursor position: {:?}",
            manager.get_current_cursor_position()
        );

        // Perform page down - should land on a shorter line and clamp the column
        let events = manager.move_cursor_page_down();

        // Should have moved
        assert!(!events.is_empty());

        // Get the new cursor position
        let new_cursor = manager.get_current_cursor_position();
        tracing::debug!("After page down cursor position: {:?}", new_cursor);

        // The cursor should have moved to a different line
        assert!(
            new_cursor.line > 0,
            "Cursor should have moved down from line 0"
        );

        // The cursor column should be clamped to a reasonable value (not exceeding the line length)
        // Since we don't know exactly which line we'll land on, just verify it's not out of bounds
        // by checking that we can get a valid cursor position (the test would fail if cursor was out of bounds)

        // Additional verification: try to get the display line we landed on
        let target_line = manager.panes[manager.current_pane].display_cursor.row;
        if let Some(display_line) = manager.panes[manager.current_pane]
            .display_cache
            .get_display_line(target_line)
        {
            let line_length = display_line.char_count();
            tracing::debug!(
                "Target line {} has length {}, cursor column is {}",
                target_line,
                line_length,
                new_cursor.column
            );

            // In Normal mode, cursor should not exceed line_length - 1
            if line_length > 0 {
                assert!(
                    new_cursor.column <= line_length.saturating_sub(1),
                    "Cursor column {} should not exceed line length - 1 ({})",
                    new_cursor.column,
                    line_length.saturating_sub(1)
                );
            } else {
                assert_eq!(
                    new_cursor.column, 0,
                    "Empty line should have cursor at column 0"
                );
            }
        }
    }
}
