//! # Rendering Coordination
//!
//! Handles rendering orchestration using semantic operations.

use super::AppState;
use anyhow::Result;

impl AppState {
    /// Handle horizontal scrolling in current area
    pub fn scroll_horizontally(&mut self, direction: i32, amount: usize) -> Result<()> {
        // Delegate to PaneManager for semantic scrolling
        self.pane_manager
            .scroll_current_horizontally(direction, amount);
        Ok(())
    }
}
