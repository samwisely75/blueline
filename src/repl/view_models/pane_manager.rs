//! # Pane Management
//!
//! Handles pane switching, mode changes, and pane-related state management.
//! Contains the PaneManager struct that encapsulates all pane-related operations.

use crate::repl::events::{LogicalPosition, Pane, ViewEvent};
use crate::repl::view_models::pane_state::PaneState;

/// Type alias for visual selection state to reduce complexity
type VisualSelectionState = (
    Option<LogicalPosition>,
    Option<LogicalPosition>,
    Option<Pane>,
);

/// PaneManager encapsulates all pane-related state and operations
/// This eliminates the need for array indexing operations throughout the codebase
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
            wrap_enabled: true,
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
        self.wrap_enabled = enabled;
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

        // Invalidate display caches for both panes
        self.panes[Pane::Request].display_cache.invalidate();
        self.panes[Pane::Response].display_cache.invalidate();

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

    /// Rebuild display caches for both panes
    pub fn rebuild_display_caches(&mut self, content_width: usize) {
        self.panes[Pane::Request].build_display_cache(content_width, self.wrap_enabled);
        self.panes[Pane::Response].build_display_cache(content_width, self.wrap_enabled);
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
            }
        }
    }

    /// Get display cursor position for current pane
    pub fn get_current_display_cursor(&self) -> (usize, usize) {
        self.panes[self.current_pane].display_cursor
    }

    /// Get scroll offset for current pane
    pub fn get_current_scroll_offset(&self) -> (usize, usize) {
        self.panes[self.current_pane].scroll_offset
    }

    /// Ensure cursor is visible in current area
    pub fn ensure_current_cursor_visible(&mut self, content_width: usize) -> Vec<ViewEvent> {
        let result = self.panes[self.current_pane].ensure_cursor_visible(content_width);

        if result.vertical_changed {
            vec![ViewEvent::CurrentAreaScrollChanged {
                old_offset: result.old_vertical_offset,
                new_offset: result.new_vertical_offset,
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
            vec![ViewEvent::RequestContentChanged]
        } else {
            vec![] // Can't edit in display area
        }
    }

    /// Delete character before cursor in Request pane
    pub fn delete_char_before_cursor_in_request(&mut self) -> Vec<ViewEvent> {
        if self.is_in_request_pane() {
            // For now, return basic event - detailed deletion logic can be added later
            vec![ViewEvent::RequestContentChanged]
        } else {
            vec![] // Can't edit in display area
        }
    }

    /// Delete character after cursor in Request pane
    pub fn delete_char_after_cursor_in_request(&mut self) -> Vec<ViewEvent> {
        if self.is_in_request_pane() {
            // For now, return basic event - detailed deletion logic can be added later
            vec![ViewEvent::RequestContentChanged]
        } else {
            vec![] // Can't edit in display area
        }
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

        // Update visual selection if active
        if self.panes[self.current_pane]
            .visual_selection_start
            .is_some()
        {
            self.panes[self.current_pane].visual_selection_end = Some(clamped_position);
        }

        vec![
            ViewEvent::ActiveCursorUpdateRequired,
            ViewEvent::PositionIndicatorUpdateRequired,
        ]
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
        vec![ViewEvent::RequestContentChanged]
    }

    /// Set Response pane content
    pub fn set_response_content(&mut self, text: &str) -> Vec<ViewEvent> {
        self.panes[Pane::Response].buffer = crate::repl::models::BufferModel::new(Pane::Response);
        self.panes[Pane::Response]
            .buffer
            .content_mut()
            .set_text(text);
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

    /// Sync display cursor with logical cursor for current pane
    pub fn sync_current_display_cursor_with_logical(&mut self) -> Vec<ViewEvent> {
        let _result = self.panes[self.current_pane].sync_display_cursor_with_logical();
        vec![]
    }

    /// Set display cursor position for current area
    pub fn set_current_display_cursor(&mut self, position: (usize, usize)) -> Vec<ViewEvent> {
        let _result = self.panes[self.current_pane].set_display_cursor(position);
        vec![
            ViewEvent::ActiveCursorUpdateRequired,
            ViewEvent::PositionIndicatorUpdateRequired,
        ]
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

    /// Handle vertical page scrolling in current area
    pub fn scroll_current_vertically_by_page(&mut self, direction: i32) -> Vec<ViewEvent> {
        let result = self.panes[self.current_pane].scroll_vertically_by_page(direction);

        let mut events = vec![ViewEvent::CurrentAreaScrollChanged {
            old_offset: result.old_offset,
            new_offset: result.new_offset,
        }];

        if result.cursor_moved {
            events.push(ViewEvent::ActiveCursorUpdateRequired);
        }

        events
    }

    /// Handle vertical half-page scrolling in current area
    pub fn scroll_current_vertically_by_half_page(&mut self, direction: i32) -> Vec<ViewEvent> {
        let result = self.panes[self.current_pane].scroll_vertically_by_half_page(direction);

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
    fn get_content_width(&self) -> usize {
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
        if current_display_pos.1 > 0 {
            let new_display_pos = (current_display_pos.0, current_display_pos.1 - 1);
            self.panes[self.current_pane].display_cursor = new_display_pos;
            moved = true;
        } else if current_display_pos.0 > 0 {
            // Move to end of previous display line
            let prev_display_line = current_display_pos.0 - 1;
            if let Some(prev_line) = display_cache.get_display_line(prev_display_line) {
                let new_col = prev_line.content.chars().count().saturating_sub(1);
                let new_display_pos = (prev_display_line, new_col);
                self.panes[self.current_pane].display_cursor = new_display_pos;
                moved = true;
            }
        }

        if moved {
            vec![
                ViewEvent::ActiveCursorUpdateRequired,
                ViewEvent::PositionIndicatorUpdateRequired,
            ]
        } else {
            vec![]
        }
    }

    /// Move cursor right in current area
    pub fn move_cursor_right(&mut self) -> Vec<ViewEvent> {
        let current_display_pos = self.get_current_display_cursor();
        let display_cache = &self.panes[self.current_pane].display_cache;

        let mut moved = false;

        // Get current display line
        if let Some(current_line) = display_cache.get_display_line(current_display_pos.0) {
            let line_char_count = current_line.content.chars().count();

            // Check if we can move right within current display line
            if current_display_pos.1 < line_char_count {
                let new_display_pos = (current_display_pos.0, current_display_pos.1 + 1);
                self.panes[self.current_pane].display_cursor = new_display_pos;
                moved = true;
            } else {
                // Try to move to beginning of next display line
                let next_display_line = current_display_pos.0 + 1;
                if display_cache.get_display_line(next_display_line).is_some() {
                    let new_display_pos = (next_display_line, 0);
                    self.panes[self.current_pane].display_cursor = new_display_pos;
                    moved = true;
                }
            }
        }

        if moved {
            vec![
                ViewEvent::ActiveCursorUpdateRequired,
                ViewEvent::PositionIndicatorUpdateRequired,
            ]
        } else {
            vec![]
        }
    }

    /// Move cursor up in current area
    pub fn move_cursor_up(&mut self) -> Vec<ViewEvent> {
        let current_display_pos = self.get_current_display_cursor();

        if current_display_pos.0 > 0 {
            let new_display_pos = (current_display_pos.0 - 1, current_display_pos.1);
            self.panes[self.current_pane].display_cursor = new_display_pos;
            vec![
                ViewEvent::ActiveCursorUpdateRequired,
                ViewEvent::PositionIndicatorUpdateRequired,
            ]
        } else {
            vec![]
        }
    }

    /// Move cursor down in current area
    pub fn move_cursor_down(&mut self) -> Vec<ViewEvent> {
        let current_display_pos = self.get_current_display_cursor();
        let display_cache = &self.panes[self.current_pane].display_cache;

        let next_display_line = current_display_pos.0 + 1;
        if display_cache.get_display_line(next_display_line).is_some() {
            let new_display_pos = (next_display_line, current_display_pos.1);
            self.panes[self.current_pane].display_cursor = new_display_pos;
            vec![
                ViewEvent::ActiveCursorUpdateRequired,
                ViewEvent::PositionIndicatorUpdateRequired,
            ]
        } else {
            vec![]
        }
    }

    /// Move cursor to start of current line
    pub fn move_cursor_to_start_of_line(&mut self) -> Vec<ViewEvent> {
        let current_display_pos = self.get_current_display_cursor();
        let new_display_pos = (current_display_pos.0, 0);
        self.panes[self.current_pane].display_cursor = new_display_pos;

        vec![
            ViewEvent::ActiveCursorUpdateRequired,
            ViewEvent::PositionIndicatorUpdateRequired,
        ]
    }

    /// Move cursor to end of current line
    pub fn move_cursor_to_end_of_line(&mut self) -> Vec<ViewEvent> {
        let current_display_pos = self.get_current_display_cursor();
        let display_cache = &self.panes[self.current_pane].display_cache;

        if let Some(current_line) = display_cache.get_display_line(current_display_pos.0) {
            let line_char_count = current_line.content.chars().count();
            let new_display_pos = (current_display_pos.0, line_char_count);
            self.panes[self.current_pane].display_cursor = new_display_pos;
        }

        vec![
            ViewEvent::ActiveCursorUpdateRequired,
            ViewEvent::PositionIndicatorUpdateRequired,
        ]
    }

    /// Move cursor to start of document
    pub fn move_cursor_to_document_start(&mut self) -> Vec<ViewEvent> {
        self.panes[self.current_pane].display_cursor = (0, 0);
        vec![
            ViewEvent::ActiveCursorUpdateRequired,
            ViewEvent::PositionIndicatorUpdateRequired,
        ]
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

        if let Some(last_line) = display_cache.get_display_line(last_line_idx) {
            let line_char_count = last_line.content.chars().count();
            self.panes[self.current_pane].display_cursor = (last_line_idx, line_char_count);
        }

        vec![
            ViewEvent::ActiveCursorUpdateRequired,
            ViewEvent::PositionIndicatorUpdateRequired,
        ]
    }

    /// Move cursor to specific line number (1-based)
    pub fn move_cursor_to_line(&mut self, line_number: usize) -> Vec<ViewEvent> {
        if line_number == 0 {
            return vec![];
        }

        let display_cache = &self.panes[self.current_pane].display_cache;
        let target_line_idx = line_number - 1; // Convert to 0-based

        if display_cache.get_display_line(target_line_idx).is_some() {
            self.panes[self.current_pane].display_cursor = (target_line_idx, 0);
            vec![
                ViewEvent::ActiveCursorUpdateRequired,
                ViewEvent::PositionIndicatorUpdateRequired,
            ]
        } else {
            vec![]
        }
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
}
