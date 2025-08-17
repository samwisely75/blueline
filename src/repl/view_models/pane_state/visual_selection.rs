//! Visual selection management for PaneState
//!
//! This module contains methods for:
//! - Starting and ending visual selections
//! - Managing different visual selection modes (Character, Line, Block)
//! - Checking position inclusion in selections
//! - Updating selections during cursor movement

use crate::repl::events::{EditorMode, LogicalPosition, PaneCapabilities, ViewEvent};

use super::PaneState;

// Type alias for visual selection state (start_position, end_position)
type VisualSelection = (Option<LogicalPosition>, Option<LogicalPosition>);

// Type alias for visual selection restoration result
pub type VisualSelectionRestoreResult = Option<(EditorMode, Vec<ViewEvent>)>;

impl PaneState {
    /// Start visual selection at current cursor position
    pub fn start_visual_selection(&mut self) -> Vec<ViewEvent> {
        // Check if visual selection is allowed on this pane
        if !self.capabilities.contains(PaneCapabilities::SELECTABLE) {
            return vec![]; // Selection not allowed on this pane
        }

        let current_cursor = self.buffer.cursor();
        self.visual_selection_start = Some(current_cursor);
        self.visual_selection_end = Some(current_cursor);

        tracing::info!(
            "🎯 PaneState::start_visual_selection at position {:?}",
            current_cursor
        );

        vec![
            ViewEvent::CurrentAreaRedrawRequired,
            ViewEvent::StatusBarUpdateRequired,
            ViewEvent::ActiveCursorUpdateRequired,
        ]
    }

    /// End visual selection and clear selection state
    pub fn end_visual_selection(&mut self) -> Vec<ViewEvent> {
        // Save the last visual selection for 'gv' command before clearing
        if self.visual_selection_start.is_some() && self.visual_selection_end.is_some() {
            self.last_visual_selection_start = self.visual_selection_start;
            self.last_visual_selection_end = self.visual_selection_end;
            // Save which visual mode was active
            self.last_visual_mode = match self.editor_mode {
                EditorMode::Visual | EditorMode::VisualLine | EditorMode::VisualBlock => {
                    Some(self.editor_mode)
                }
                _ => None,
            };

            tracing::info!(
                "🎯 PaneState::end_visual_selection - saved last selection {:?} to {:?} in mode {:?}",
                self.last_visual_selection_start,
                self.last_visual_selection_end,
                self.last_visual_mode
            );
        }

        self.visual_selection_start = None;
        self.visual_selection_end = None;

        tracing::info!("🎯 PaneState::end_visual_selection - cleared selection state");

        vec![
            ViewEvent::CurrentAreaRedrawRequired,
            ViewEvent::StatusBarUpdateRequired,
            ViewEvent::ActiveCursorUpdateRequired,
        ]
    }

    /// Update visual selection end position
    pub fn update_visual_selection(&mut self, position: LogicalPosition) -> Vec<ViewEvent> {
        // Check if visual selection is allowed on this pane
        if !self.capabilities.contains(PaneCapabilities::SELECTABLE) {
            return vec![]; // Selection not allowed on this pane
        }

        if self.visual_selection_start.is_some() {
            self.visual_selection_end = Some(position);
            tracing::debug!(
                "🎯 PaneState::update_visual_selection end position to {:?}",
                position
            );
            vec![ViewEvent::CurrentAreaRedrawRequired]
        } else {
            vec![]
        }
    }

    /// Get current visual selection state
    pub fn get_visual_selection(&self) -> VisualSelection {
        (self.visual_selection_start, self.visual_selection_end)
    }

