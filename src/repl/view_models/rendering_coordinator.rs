//! # Rendering Coordination
//!
//! Handles view event emission, rendering orchestration, and event collection.

use crate::repl::events::ViewEvent;
use crate::repl::view_models::core::ViewModel;

impl ViewModel {
    /// Emit a view event (adds to pending events collection)
    pub(super) fn emit_view_event(&mut self, event: ViewEvent) {
        self.pending_view_events.push(event);
        tracing::debug!("View event emitted: {:?}", self.pending_view_events.last());
    }

    /// Collect and clear pending view events
    pub fn collect_pending_view_events(&mut self) -> Vec<ViewEvent> {
        let events = self.pending_view_events.clone();
        self.pending_view_events.clear();
        events
    }

    /// Handle horizontal scrolling
    pub fn scroll_horizontally(
        &mut self,
        direction: i32,
        amount: usize,
    ) -> Result<(), anyhow::Error> {
        let current_pane = self.editor.current_pane();
        let (vertical_offset, horizontal_offset) = self.get_scroll_offset(current_pane);

        let new_horizontal_offset = if direction > 0 {
            horizontal_offset + amount
        } else {
            horizontal_offset.saturating_sub(amount)
        };

        let old_offset = horizontal_offset;
        self.set_scroll_offset(current_pane, (vertical_offset, new_horizontal_offset));

        // Emit scroll changed event
        self.emit_view_event(ViewEvent::ScrollChanged {
            pane: current_pane,
            old_offset,
            new_offset: new_horizontal_offset,
        });

        tracing::debug!(
            "Horizontal scroll: pane={:?}, direction={}, amount={}, new_offset={}",
            current_pane,
            direction,
            amount,
            new_horizontal_offset
        );

        Ok(())
    }
}
