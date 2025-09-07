//! # Rendering Coordination
//!
//! Handles rendering orchestration and event collection using semantic operations.

use super::AppState;
use crate::repl::models::events::ModelEvent;

impl AppState {
    /// Collect and clear pending model events
    pub fn collect_pending_model_events(&mut self) -> Vec<ModelEvent> {
        let events = self.pending_model_events.clone();
        self.pending_model_events.clear();
        events
    }

    /// Handle horizontal scrolling in current area
    pub fn scroll_horizontally(
        &mut self,
        direction: i32,
        amount: usize,
    ) -> Result<(), anyhow::Error> {
        // Delegate to PaneManager for semantic scrolling
        self.pane_manager
            .scroll_current_horizontally(direction, amount);
        Ok(())
    }
}
