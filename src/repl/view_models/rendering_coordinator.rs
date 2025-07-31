//! # Rendering Coordination
//!
//! Handles view event emission, rendering orchestration, and event collection using semantic operations.

use crate::repl::events::{ModelEvent, ViewEvent};
use crate::repl::view_models::core::ViewModel;

impl ViewModel {
    /// Emit view events (adds to pending events collection)
    /// Accepts single events, vectors, arrays, or any iterator of ViewEvent
    pub(super) fn emit_view_event<E>(&mut self, events: E)
    where
        E: IntoIterator<Item = ViewEvent>,
    {
        for event in events {
            self.pending_view_events.push(event);
            tracing::debug!("View event emitted: {:?}", self.pending_view_events.last());
        }
    }

    /// Collect and clear pending view events
    pub fn collect_pending_view_events(&mut self) -> Vec<ViewEvent> {
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
        self.emit_view_event(events);
        Ok(())
    }

    /// Handle vertical page scrolling (Ctrl+f, Ctrl+b) in current area
    pub fn scroll_vertically_by_page(
        &mut self,
        direction: i32, // 1 for down (Ctrl+f), -1 for up (Ctrl+b)
    ) -> Result<(), anyhow::Error> {
        // Delegate to PaneManager for semantic scrolling
        let events = self
            .pane_manager
            .scroll_current_vertically_by_page(direction);
        self.emit_view_event(events);
        Ok(())
    }

    /// Handle vertical half-page scrolling (Ctrl+d, Ctrl+u) in current area
    pub fn scroll_vertically_by_half_page(
        &mut self,
        direction: i32, // 1 for down (Ctrl+d), -1 for up (Ctrl+u)
    ) -> Result<(), anyhow::Error> {
        // Delegate to PaneManager for semantic scrolling
        let events = self
            .pane_manager
            .scroll_current_vertically_by_half_page(direction);
        self.emit_view_event(events);
        Ok(())
    }
}
