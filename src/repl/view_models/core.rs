//! # Core ViewModel Structure
//!
//! Contains the main ViewModel struct and basic initialization logic.
//! This is the central coordinator that delegates to specialized managers.

use crate::repl::events::{EditorMode, EventBus, LogicalPosition, ModelEvent, Pane, ViewEvent};
use crate::repl::models::{BufferModel, DisplayCache, ResponseModel};
use crate::repl::view_models::screen_buffer::ScreenBuffer;
// use anyhow::Result; // Currently unused
use bluenote::HttpClient;
use std::collections::HashMap;
use std::ops::{Index, IndexMut, Not};

/// Type alias for event bus option to reduce complexity
type EventBusOption = Option<Box<dyn EventBus>>;

/// Type alias for display line rendering data: (content, line_number, is_continuation, logical_start_col, logical_line)
pub type DisplayLineData = (String, Option<usize>, bool, usize, usize);

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
    pub old_display_pos: (usize, usize),
    pub new_display_pos: (usize, usize),
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
#[derive(Debug, Clone)]
pub struct PaneState {
    pub buffer: BufferModel,
    pub display_cache: DisplayCache,
    pub display_cursor: (usize, usize), // (display_line, display_column)
    pub scroll_offset: (usize, usize),  // (vertical, horizontal)
    pub visual_selection_start: Option<LogicalPosition>,
    pub visual_selection_end: Option<LogicalPosition>,
    pub pane_dimensions: (usize, usize), // (width, height)
}

impl PaneState {
    fn new(pane: Pane, pane_width: usize, pane_height: usize, wrap_enabled: bool) -> Self {
        let mut pane_state = Self {
            buffer: BufferModel::new(pane),
            display_cache: DisplayCache::new(),
            display_cursor: (0, 0),
            scroll_offset: (0, 0),
            visual_selection_start: None,
            visual_selection_end: None,
            pane_dimensions: (pane_width, pane_height),
        };
        pane_state.build_display_cache(pane_width, wrap_enabled);
        pane_state
    }

    /// Build display cache for this pane's content
    pub fn build_display_cache(&mut self, content_width: usize, wrap_enabled: bool) {
        let lines = self.buffer.content().lines().to_vec();
        self.display_cache =
            crate::repl::models::build_display_cache(&lines, content_width, wrap_enabled)
                .unwrap_or_else(|_| DisplayCache::new());
    }

    /// Get page size for scrolling (pane height minus UI chrome)
    pub fn get_page_size(&self) -> usize {
        self.pane_dimensions.1.saturating_sub(2).max(1)
    }

    /// Get half page size for scrolling
    pub fn get_half_page_size(&self) -> usize {
        (self.pane_dimensions.1 / 2).max(1)
    }

    /// Get content width for this pane
    pub fn get_content_width(&self) -> usize {
        self.pane_dimensions.0
    }

    /// Update pane dimensions (for terminal resize)
    pub fn update_dimensions(&mut self, width: usize, height: usize) {
        self.pane_dimensions = (width, height);
    }

    /// Handle horizontal scrolling within this pane
    pub fn scroll_horizontally(&mut self, direction: i32, amount: usize) -> ScrollResult {
        use crate::repl::events::LogicalPosition;

        let old_offset = self.scroll_offset.1; // horizontal offset
        let new_offset = if direction > 0 {
            old_offset + amount
        } else {
            old_offset.saturating_sub(amount)
        };

        self.scroll_offset.1 = new_offset;

        // Handle cursor repositioning to stay visible after horizontal scroll
        let current_cursor = self.buffer.cursor();
        let mut cursor_moved = false;

        // Convert current logical position to display coordinates
        if let Some(display_pos) = self
            .display_cache
            .logical_to_display_position(current_cursor.line, current_cursor.column)
        {
            // Check if cursor is still visible after horizontal scroll
            let content_width = self.get_content_width();

            // If cursor is off-screen, move it to the first/last visible column
            let new_cursor_column = if display_pos.1 < new_offset {
                // Cursor is off-screen to the left, move to first visible column
                new_offset
            } else if display_pos.1 >= new_offset + content_width {
                // Cursor is off-screen to the right, move to last visible column
                new_offset + content_width - 1
            } else {
                // Cursor is still visible, keep current position
                display_pos.1
            };

            // Convert back to logical position and update cursor if needed
            if let Some(logical_pos) = self
                .display_cache
                .display_to_logical_position(display_pos.0, new_cursor_column)
            {
                let new_cursor_position = LogicalPosition::new(logical_pos.0, logical_pos.1);
                let clamped_position = self.buffer.content().clamp_position(new_cursor_position);

                if clamped_position != current_cursor {
                    self.buffer.set_cursor(clamped_position);
                    cursor_moved = true;
                }
            }
        }

        ScrollResult {
            old_offset,
            new_offset,
            cursor_moved,
        }
    }

