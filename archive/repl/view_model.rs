//! ViewModel for MVVM architecture
//!
//! The ViewModel coordinates all models and manages display-specific logic like
//! scrolling, wrapping, and converting logical positions to display positions.
//! It subscribes to model events and emits view events for rendering.

use crate::repl::display_cache::DisplayCache;
use crate::repl::events::{EventBus, LogicalPosition, ModelEvent, ViewModelEvent};
use crate::repl::model::{EditorMode, Pane};
use crate::repl::models::{BufferModel, EditorModel, RequestModel, ResponseModel};
use anyhow::Result;

/// Type alias for event bus to reduce complexity
type EventBusType = Option<Box<dyn EventBus>>;

/// Display position in terminal coordinates
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DisplayPosition {
    pub row: usize,
    pub column: usize,
}

/// Scroll state for a pane
#[derive(Debug, Clone)]
pub struct ScrollState {
    /// First visible line (0-based)
    pub top_line: usize,
    /// Horizontal scroll offset
    pub left_column: usize,
}

impl ScrollState {
    pub fn new() -> Self {
        Self {
            top_line: 0,
            left_column: 0,
        }
    }
}

impl Default for ScrollState {
    fn default() -> Self {
        Self::new()
    }
}

/// The ViewModel coordinates all models and handles display concerns
///
/// This is the central coordinator in the MVVM architecture that:
/// - Owns all model instances
/// - Converts logical positions to display positions
/// - Manages scrolling and display cache
/// - Subscribes to model events and emits view events
/// - Handles all display-specific logic without the models knowing about it
pub struct ViewModel {
    /// Editor state model
    pub editor: EditorModel,
    /// Request buffer model
    pub request_buffer: BufferModel,
    /// Response buffer model  
    pub response_buffer: BufferModel,
    /// HTTP request model
    pub request: RequestModel,
    /// HTTP response model
    pub response: ResponseModel,

    /// Display cache for request pane
    request_display_cache: DisplayCache,
    /// Display cache for response pane
    response_display_cache: DisplayCache,

    /// Scroll state for request pane
    request_scroll: ScrollState,
    /// Scroll state for response pane
    response_scroll: ScrollState,

    /// Terminal dimensions
    terminal_width: u16,
    terminal_height: u16,

    /// Event bus for communication
    event_bus: EventBusType,
}

impl ViewModel {
    /// Create a new ViewModel with default state
    pub fn new() -> Self {
        Self {
            editor: EditorModel::new(),
            request_buffer: BufferModel::new(Pane::Request),
            response_buffer: BufferModel::new(Pane::Response),
            request: RequestModel::new(),
            response: ResponseModel::new(),
            request_display_cache: DisplayCache::new(),
            response_display_cache: DisplayCache::new(),
            request_scroll: ScrollState::new(),
            response_scroll: ScrollState::new(),
            terminal_width: 80,
            terminal_height: 24,
            event_bus: None,
        }
    }

    /// Set the event bus for communication
    pub fn set_event_bus(&mut self, event_bus: Box<dyn EventBus>) {
        self.event_bus = Some(event_bus);
    }

    /// Update terminal dimensions
    pub fn update_terminal_size(&mut self, width: u16, height: u16) {
        if self.terminal_width != width || self.terminal_height != height {
            self.terminal_width = width;
            self.terminal_height = height;

            // Invalidate display caches since terminal size changed
            self.request_display_cache.invalidate();
            self.response_display_cache.invalidate();

            self.emit_view_event(ViewModelEvent::FullRedrawRequired);
        }
    }

    /// Get current cursor position for the active pane
    pub fn get_cursor_position(&self) -> LogicalPosition {
        self.editor.get_cursor(self.editor.current_pane)
    }

    /// Convert logical position to display position for a pane
    pub fn logical_to_display(&self, pane: Pane, logical_pos: LogicalPosition) -> DisplayPosition {
        let display_cache = self.get_display_cache(pane);
        let scroll_state = self.get_scroll_state(pane);

        // Find display lines for this logical line
        if let Some(display_indices) = display_cache.logical_to_display.get(&logical_pos.line) {
            // Find which display line contains this column
            for &display_idx in display_indices {
                if let Some(display_line) = display_cache.display_lines.get(display_idx) {
                    if logical_pos.column >= display_line.logical_start_col
                        && logical_pos.column < display_line.logical_end_col
                    {
                        let display_col = logical_pos.column - display_line.logical_start_col;

                        // Apply scroll offset
                        let visible_row = display_idx.saturating_sub(scroll_state.top_line);
                        let visible_col = display_col.saturating_sub(scroll_state.left_column);

                        return DisplayPosition {
                            row: visible_row,
                            column: visible_col,
                        };
                    }
                }
            }
        }

        // Fallback for invalid positions
        DisplayPosition { row: 0, column: 0 }
    }