    /// Check if a position is within the current visual selection
    pub fn is_position_selected(&self, position: LogicalPosition) -> bool {
        // Early return if no selection exists
        let (Some(start), Some(end)) = (self.visual_selection_start, self.visual_selection_end)
        else {
            tracing::trace!("🎯 is_position_selected: no visual selection active");
            return false;
        };

        let editor_mode = self.editor_mode;
        tracing::trace!(
            "🎯 is_position_selected: checking position {:?} against selection {:?} to {:?} in mode {:?}",
            position,
            start,
            end,
            editor_mode
        );

        // Delegate to mode-specific selection checking
        match editor_mode {
            EditorMode::Visual => self.is_position_in_character_selection(position, start, end),
            EditorMode::VisualLine => self.is_position_in_line_selection(position, start, end),
            EditorMode::VisualBlock => self.is_position_in_block_selection(position, start, end),
            _ => {
                // Not in a visual mode, no selection
                tracing::trace!(
                    "🎯 is_position_selected: not in visual mode ({:?}), returning false",
                    editor_mode
                );
                false
            }
        }
    }

    /// Update visual selection end position during cursor movement
    /// Returns Some(ViewEvent) if visual selection was updated, None otherwise
    pub fn update_visual_selection_on_cursor_move(
        &mut self,
        new_position: LogicalPosition,
    ) -> Option<ViewEvent> {
        // Only update if we have an active selection and selection is allowed
        if self.visual_selection_start.is_some()
            && self.capabilities.contains(PaneCapabilities::SELECTABLE)
        {
            self.visual_selection_end = Some(new_position);
            tracing::debug!(
                "🎯 PaneState::update_visual_selection_on_cursor_move to {:?}",
                new_position
            );
            Some(ViewEvent::CurrentAreaRedrawRequired)
        } else {
            None
        }
    }

    // ========================================
    // Private Helper Methods
    // ========================================

    /// Check if position is in character-wise visual selection
    fn is_position_in_character_selection(
        &self,
        position: LogicalPosition,
        start: LogicalPosition,
        end: LogicalPosition,
    ) -> bool {
        let (actual_start, actual_end) = if start <= end {
            (start, end)
        } else {
            (end, start)
        };

        position >= actual_start && position <= actual_end
    }

    /// Check if position is in line-wise visual selection
    fn is_position_in_line_selection(
        &self,
        position: LogicalPosition,
        start: LogicalPosition,
        end: LogicalPosition,
    ) -> bool {
        let first_line = start.line.min(end.line);
        let last_line = start.line.max(end.line);

        position.line >= first_line && position.line <= last_line
    }

    /// Check if position is in block-wise visual selection
    fn is_position_in_block_selection(
        &self,
        position: LogicalPosition,
        start: LogicalPosition,
        end: LogicalPosition,
    ) -> bool {
        let first_line = start.line.min(end.line);
        let last_line = start.line.max(end.line);
        let first_col = start.column.min(end.column);
        let last_col = start.column.max(end.column);

        position.line >= first_line
            && position.line <= last_line
            && position.column >= first_col
            && position.column <= last_col
    }

    /// Restore the last visual selection (for 'gv' command)
    /// Returns the mode to enter and view events, or None if no last selection exists
    pub fn restore_last_visual_selection(&mut self) -> VisualSelectionRestoreResult {
        // Check if we have a saved selection
        let (Some(start), Some(end), Some(mode)) = (
            self.last_visual_selection_start,
            self.last_visual_selection_end,
            self.last_visual_mode,
        ) else {
            tracing::info!("🎯 PaneState::restore_last_visual_selection - no saved selection");
            return None;
        };

        // Restore the selection
        self.visual_selection_start = Some(start);
        self.visual_selection_end = Some(end);

        // Move cursor to the end of the selection
        self.buffer.set_cursor(end);
        self.sync_display_cursor_with_logical();

        tracing::info!(
            "🎯 PaneState::restore_last_visual_selection - restored selection {:?} to {:?} in mode {:?}",
            start,
            end,
            mode
        );

        Some((
            mode,
            vec![
                ViewEvent::CurrentAreaRedrawRequired,
                ViewEvent::StatusBarUpdateRequired,
                ViewEvent::ActiveCursorUpdateRequired,
            ],
        ))
    }
}