    /// Handle vertical page scrolling within this pane
    pub fn scroll_vertically_by_page(&mut self, direction: i32) -> ScrollResult {
        use crate::repl::events::LogicalPosition;

        let old_offset = self.scroll_offset.0; // vertical offset
        let page_size = self.get_page_size();

        // Vim typically scrolls by (page_size - 1) to maintain some context
        let scroll_amount = page_size.saturating_sub(1).max(1);
        
        tracing::debug!("scroll_vertically_by_page: pane_dimensions=({}, {}), page_size={}, scroll_amount={}", 
            self.pane_dimensions.0, self.pane_dimensions.1, page_size, scroll_amount);

        // Prevent scrolling beyond actual content bounds
        let max_scroll_offset = self
            .display_cache
            .display_line_count()
            .saturating_sub(page_size)
            .max(0);

        let new_offset = if direction > 0 {
            std::cmp::min(old_offset + scroll_amount, max_scroll_offset)
        } else {
            old_offset.saturating_sub(scroll_amount)
        };

        // If scroll offset wouldn't change, don't do anything
        if new_offset == old_offset {
            return ScrollResult {
                old_offset,
                new_offset: old_offset,
                cursor_moved: false,
            };
        }

        self.scroll_offset.0 = new_offset;

        // BUGFIX: Move cursor by exactly the scroll amount in display coordinates
        // This should be simple: if we scroll by N display lines, cursor moves by N display lines
        let current_cursor = self.buffer.cursor();
        let mut cursor_moved = false;

        tracing::debug!("scroll_vertically_by_page: old_offset={}, new_offset={}, scroll_amount={}, current_cursor=({}, {})",
            old_offset, new_offset, scroll_amount, current_cursor.line, current_cursor.column);

        // Get current cursor display position
        if let Some(current_display_pos) = self
            .display_cache
            .logical_to_display_position(current_cursor.line, current_cursor.column)
        {
            // Move cursor by exactly the scroll amount in display lines
            let scroll_delta = new_offset as i32 - old_offset as i32;
            let new_display_line = (current_display_pos.0 as i32 + scroll_delta).max(0) as usize;
            let new_display_col = current_display_pos.1; // Keep same column position

            tracing::debug!("scroll_vertically_by_page: current_display=({}, {}), scroll_delta={}, new_display=({}, {})",
                current_display_pos.0, current_display_pos.1, scroll_delta, new_display_line, new_display_col);

            // Convert new display position back to logical position
            if let Some(logical_pos) = self
                .display_cache
                .display_to_logical_position(new_display_line, new_display_col)
            {
                let cursor_position = LogicalPosition::new(logical_pos.0, logical_pos.1);
                let clamped_position = self.buffer.content().clamp_position(cursor_position);
                
                tracing::debug!("scroll_vertically_by_page: new_logical=({}, {}), clamped=({}, {})",
                    logical_pos.0, logical_pos.1, clamped_position.line, clamped_position.column);
                
                // Update cursor position
                if clamped_position != current_cursor {
                    self.buffer.set_cursor(clamped_position);
                    self.display_cursor = (new_display_line, new_display_col);
                    cursor_moved = true;
                }
            }
        }

        ScrollResult {
            old_offset,
            new_offset,
            cursor_moved,
        }
    }