    /// Convert display position to logical position for a pane
    pub fn display_to_logical(&self, pane: Pane, display_pos: DisplayPosition) -> LogicalPosition {
        let display_cache = self.get_display_cache(pane);
        let scroll_state = self.get_scroll_state(pane);

        // Apply scroll offset to get absolute display position
        let abs_display_row = display_pos.row + scroll_state.top_line;
        let abs_display_col = display_pos.column + scroll_state.left_column;

        // Find display line at this row
        if let Some(display_line) = display_cache.display_lines.get(abs_display_row) {
            let logical_line = display_line.logical_line;
            let column = (display_line.logical_start_col + abs_display_col)
                .min(display_line.logical_end_col);

            return LogicalPosition {
                line: logical_line,
                column,
            };
        }

        // Fallback
        LogicalPosition { line: 0, column: 0 }
    }

    /// Move cursor left and handle auto-scrolling
    pub fn move_cursor_left(&mut self) -> Result<()> {
        let current_pane = self.editor.current_pane;
        let current_pos = self.editor.get_cursor(current_pane);
        let buffer = self.get_buffer(current_pane);

        if let (new_pos, Some(event)) = buffer.move_cursor_left(current_pos) {
            // Update editor cursor
            self.editor.set_cursor(current_pane, new_pos);

            // Handle auto-scrolling if needed
            self.ensure_cursor_visible(current_pane, new_pos);

            // Emit model event
            self.emit_model_event(event);

            // Emit view event for cursor repositioning
            self.emit_view_event(ViewModelEvent::CursorRepositionRequired);
        }

        Ok(())
    }

    /// Move cursor right and handle auto-scrolling
    pub fn move_cursor_right(&mut self) -> Result<()> {
        let current_pane = self.editor.current_pane;
        let current_pos = self.editor.get_cursor(current_pane);
        let buffer = self.get_buffer(current_pane);

        if let (new_pos, Some(event)) = buffer.move_cursor_right(current_pos) {
            // Update editor cursor
            self.editor.set_cursor(current_pane, new_pos);

            // Handle auto-scrolling if needed
            self.ensure_cursor_visible(current_pane, new_pos);

            // Emit model event
            self.emit_model_event(event);

            // Emit view event for cursor repositioning
            self.emit_view_event(ViewModelEvent::CursorRepositionRequired);
        }

        Ok(())
    }

    /// Switch to a different pane
    pub fn switch_pane(&mut self, new_pane: Pane) -> Result<()> {
        if let Some(event) = self.editor.switch_pane(new_pane) {
            self.emit_model_event(event);
            self.emit_view_event(ViewModelEvent::FullRedrawRequired);
        }
        Ok(())
    }

    /// Change editor mode
    pub fn change_mode(&mut self, new_mode: EditorMode) -> Result<()> {
        if let Some(event) = self.editor.change_mode(new_mode) {
            self.emit_model_event(event);
            self.emit_view_event(ViewModelEvent::StatusBarUpdateRequired);
        }
        Ok(())
    }

    /// Insert a character at the current cursor position
    pub fn insert_char(&mut self, ch: char) -> Result<()> {
        let current_pane = self.editor.current_pane;
        let current_pos = self.editor.get_cursor(current_pane);
        let buffer = self.get_buffer_mut(current_pane);

        // Use insert_text with single character
        let event = buffer.content_mut().insert_text(current_pane, current_pos, &ch.to_string());

        // Calculate new cursor position (moved one column right)
        let new_pos = LogicalPosition {
            line: current_pos.line,
            column: current_pos.column + 1,
        };

        // Update editor cursor
        self.editor.set_cursor(current_pane, new_pos);

        // Update display cache since content changed
        let display_cache = self.get_display_cache_mut(current_pane);
        display_cache.invalidate(); // Simple approach - invalidate cache on content change

        // Handle auto-scrolling if needed
        self.ensure_cursor_visible(current_pane, new_pos);

        // Emit model event for content change
        self.emit_model_event(event);

        // Emit view event for pane redraw (content changed)
        self.emit_view_event(ViewModelEvent::PaneRedrawRequired { pane: current_pane });

        Ok(())
    }

    /// Get buffer for the specified pane
    fn get_buffer(&self, pane: Pane) -> &BufferModel {
        match pane {
            Pane::Request => &self.request_buffer,
            Pane::Response => &self.response_buffer,
        }
    }

    /// Get mutable buffer for the specified pane
    fn get_buffer_mut(&mut self, pane: Pane) -> &mut BufferModel {
        match pane {
            Pane::Request => &mut self.request_buffer,
            Pane::Response => &mut self.response_buffer,
        }
    }

    /// Get display cache for the specified pane
    fn get_display_cache(&self, pane: Pane) -> &DisplayCache {
        match pane {
            Pane::Request => &self.request_display_cache,
            Pane::Response => &self.response_display_cache,
        }
    }

