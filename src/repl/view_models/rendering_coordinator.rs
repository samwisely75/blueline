//! # Rendering Coordination
//!
//! Handles view event emission, rendering orchestration, and event collection.

use crate::repl::events::{Pane, ViewEvent};
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

        // BUGFIX: Update logical cursor position to stay visible after horizontal scroll
        // Without this fix, cursor position indicator doesn't update during horizontal scrolling
        // (e.g., stays at "RESPONSE 56:1" while visually scrolling horizontally)
        let current_cursor = self.get_cursor_position();
        let display_cache = self.get_display_cache(current_pane);

        // Convert current logical position to display coordinates
        if let Some(display_pos) =
            display_cache.logical_to_display_position(current_cursor.line, current_cursor.column)
        {
            // Check if cursor is still visible after horizontal scroll
            let content_width = self.get_content_width();

            // If cursor is off-screen, move it to the first/last visible column
            let new_cursor_column = if display_pos.1 < new_horizontal_offset {
                // Cursor is off-screen to the left, move to first visible column
                new_horizontal_offset
            } else if display_pos.1 >= new_horizontal_offset + content_width {
                // Cursor is off-screen to the right, move to last visible column
                new_horizontal_offset + content_width - 1
            } else {
                // Cursor is still visible, keep current position
                display_pos.1
            };

            // Convert back to logical position and update cursor
            if let Some(logical_pos) =
                display_cache.display_to_logical_position(display_pos.0, new_cursor_column)
            {
                use crate::repl::events::LogicalPosition;
                let new_cursor_position = LogicalPosition::new(logical_pos.0, logical_pos.1);

                // Update logical cursor in appropriate buffer
                match current_pane {
                    Pane::Request => {
                        let clamped_position = self
                            .request_buffer
                            .content()
                            .clamp_position(new_cursor_position);
                        self.request_buffer.set_cursor(clamped_position);
                    }
                    Pane::Response => {
                        let clamped_position = self
                            .response_buffer
                            .content()
                            .clamp_position(new_cursor_position);
                        self.response_buffer.set_cursor(clamped_position);
                    }
                }

                // Sync display cursor with the new logical position
                self.sync_display_cursor_with_logical(current_pane)?;
            }
        }

        // Emit scroll changed event
        self.emit_view_event(ViewEvent::ScrollChanged {
            pane: current_pane,
            old_offset,
            new_offset: new_horizontal_offset,
        });

        // Emit cursor update events since cursor position may have changed
        self.emit_view_event(ViewEvent::CursorUpdateRequired { pane: current_pane });
        self.emit_view_event(ViewEvent::PositionIndicatorUpdateRequired);

        tracing::debug!(
            "Horizontal scroll: pane={:?}, direction={}, amount={}, new_offset={}",
            current_pane,
            direction,
            amount,
            new_horizontal_offset
        );

        Ok(())
    }

    /// Handle vertical page scrolling (Ctrl+f, Ctrl+b)
    pub fn scroll_vertically_by_page(
        &mut self,
        direction: i32, // 1 for down (Ctrl+f), -1 for up (Ctrl+b)
    ) -> Result<(), anyhow::Error> {
        let current_pane = self.editor.current_pane();
        let (vertical_offset, horizontal_offset) = self.get_scroll_offset(current_pane);

        // Calculate page size based on pane height
        let page_size = match current_pane {
            Pane::Request => self.request_pane_height() as usize,
            Pane::Response => self.response_pane_height() as usize,
        };

        // Vim typically scrolls by (page_size - 2) to maintain some context
        let scroll_amount = page_size.saturating_sub(2).max(1);

        // BUGFIX: Prevent scrolling beyond actual content bounds to avoid cursor/buffer desync
        // When scrolling past content in single-line buffers, display_to_logical_position fails
        // and leaves cursor in inconsistent state, causing buffer content to appear erased
        let display_cache = self.get_display_cache(current_pane);
        let max_scroll_offset = display_cache
            .display_line_count()
            .saturating_sub(page_size)
            .max(0);

        let new_vertical_offset = if direction > 0 {
            std::cmp::min(vertical_offset + scroll_amount, max_scroll_offset)
        } else {
            vertical_offset.saturating_sub(scroll_amount)
        };

        // If scroll offset wouldn't change, don't do anything to avoid unnecessary cursor updates
        if new_vertical_offset == vertical_offset {
            return Ok(());
        }

        let old_offset = vertical_offset;
        self.set_scroll_offset(current_pane, (new_vertical_offset, horizontal_offset));

        // BUGFIX: Move logical cursor to match the new scroll position to prevent navigation issues
        // Without this fix, after page scrolling the logical cursor remains at the old position,
        // causing k/j navigation to behave incorrectly (cursor jumps back to old position)
        // This matches Vim's Ctrl+f behavior where the cursor moves to the top visible line
        let display_cache = self.get_display_cache(current_pane);
        // BUGFIX: Account for horizontal scroll offset when positioning cursor after page scroll
        // Without this fix, cursor appears off-screen when horizontally scrolled (e.g., at col 300)
        let cursor_column = horizontal_offset; // Position at first visible column, not always 0
        if let Some(logical_pos) =
            display_cache.display_to_logical_position(new_vertical_offset, cursor_column)
        {
            use crate::repl::events::LogicalPosition;
            let cursor_position = LogicalPosition::new(logical_pos.0, logical_pos.1);

            // Update logical cursor in appropriate buffer to maintain cursor-scroll synchronization
            match current_pane {
                Pane::Request => {
                    let clamped_position = self
                        .request_buffer
                        .content()
                        .clamp_position(cursor_position);
                    self.request_buffer.set_cursor(clamped_position);
                }
                Pane::Response => {
                    let clamped_position = self
                        .response_buffer
                        .content()
                        .clamp_position(cursor_position);
                    self.response_buffer.set_cursor(clamped_position);
                }
            }

            // Sync display cursor with the new logical position to ensure consistency
            self.sync_display_cursor_with_logical(current_pane)?;
        }

        // Emit scroll changed event
        self.emit_view_event(ViewEvent::ScrollChanged {
            pane: current_pane,
            old_offset,
            new_offset: new_vertical_offset,
        });

        // Emit cursor update events
        self.emit_view_event(ViewEvent::CursorUpdateRequired { pane: current_pane });
        self.emit_view_event(ViewEvent::PositionIndicatorUpdateRequired);

        Ok(())
    }

    /// Handle vertical half-page scrolling (Ctrl+d, Ctrl+u)
    pub fn scroll_vertically_by_half_page(
        &mut self,
        direction: i32, // 1 for down (Ctrl+d), -1 for up (Ctrl+u)
    ) -> Result<(), anyhow::Error> {
        let current_pane = self.editor.current_pane();
        let (vertical_offset, horizontal_offset) = self.get_scroll_offset(current_pane);

        // Calculate half page size based on pane height
        let page_size = match current_pane {
            Pane::Request => self.request_pane_height() as usize,
            Pane::Response => self.response_pane_height() as usize,
        };

        // Half page scrolling is typically page_size / 2
        let scroll_amount = (page_size / 2).max(1);

        // BUGFIX: Prevent half-page scrolling beyond actual content bounds like full page scrolling
        // Endless scrolling in small buffers causes cursor/line number corruption and poor UX
        let max_scroll_offset = {
            let display_cache = self.get_display_cache(current_pane);
            display_cache
                .display_line_count()
                .saturating_sub(page_size)
                .max(0)
        };

        let new_vertical_offset = if direction > 0 {
            std::cmp::min(vertical_offset + scroll_amount, max_scroll_offset)
        } else {
            vertical_offset.saturating_sub(scroll_amount)
        };

        // If scroll offset wouldn't change, don't do anything to avoid unnecessary cursor updates
        if new_vertical_offset == vertical_offset {
            return Ok(());
        }

        let old_offset = vertical_offset;
        self.set_scroll_offset(current_pane, (new_vertical_offset, horizontal_offset));

        // Move logical cursor to match the new scroll position like full page scrolling
        // This ensures consistency with page scrolling behavior and prevents navigation issues
        let cursor_column = horizontal_offset; // Position at first visible column
        let display_cache = self.get_display_cache(current_pane);
        if let Some(logical_pos) =
            display_cache.display_to_logical_position(new_vertical_offset, cursor_column)
        {
            use crate::repl::events::LogicalPosition;
            let cursor_position = LogicalPosition::new(logical_pos.0, logical_pos.1);

            // Update logical cursor in appropriate buffer
            match current_pane {
                Pane::Request => {
                    let clamped_position = self
                        .request_buffer
                        .content()
                        .clamp_position(cursor_position);
                    self.request_buffer.set_cursor(clamped_position);
                }
                Pane::Response => {
                    let clamped_position = self
                        .response_buffer
                        .content()
                        .clamp_position(cursor_position);
                    self.response_buffer.set_cursor(clamped_position);
                }
            }

            // Sync display cursor with the new logical position
            self.sync_display_cursor_with_logical(current_pane)?;
        }

        // Emit scroll changed event
        self.emit_view_event(ViewEvent::ScrollChanged {
            pane: current_pane,
            old_offset,
            new_offset: new_vertical_offset,
        });

        // Emit cursor update events
        self.emit_view_event(ViewEvent::CursorUpdateRequired { pane: current_pane });
        self.emit_view_event(ViewEvent::PositionIndicatorUpdateRequired);

        Ok(())
    }
}