    /// Handle vertical half-page scrolling within this pane
    pub fn scroll_vertically_by_half_page(&mut self, direction: i32) -> ScrollResult {
        use crate::repl::events::LogicalPosition;

        let old_offset = self.scroll_offset.0; // vertical offset
        let page_size = self.get_page_size();
        let scroll_amount = self.get_half_page_size();

        // Prevent half-page scrolling beyond actual content bounds
        let max_scroll_offset = self
            .display_cache
            .display_line_count()
            .saturating_sub(page_size)
            .max(0);

        let new_offset = if direction > 0 {
            std::cmp::min(old_offset + scroll_amount, max_scroll_offset)
        } else {
            old_offset.saturating_sub(scroll_amount)
        };

        // If scroll offset wouldn't change, don't do anything
        if new_offset == old_offset {
            return ScrollResult {
                old_offset,
                new_offset: old_offset,
                cursor_moved: false,
            };
        }

        self.scroll_offset.0 = new_offset;

        // BUGFIX: Move cursor by exactly the scroll amount in display coordinates
        // Simple approach: cursor moves by the same amount as the scroll
        let current_cursor = self.buffer.cursor();
        let mut cursor_moved = false;

        // Get current cursor display position
        if let Some(current_display_pos) = self
            .display_cache
            .logical_to_display_position(current_cursor.line, current_cursor.column)
        {
            // Move cursor by exactly the scroll amount in display lines
            let scroll_delta = new_offset as i32 - old_offset as i32;
            let new_display_line = (current_display_pos.0 as i32 + scroll_delta).max(0) as usize;
            let new_display_col = current_display_pos.1; // Keep same column position

            // Convert new display position back to logical position
            if let Some(logical_pos) = self
                .display_cache
                .display_to_logical_position(new_display_line, new_display_col)
            {
                let cursor_position = LogicalPosition::new(logical_pos.0, logical_pos.1);
                let clamped_position = self.buffer.content().clamp_position(cursor_position);
                
                // Update cursor position
                if clamped_position != current_cursor {
                    self.buffer.set_cursor(clamped_position);
                    self.display_cursor = (new_display_line, new_display_col);
                    cursor_moved = true;
                }
            }
        }

        ScrollResult {
            old_offset,
            new_offset,
            cursor_moved,
        }
    }

    /// Set display cursor position for this pane with proper clamping
    pub fn set_display_cursor(&mut self, position: (usize, usize)) -> CursorMoveResult {
        use crate::repl::events::LogicalPosition;
        
        let old_display_pos = self.display_cursor;
        
        tracing::debug!("PaneState::set_display_cursor: requested_pos={:?}", position);

        // Convert to logical position first (this will clamp if needed)
        if let Some(logical_pos) = self.display_cache
            .display_to_logical_position(position.0, position.1)
        {
            let logical_position = LogicalPosition::new(logical_pos.0, logical_pos.1);
            tracing::debug!("PaneState::set_display_cursor: converted display ({}, {}) to logical ({}, {})", 
                position.0, position.1, logical_position.line, logical_position.column);
            
            // Update logical cursor
            self.buffer.set_cursor(logical_position);
            
            // Set display cursor to the actual position that corresponds to the clamped logical position
            if let Some(actual_display_pos) = self.display_cache
                .logical_to_display_position(logical_position.line, logical_position.column)
            {
                self.display_cursor = actual_display_pos;
                tracing::debug!("PaneState::set_display_cursor: updated display cursor to actual position {:?}", actual_display_pos);
            } else {
                self.display_cursor = position;
            }
        } else {
            tracing::warn!("PaneState::set_display_cursor: failed to convert display position {:?} to logical", position);
            self.display_cursor = position;
        }

        let cursor_moved = self.display_cursor != old_display_pos;
        
        CursorMoveResult {
            cursor_moved,
            old_display_pos,
            new_display_pos: self.display_cursor,
        }
    }

    /// Synchronize display cursor with logical cursor position
    pub fn sync_display_cursor_with_logical(&mut self) -> CursorMoveResult {
        let old_display_pos = self.display_cursor;
        let logical_pos = self.buffer.cursor();

        if let Some(display_pos) = self.display_cache
            .logical_to_display_position(logical_pos.line, logical_pos.column)
        {
            tracing::debug!("PaneState::sync_display_cursor_with_logical: converted logical ({}, {}) to display ({}, {})", 
                logical_pos.line, logical_pos.column, display_pos.0, display_pos.1);
            self.display_cursor = display_pos;
        } else {
            tracing::warn!("PaneState::sync_display_cursor_with_logical: failed to convert logical ({}, {}) to display", 
                logical_pos.line, logical_pos.column);
        }

        let cursor_moved = self.display_cursor != old_display_pos;
        
        CursorMoveResult {
            cursor_moved,
            old_display_pos,
            new_display_pos: self.display_cursor,
        }
    }

