//! # Mode Management
//!
//! Handles editor mode transitions, visual mode selection state, and mode-related operations.

use crate::repl::events::{EditorMode, LogicalPosition, Pane, ViewEvent};
use crate::repl::view_models::core::ViewModel;
use anyhow::Result;

/// Type alias for visual selection state to reduce complexity
type VisualSelectionState = (
    Option<LogicalPosition>,
    Option<LogicalPosition>,
    Option<Pane>,
);

impl ViewModel {
    /// Get current editor mode
    pub fn get_mode(&self) -> EditorMode {
        self.mode()
    }

    /// Change editor mode
    pub fn change_mode(&mut self, mode: EditorMode) -> Result<()> {
        let old_mode = self.mode();
        tracing::debug!("Changing mode from {:?} to {:?}", old_mode, mode);

        let _event = self.set_mode(mode);
        // TODO: self.emit_model_event(event);

        // Update status line mode
        self.status_line.set_editor_mode(mode);

        // Clear command buffer when exiting Command mode (e.g., when pressing Escape)
        if old_mode == EditorMode::Command && mode != EditorMode::Command {
            self.status_line.clear_command_buffer();
            tracing::debug!("Cleared command buffer when exiting Command mode");
        }

        // Update the mode for the current pane and handle cursor adjustments
        let mut events = self.pane_manager.set_current_pane_mode(mode);

        // Handle visual mode selection state using PaneManager
        if mode == EditorMode::Visual && old_mode != EditorMode::Visual {
            // Entering visual mode
            events.extend(self.pane_manager.start_visual_selection());
        } else if old_mode == EditorMode::Visual && mode != EditorMode::Visual {
            // Exiting visual mode
            events.extend(self.pane_manager.end_visual_selection());
        }

        // Add standard mode change events
        events.extend([
            ViewEvent::StatusBarUpdateRequired,
            ViewEvent::ActiveCursorUpdateRequired,
        ]);

        let _ = self.emit_view_event(events);

        tracing::info!(
            "Successfully changed mode from {:?} to {:?}",
            old_mode,
            mode
        );
        Ok(())
    }

    /// Get visual selection state
    pub fn get_visual_selection(&self) -> VisualSelectionState {
        self.pane_manager.get_visual_selection()
    }

    /// Check if a position is within visual selection
    pub fn is_position_selected(&self, position: LogicalPosition, pane: Pane) -> bool {
        self.pane_manager.is_position_selected(position, pane)
    }
}
