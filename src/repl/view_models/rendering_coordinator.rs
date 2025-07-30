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

        // Ensure cursor remains visible after page scrolling if cursor moved
        if result.cursor_moved {
            // BUGFIX: Don't call sync_display_cursor_with_logical here as it would undo
            // the careful cursor positioning done in scroll_vertically_by_page.
            // The scroll method already sets both logical and display cursor positions correctly.
            self.ensure_cursor_visible(current_pane);
            
            // BUGFIX: Update visual selection end if in visual mode and cursor moved during page scroll
            // The page scroll functions directly update cursor position, bypassing the normal 
            // update_visual_selection_end() call that happens in cursor movement functions
            if self.mode() == crate::repl::events::EditorMode::Visual {
                let current_cursor = self.panes[current_pane].buffer.cursor();
                if self.panes[current_pane].visual_selection_start.is_some() {
                    self.panes[current_pane].visual_selection_end = Some(current_cursor);
                    tracing::debug!("Updated visual selection end to {:?} after page scroll", current_cursor);
                }
            }
        }

        // Emit scroll changed and cursor update events
        let mut events = vec![
            ViewEvent::ScrollChanged {
                pane: current_pane,
                old_offset: result.old_offset,
                new_offset: result.new_offset,
            },
            ViewEvent::CursorUpdateRequired { pane: current_pane },
            ViewEvent::PositionIndicatorUpdateRequired,
        ];
        
        // BUGFIX: Always emit pane redraw for visual mode during page scrolling
        // User requirement: "At Ctrl F/D, you must redraw the lines from the start of `v` to the jumped point"
        // The complex intersection logic was working but highlighting still wasn't updating properly,
        // so we'll use a simpler approach: always redraw the pane in visual mode during page scrolling
        if self.mode() == crate::repl::events::EditorMode::Visual {
            events.push(ViewEvent::PaneRedrawRequired { pane: current_pane });
            tracing::debug!("Visual mode page scroll: emitting pane redraw for visual selection highlighting");
        }
        
        self.emit_view_event(events);

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

        // Ensure cursor remains visible after half-page scrolling if cursor moved
        if result.cursor_moved {
            // BUGFIX: Don't call sync_display_cursor_with_logical here as it would undo
            // the careful cursor positioning done in scroll_vertically_by_half_page.
            // The scroll method already sets both logical and display cursor positions correctly.
            self.ensure_cursor_visible(current_pane);
            
            // BUGFIX: Update visual selection end if in visual mode and cursor moved during half-page scroll
            // The half-page scroll functions directly update cursor position, bypassing the normal 
            // update_visual_selection_end() call that happens in cursor movement functions
            if self.mode() == crate::repl::events::EditorMode::Visual {
                let current_cursor = self.panes[current_pane].buffer.cursor();
                if self.panes[current_pane].visual_selection_start.is_some() {
                    self.panes[current_pane].visual_selection_end = Some(current_cursor);
                    tracing::debug!("Updated visual selection end to {:?} after half-page scroll", current_cursor);
                }
            }
        }

        // Emit scroll changed and cursor update events
        let mut events = vec![
            ViewEvent::ScrollChanged {
                pane: current_pane,
                old_offset: result.old_offset,
                new_offset: result.new_offset,
            },
            ViewEvent::CursorUpdateRequired { pane: current_pane },
            ViewEvent::PositionIndicatorUpdateRequired,
        ];
        
        // BUGFIX: Always emit pane redraw for visual mode during half-page scrolling
        // User requirement: same logic as page scrolling for Ctrl+D/U
        if self.mode() == crate::repl::events::EditorMode::Visual {
            events.push(ViewEvent::PaneRedrawRequired { pane: current_pane });
            tracing::debug!("Visual mode half-page scroll: emitting pane redraw for visual selection highlighting");
        }
        
        self.emit_view_event(events);

        Ok(())
    }
}