    /// Ensure cursor is visible within the viewport, adjusting scroll offsets if needed
    pub fn ensure_cursor_visible(&mut self, content_width: usize) -> ScrollAdjustResult {
        let display_pos = self.display_cursor;
        let (old_vertical_offset, old_horizontal_offset) = self.scroll_offset;
        let pane_height = self.pane_dimensions.1;

        tracing::debug!("PaneState::ensure_cursor_visible: display_pos=({}, {}), scroll_offset=({}, {}), pane_size=({}, {})",
            display_pos.0, display_pos.1, old_vertical_offset, old_horizontal_offset, content_width, pane_height);

        let mut new_vertical_offset = old_vertical_offset;
        let mut new_horizontal_offset = old_horizontal_offset;

        // Vertical scrolling to keep cursor within visible area
        if display_pos.0 < old_vertical_offset {
            new_vertical_offset = display_pos.0;
        } else if display_pos.0 >= old_vertical_offset + pane_height && pane_height > 0 {
            new_vertical_offset = display_pos.0.saturating_sub(pane_height.saturating_sub(1));
        }

        // Horizontal scrolling
        if display_pos.1 < old_horizontal_offset {
            new_horizontal_offset = display_pos.1;
            tracing::debug!("PaneState::ensure_cursor_visible: cursor off-screen left, adjusting horizontal offset to {}", new_horizontal_offset);
        } else if display_pos.1 >= old_horizontal_offset + content_width && content_width > 0 {
            new_horizontal_offset = display_pos.1.saturating_sub(content_width.saturating_sub(1));
            tracing::debug!("PaneState::ensure_cursor_visible: cursor off-screen right at pos {}, adjusting horizontal offset from {} to {}", display_pos.1, old_horizontal_offset, new_horizontal_offset);
        }

        // Update scroll offset if changed
        let vertical_changed = new_vertical_offset != old_vertical_offset;
        let horizontal_changed = new_horizontal_offset != old_horizontal_offset;
        
        if vertical_changed || horizontal_changed {
            tracing::debug!("PaneState::ensure_cursor_visible: adjusting scroll from ({}, {}) to ({}, {})",
                old_vertical_offset, old_horizontal_offset, new_vertical_offset, new_horizontal_offset);
            self.scroll_offset = (new_vertical_offset, new_horizontal_offset);
        } else {
            tracing::debug!("PaneState::ensure_cursor_visible: no scroll adjustment needed");
        }

        ScrollAdjustResult {
            vertical_changed,
            horizontal_changed,
            old_vertical_offset,
            new_vertical_offset,
            old_horizontal_offset,
            new_horizontal_offset,
        }
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

/// Enable pane switching with !current_pane
impl Not for Pane {
    type Output = Self;
    fn not(self) -> Self::Output {
        match self {
            Pane::Request => Pane::Response,
            Pane::Response => Pane::Request,
        }
    }
}

/// The central ViewModel that coordinates all business logic
pub struct ViewModel {
    // Core state
    pub(super) editor_mode: EditorMode,
    pub(super) current_pane: Pane,
    pub(super) response: ResponseModel,

    // Pane management - replaces 8 duplicate fields with unified structure
    pub(super) panes: [PaneState; 2],

    // Display coordination (word wrap support)
    pub(super) wrap_enabled: bool,

    // Display state
    pub(super) terminal_dimensions: (u16, u16), // (width, height)
    pub(super) request_pane_height: u16,

    // Ex command mode state (for :q, :w, etc.)
    pub(super) ex_command_buffer: String,

    // Request execution state
    pub(super) is_executing_request: bool,

    // HTTP client and configuration
    pub(super) http_client: Option<HttpClient>,
    pub(super) http_session_headers: HashMap<String, String>,
    pub(super) http_verbose: bool,

    // Profile information
    pub(super) profile_name: String,
    pub(super) profile_path: String,

    // Status message for temporary display
    pub(super) status_message: Option<String>,

    // Event management
    pub(super) event_bus: EventBusOption,
    pub(super) pending_view_events: Vec<ViewEvent>,
    pub(super) pending_model_events: Vec<ModelEvent>,

    // Double buffering state
    pub(super) current_screen_buffer: ScreenBuffer,
    pub(super) previous_screen_buffer: ScreenBuffer,
}

impl ViewModel {
    /// Create a new ViewModel with default state
    pub fn new() -> Self {
        let response = ResponseModel::new();

        // Default terminal size
        let terminal_dimensions = (80, 24);

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
            editor_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            response,
            panes: [request_pane, response_pane],
            wrap_enabled: true,
            terminal_dimensions,
            request_pane_height: terminal_dimensions.1 / 2,
            ex_command_buffer: String::new(),
            is_executing_request: false,
            http_client: None,
            http_session_headers: HashMap::new(),
            http_verbose: false,
            profile_name: "default".to_string(),
            profile_path: "~/.blueline/profile".to_string(),
            status_message: None,
            event_bus: None,
            pending_view_events: Vec::new(),
            pending_model_events: Vec::new(),
            current_screen_buffer: ScreenBuffer::new(
                terminal_dimensions.0 as usize,
                terminal_dimensions.1 as usize,
            ),
            previous_screen_buffer: ScreenBuffer::new(
                terminal_dimensions.0 as usize,
                terminal_dimensions.1 as usize,
            ),
        }
    }

    /// Set the event bus for this ViewModel
    pub fn set_event_bus(&mut self, event_bus: Box<dyn EventBus>) {
        self.event_bus = Some(event_bus);
        tracing::debug!("Event bus set for ViewModel");
    }

    /// Update terminal size and resize screen buffers
    pub fn update_terminal_size(&mut self, width: u16, height: u16) {
        self.terminal_dimensions = (width, height);

        // Calculate request pane height (split screen when response exists)
        self.request_pane_height = if self.response.status_code().is_some() {
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

        // Resize screen buffers
        self.current_screen_buffer
            .resize(width as usize, height as usize);
        self.previous_screen_buffer
            .resize(width as usize, height as usize);

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

    /// Get current screen buffer dimensions
    pub fn screen_buffer_dimensions(&self) -> (usize, usize) {
        self.current_screen_buffer.dimensions()
    }

    /// Swap screen buffers (for double buffering)
    pub fn swap_screen_buffers(&mut self) {
        std::mem::swap(
            &mut self.current_screen_buffer,
            &mut self.previous_screen_buffer,
        );
        self.current_screen_buffer.clear();
    }

    /// Get changed rows between current and previous screen buffers
    pub fn get_screen_buffer_diff(&self) -> Vec<usize> {
        self.current_screen_buffer
            .diff(&self.previous_screen_buffer)
    }

    /// Get reference to current screen buffer (for rendering)
    pub fn current_screen_buffer(&self) -> &ScreenBuffer {
        &self.current_screen_buffer
    }

    /// Get mutable reference to current screen buffer (for building)
    pub fn current_screen_buffer_mut(&mut self) -> &mut ScreenBuffer {
        &mut self.current_screen_buffer
    }

    /// Get terminal size
    pub fn terminal_size(&self) -> (u16, u16) {
        self.terminal_dimensions
    }

    /// Set the profile information for display
    pub fn set_profile_info(&mut self, profile_name: String, profile_path: String) {
        self.profile_name = profile_name;
        self.profile_path = profile_path;
    }

    /// Get the current profile name
    pub fn get_profile_name(&self) -> &str {
        &self.profile_name
    }

    /// Get the current profile path
    pub fn get_profile_path(&self) -> &str {
        &self.profile_path
    }

    // === Pane Access Methods ===

    /// Get the current active pane
    pub fn current_pane(&self) -> Pane {
        self.current_pane
    }

    /// Switch to the other pane
    pub fn toggle_pane(&mut self) {
        self.current_pane = !self.current_pane;
    }

    /// Get immutable reference to request pane
    pub fn request_pane(&self) -> &PaneState {
        &self.panes[Pane::Request]
    }

    /// Get mutable reference to request pane
    pub fn request_pane_mut(&mut self) -> &mut PaneState {
        &mut self.panes[Pane::Request]
    }

    /// Get immutable reference to response pane
    pub fn response_pane(&self) -> &PaneState {
        &self.panes[Pane::Response]
    }

    /// Get mutable reference to response pane
    pub fn response_pane_mut(&mut self) -> &mut PaneState {
        &mut self.panes[Pane::Response]
    }

    /// Get immutable reference to current active pane
    pub fn current_pane_state(&self) -> &PaneState {
        &self.panes[self.current_pane]
    }

    /// Get mutable reference to current active pane
    pub fn current_pane_state_mut(&mut self) -> &mut PaneState {
        &mut self.panes[self.current_pane]
    }

    /// Set a temporary status message for display
    pub fn set_status_message<S: Into<String>>(&mut self, message: S) {
        self.status_message = Some(message.into());
    }

    /// Clear the status message
    pub fn clear_status_message(&mut self) {
        self.status_message = None;
    }

    /// Get the current status message
    pub fn get_status_message(&self) -> Option<&str> {
        self.status_message.as_deref()
    }

    // === Editor State Management ===

    /// Get current editor mode
    pub fn mode(&self) -> EditorMode {
        self.editor_mode
    }

    /// Set editor mode, returning event if changed
    pub fn set_mode(&mut self, new_mode: EditorMode) -> Option<ModelEvent> {
        if self.editor_mode != new_mode {
            let old_mode = self.editor_mode;
            self.editor_mode = new_mode;
            Some(ModelEvent::ModeChanged { old_mode, new_mode })
        } else {
            None
        }
    }

    /// Set current pane, returning event if changed
    pub fn set_current_pane(&mut self, new_pane: Pane) -> Option<ModelEvent> {
        if self.current_pane != new_pane {
            let old_pane = self.current_pane;
            self.current_pane = new_pane;
            Some(ModelEvent::PaneSwitched { old_pane, new_pane })
        } else {
            None
        }
    }
}

impl Default for ViewModel {
    fn default() -> Self {
        Self::new()
    }
}
