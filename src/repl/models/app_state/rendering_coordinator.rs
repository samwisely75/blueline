//! # Rendering Coordination
//!
//! Handles view event emission, rendering orchestration, and event collection using semantic operations.

use super::AppState;
use crate::repl::models::events::ModelEvent;
use crate::repl::view_models::PostCommandAction;

impl AppState {
    /// Emit view events (adds to pending events collection)
    /// Accepts single events, vectors, arrays, or any iterator of PostCommandAction
    pub fn emit_view_event<E>(&mut self, events: E) -> Result<(), anyhow::Error>
    where
        E: IntoIterator<Item = PostCommandAction>,
    {
        let event_vec: Vec<PostCommandAction> = events.into_iter().collect();
        if !event_vec.is_empty() {
            for event in event_vec {
                self.pending_view_events.push(event);
                tracing::debug!("View event emitted: {:?}", self.pending_view_events.last());
            }
        }
        Ok(())
    }

    /// Collect and clear pending view events
    pub fn collect_pending_view_events(&mut self) -> Vec<PostCommandAction> {
        let events = self.pending_view_events.clone();
        self.pending_view_events.clear();
        events
    }

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
        let events = self
            .pane_manager
            .scroll_current_horizontally(direction, amount);
        self.emit_view_event(events)
    }
}