    /// Get mutable display cache for the specified pane
    fn get_display_cache_mut(&mut self, pane: Pane) -> &mut DisplayCache {
        match pane {
            Pane::Request => &mut self.request_display_cache,
            Pane::Response => &mut self.response_display_cache,
        }
    }

    /// Get scroll state for the specified pane
    fn get_scroll_state(&self, pane: Pane) -> &ScrollState {
        match pane {
            Pane::Request => &self.request_scroll,
            Pane::Response => &self.response_scroll,
        }
    }

    /// Get mutable scroll state for the specified pane
    fn get_scroll_state_mut(&mut self, pane: Pane) -> &mut ScrollState {
        match pane {
            Pane::Request => &mut self.request_scroll,
            Pane::Response => &mut self.response_scroll,
        }
    }

    /// Ensure cursor is visible by adjusting scroll if needed
    fn ensure_cursor_visible(&mut self, pane: Pane, cursor_pos: LogicalPosition) {
        let display_pos = self.logical_to_display(pane, cursor_pos);
        let terminal_height = self.terminal_height;
        let terminal_width = self.terminal_width;
        let scroll_state = self.get_scroll_state_mut(pane);
        let mut scroll_changed = false;

        // Check vertical scrolling
        let visible_height = terminal_height as usize / 2; // Assuming panes split screen
        if display_pos.row >= visible_height {
            let old_offset = scroll_state.top_line;
            scroll_state.top_line = display_pos.row - visible_height + 1;
            if old_offset != scroll_state.top_line {
                scroll_changed = true;
            }
        } else if display_pos.row < scroll_state.top_line {
            let old_offset = scroll_state.top_line;
            scroll_state.top_line = display_pos.row;
            if old_offset != scroll_state.top_line {
                scroll_changed = true;
            }
        }

        // Check horizontal scrolling
        let visible_width = terminal_width as usize;
        if display_pos.column >= visible_width {
            let old_offset = scroll_state.left_column;
            scroll_state.left_column = display_pos.column - visible_width + 1;
            if old_offset != scroll_state.left_column {
                scroll_changed = true;
            }
        } else if display_pos.column < scroll_state.left_column {
            let old_offset = scroll_state.left_column;
            scroll_state.left_column = display_pos.column;
            if old_offset != scroll_state.left_column {
                scroll_changed = true;
            }
        }

        if scroll_changed {
            let new_offset = scroll_state.top_line;
            let _ = scroll_state; // Release the mutable borrow
            self.emit_view_event(ViewModelEvent::ScrollPositionChanged {
                pane,
                old_offset: 0, // TODO: Track previous offset properly
                new_offset,
            });
        }
    }

    /// Emit a model event through the event bus
    fn emit_model_event(&mut self, event: ModelEvent) {
        if let Some(event_bus) = &mut self.event_bus {
            event_bus.publish_model(event);
        }
    }

    /// Emit a view event through the event bus
    fn emit_view_event(&mut self, event: ViewModelEvent) {
        if let Some(event_bus) = &mut self.event_bus {
            event_bus.publish_view(event);
        }
    }
}

impl Default for ViewModel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn view_model_should_create_with_defaults() {
        let view_model = ViewModel::new();

        assert_eq!(view_model.editor.current_pane, Pane::Request);
        assert_eq!(view_model.editor.mode, EditorMode::Normal);
        assert_eq!(view_model.terminal_width, 80);
        assert_eq!(view_model.terminal_height, 24);
    }

    #[test]
    fn view_model_should_handle_terminal_resize() {
        let mut view_model = ViewModel::new();

        view_model.update_terminal_size(120, 40);

        assert_eq!(view_model.terminal_width, 120);
        assert_eq!(view_model.terminal_height, 40);
    }

    #[test]
    fn view_model_should_get_cursor_position() {
        let view_model = ViewModel::new();

        let cursor_pos = view_model.get_cursor_position();

        assert_eq!(cursor_pos, LogicalPosition { line: 0, column: 0 });
    }

    #[test]
    fn view_model_should_switch_panes() {
        let mut view_model = ViewModel::new();

        let result = view_model.switch_pane(Pane::Response);

        assert!(result.is_ok());
        assert_eq!(view_model.editor.current_pane, Pane::Response);
    }

    #[test]
    fn view_model_should_change_modes() {
        let mut view_model = ViewModel::new();

        let result = view_model.change_mode(EditorMode::Insert);

        assert!(result.is_ok());
        assert_eq!(view_model.editor.mode, EditorMode::Insert);
    }

    #[test]
    fn view_model_should_handle_cursor_movement() {
        let mut view_model = ViewModel::new();

        // Add some content to move within
        view_model.request_buffer.content_mut().insert_text(
            Pane::Request,
            LogicalPosition { line: 0, column: 0 },
            "test content",
        );

        let result = view_model.move_cursor_right();
        assert!(result.is_ok());

        let cursor_pos = view_model.get_cursor_position();
        assert_eq!(cursor_pos, LogicalPosition { line: 0, column: 1 });
    }
}
