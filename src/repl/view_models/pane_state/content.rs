//! Content management operations for PaneState
//!
//! This module contains methods for:
//! - Setting request and response content
//! - Clearing editable content
//! - Content manipulation with capability checking

use crate::repl::events::{Pane, PaneCapabilities, ViewEvent};
use crate::repl::geometry::Position;
use crate::repl::models::BufferModel;

use super::PaneState;

impl PaneState {
    /// Clear editable content with capability checking
    pub fn clear_editable_content(&mut self) -> Vec<ViewEvent> {
        // Check if editing is allowed on this pane
        if !self.capabilities.contains(PaneCapabilities::EDITABLE) {
            return vec![]; // Editing not allowed on this pane
        }

        // Create new buffer (same as original implementation)
        self.buffer = BufferModel::new(Pane::Request);

        vec![ViewEvent::RequestContentChanged]
    }

    /// Set request content with capability checking
    pub fn set_request_content(&mut self, text: &str) -> Vec<ViewEvent> {
        // Check if editing is allowed on this pane
        if !self.capabilities.contains(PaneCapabilities::EDITABLE) {
            return vec![]; // Editing not allowed on this pane
        }

        // Create new buffer and set content (same as original implementation)
        self.buffer = BufferModel::new(Pane::Request);
        self.buffer.content_mut().set_text(text);

        // Update line number width after content changes
        self.update_line_number_width();

        vec![ViewEvent::RequestContentChanged]
    }

    /// Set response content (read-only operation, no capability check needed)
    pub fn set_response_content(&mut self, text: &str) -> Vec<ViewEvent> {
        // Response content setting doesn't require EDITABLE capability
        // as this is internal content display, not user editing

        // Create new buffer and set content (same as original implementation)
        self.buffer = BufferModel::new(Pane::Response);
        self.buffer.content_mut().set_text(text);

        // Update line number width after content changes
        self.update_line_number_width();

        // Reset cursor and scroll positions to avoid out-of-bounds issues
        self.display_cursor = Position::origin();
        self.scroll_offset = Position::origin();

        // Clear any visual selection in the response pane
        self.visual_selection_start = None;
        self.visual_selection_end = None;

        vec![ViewEvent::ResponseContentChanged]
    }
}