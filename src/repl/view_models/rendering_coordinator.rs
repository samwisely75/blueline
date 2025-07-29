//! # Rendering Coordination
//!
//! Handles view event emission, rendering orchestration, and event collection.

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

    /// Handle horizontal scrolling
    pub fn scroll_horizontally(
        &mut self,
        direction: i32,
        amount: usize,
    ) -> Result<(), anyhow::Error> {
        let current_pane = self.current_pane;

        // Delegate to PaneState for scrolling logic
        let result = self.panes[current_pane].scroll_horizontally(direction, amount);

        // Sync display cursor with the new logical position if cursor moved
        if result.cursor_moved {
            self.sync_display_cursor_with_logical(current_pane)?;
        }

        // Emit scroll changed event
        self.emit_view_event([ViewEvent::ScrollChanged {
            pane: current_pane,
            old_offset: result.old_offset,
            new_offset: result.new_offset,
        }]);

        // Emit cursor update events since cursor position may have changed
        self.emit_view_event([
            ViewEvent::CursorUpdateRequired { pane: current_pane },
            ViewEvent::PositionIndicatorUpdateRequired,
        ]);

        tracing::debug!(
            "Horizontal scroll: pane={:?}, direction={}, amount={}, new_offset={}",
            current_pane,
            direction,
            amount,
            result.new_offset
        );

        Ok(())
    }

    /// Handle vertical page scrolling (Ctrl+f, Ctrl+b)
    pub fn scroll_vertically_by_page(
        &mut self,
        direction: i32, // 1 for down (Ctrl+f), -1 for up (Ctrl+b)
    ) -> Result<(), anyhow::Error> {
        let current_pane = self.current_pane;

        // Delegate to PaneState for scrolling logic
        let result = self.panes[current_pane].scroll_vertically_by_page(direction);

        // If scroll offset wouldn't change, don't do anything
        if result.old_offset == result.new_offset {
            return Ok(());
        }

        // Sync display cursor with the new logical position if cursor moved
        if result.cursor_moved {
            self.sync_display_cursor_with_logical(current_pane)?;
        }

        // Emit scroll changed and cursor update events
        self.emit_view_event([
            ViewEvent::ScrollChanged {
                pane: current_pane,
                old_offset: result.old_offset,
                new_offset: result.new_offset,
            },
            ViewEvent::CursorUpdateRequired { pane: current_pane },
            ViewEvent::PositionIndicatorUpdateRequired,
        ]);

        Ok(())
    }

    /// Handle vertical half-page scrolling (Ctrl+d, Ctrl+u)
    pub fn scroll_vertically_by_half_page(
        &mut self,
        direction: i32, // 1 for down (Ctrl+d), -1 for up (Ctrl+u)
    ) -> Result<(), anyhow::Error> {
        let current_pane = self.current_pane;

        // Delegate to PaneState for scrolling logic
        let result = self.panes[current_pane].scroll_vertically_by_half_page(direction);

        // If scroll offset wouldn't change, don't do anything
        if result.old_offset == result.new_offset {
            return Ok(());
        }

        // Sync display cursor with the new logical position if cursor moved
        if result.cursor_moved {
            self.sync_display_cursor_with_logical(current_pane)?;
        }

        // Emit scroll changed and cursor update events
        self.emit_view_event([
            ViewEvent::ScrollChanged {
                pane: current_pane,
                old_offset: result.old_offset,
                new_offset: result.new_offset,
            },
            ViewEvent::CursorUpdateRequired { pane: current_pane },
            ViewEvent::PositionIndicatorUpdateRequired,
        ]);

        Ok(())
    }
}
