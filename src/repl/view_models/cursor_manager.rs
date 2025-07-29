//! # Cursor Management
//!
//! Handles all cursor movement and positioning logic including display cursor synchronization,
//! scrolling, and coordinate transformations between logical and display positions.

use crate::repl::events::{EditorMode, LogicalPosition, Pane, ViewEvent};
use crate::repl::view_models::core::ViewModel;
use anyhow::Result;

impl ViewModel {
    /// Get current logical cursor position for the active pane
    pub fn get_cursor_position(&self) -> LogicalPosition {
        let current_pane = self.current_pane;
        self.panes[current_pane].buffer.cursor()
    }

    /// Move cursor left in current pane (display coordinate based)
    pub fn move_cursor_left(&mut self) -> Result<()> {
        let current_pane = self.current_pane;

        // Sync display cursor with current logical cursor position
        self.sync_display_cursor_with_logical(current_pane)?;

        let current_display_pos = self.get_display_cursor(current_pane);
        let display_cache = self.get_display_cache(current_pane);

        tracing::debug!(
            "move_cursor_left: pane={:?}, current_pos={:?}",
            current_pane,
            current_display_pos
        );

        let mut moved = false;

        // Check if we can move left within current display line
        if current_display_pos.1 > 0 {
            let new_display_pos = (current_display_pos.0, current_display_pos.1 - 1);
            tracing::debug!(
                "move_cursor_left: moving within line to {:?}",
                new_display_pos
            );
            self.set_display_cursor(current_pane, new_display_pos)?;
            self.ensure_cursor_visible(current_pane);
            moved = true;
        } else if current_display_pos.0 > 0 {
            // Move to end of previous display line
            let prev_display_line = current_display_pos.0 - 1;
            if let Some(prev_line) = display_cache.get_display_line(prev_display_line) {
                let new_col = prev_line.content.chars().count();
                let new_display_pos = (prev_display_line, new_col);
                tracing::debug!(
                    "move_cursor_left: moving to end of previous line {:?}",
                    new_display_pos
                );
                self.set_display_cursor(current_pane, new_display_pos)?;
                self.ensure_cursor_visible(current_pane);
                moved = true;
            }
        } else {
            tracing::debug!("move_cursor_left: already at beginning, no movement");
        }

        // Only emit events if we actually moved
        if moved {
            self.emit_view_event([
                ViewEvent::CursorUpdateRequired { pane: current_pane },
                ViewEvent::PositionIndicatorUpdateRequired,
            ]);
        }

        Ok(())
    }

    /// Move cursor right in current pane (display coordinate based)
    pub fn move_cursor_right(&mut self) -> Result<()> {
        let current_pane = self.current_pane;

        // Sync display cursor with current logical cursor position
        self.sync_display_cursor_with_logical(current_pane)?;

        let current_display_pos = self.get_display_cursor(current_pane);
        let display_cache = self.get_display_cache(current_pane);

        tracing::debug!(
            "move_cursor_right: pane={:?}, current_pos={:?}",
            current_pane,
            current_display_pos
        );

        let mut moved = false;

        // Get current display line info
        if let Some(current_line) = display_cache.get_display_line(current_display_pos.0) {
            let line_length = current_line.content.chars().count();

            // Check if we can move right within current display line
            if current_display_pos.1 < line_length {
                let new_display_pos = (current_display_pos.0, current_display_pos.1 + 1);
                tracing::debug!(
                    "move_cursor_right: moving within line to {:?}",
                    new_display_pos
                );
                self.set_display_cursor(current_pane, new_display_pos)?;
                self.ensure_cursor_visible(current_pane);
                moved = true;
            } else if current_display_pos.0 + 1 < display_cache.display_line_count() {
                // Move to beginning of next display line
                let new_display_pos = (current_display_pos.0 + 1, 0);
                tracing::debug!(
                    "move_cursor_right: moving to next line {:?}",
                    new_display_pos
                );
                self.set_display_cursor(current_pane, new_display_pos)?;
                self.ensure_cursor_visible(current_pane);
                moved = true;
            } else {
                tracing::debug!("move_cursor_right: already at end, no movement");
            }
        } else {
            tracing::debug!("move_cursor_right: invalid display line, no movement");
        }

        // Only emit events if we actually moved
        if moved {
            self.emit_view_event([
                ViewEvent::CursorUpdateRequired { pane: current_pane },
                ViewEvent::PositionIndicatorUpdateRequired,
            ]);
        }

        Ok(())
    }

    /// Move cursor up in current pane (display line based)
    pub fn move_cursor_up(&mut self) -> Result<()> {
        let current_pane = self.current_pane;
        let current_display_pos = self.get_display_cursor(current_pane);
        let display_cache = self.get_display_cache(current_pane);

        tracing::debug!(
            "move_cursor_up: pane={:?}, current_pos={:?}",
            current_pane,
            current_display_pos
        );

        if let Some(new_pos) = display_cache.move_up(current_display_pos.0, current_display_pos.1) {
            self.set_display_cursor(current_pane, new_pos)?;
            self.ensure_cursor_visible(current_pane);
            self.emit_view_event([
                ViewEvent::CursorUpdateRequired { pane: current_pane },
                ViewEvent::PositionIndicatorUpdateRequired,
            ]);
        } else {
            tracing::debug!("move_cursor_up: already at top, no movement");
        }

        Ok(())
    }

    /// Move cursor down in current pane (display line based)
    pub fn move_cursor_down(&mut self) -> Result<()> {
        let current_pane = self.current_pane;
        let current_display_pos = self.get_display_cursor(current_pane);
        let display_cache = self.get_display_cache(current_pane);

        tracing::debug!(
            "move_cursor_down: pane={:?}, current_pos={:?}",
            current_pane,
            current_display_pos
        );

        if let Some(new_pos) = display_cache.move_down(current_display_pos.0, current_display_pos.1)
        {
            self.set_display_cursor(current_pane, new_pos)?;
            self.ensure_cursor_visible(current_pane);
            self.emit_view_event([
                ViewEvent::CursorUpdateRequired { pane: current_pane },
                ViewEvent::PositionIndicatorUpdateRequired,
            ]);
        } else {
            tracing::debug!("move_cursor_down: already at bottom, no movement");
        }

        Ok(())
    }

    /// Move cursor to end of current line
    pub fn move_cursor_to_end_of_line(&mut self) -> Result<()> {
        let current_pane = self.current_pane;
        let current_logical_pos = self.get_cursor_position();

        // Get the text content for the current pane
        let text = match current_pane {
            Pane::Request => self.get_request_text(),
            Pane::Response => self.get_response_text(),
        };

        let lines: Vec<_> = text.lines().collect();

        if current_logical_pos.line < lines.len() {
            let line_content = lines[current_logical_pos.line];
            let line_length = line_content.chars().count();
            let new_position = LogicalPosition::new(current_logical_pos.line, line_length);
            self.set_cursor_position(new_position)?;
        }

        Ok(())
    }

    /// Move cursor to start of current line
    pub fn move_cursor_to_start_of_line(&mut self) -> Result<()> {
        let current_logical_pos = self.get_cursor_position();
        let new_position = LogicalPosition::new(current_logical_pos.line, 0);
        self.set_cursor_position(new_position)?;

        Ok(())
    }

    /// Move cursor to start of document (gg command)
    pub fn move_cursor_to_document_start(&mut self) -> Result<()> {
        // Set logical cursor to (0, 0) - first line, first column
        let start_position = LogicalPosition::new(0, 0);
        self.set_cursor_position(start_position)?;

        Ok(())
    }

    /// Move cursor to end of document (G command)
    pub fn move_cursor_to_document_end(&mut self) -> Result<()> {
        let current_pane = self.current_pane;

        // BUGFIX: Use the exact same approach as the test framework to ensure consistency
        // Without this matching approach, G command integration tests fail due to line counting mismatch
        // (expected cursor at line 7 but was at line 8) because test framework counts lines differently
        let text = match current_pane {
            Pane::Request => self.get_request_text(),
            Pane::Response => self.get_response_text(),
        };

        let lines: Vec<_> = text.lines().collect();

        if lines.is_empty() {
            // If no content, position at (0, 0)
            let end_position = LogicalPosition::new(0, 0);
            self.set_cursor_position(end_position)?;
        } else {
            // Position at the beginning of the last line (column 0), like Vim's G command
            // Use the same calculation as the test: lines.len() - 1
            let last_line_index = lines.len() - 1;
            let end_position = LogicalPosition::new(last_line_index, 0);
            self.set_cursor_position(end_position)?;
        }

        Ok(())
    }

    /// Set cursor to specific logical position
    pub fn set_cursor_position(&mut self, position: LogicalPosition) -> Result<()> {
        let current_pane = self.current_pane;

        // Update logical cursor in appropriate buffer
        let clamped_position = self.panes[current_pane]
            .buffer
            .content()
            .clamp_position(position);
        self.panes[current_pane].buffer.set_cursor(clamped_position);

        // Update visual selection if in visual mode
        self.update_visual_selection_end();

        // Sync display cursor
        self.sync_display_cursor_with_logical(current_pane)?;
        self.ensure_cursor_visible(current_pane);
        self.emit_view_event([
            ViewEvent::CursorUpdateRequired { pane: current_pane },
            ViewEvent::PositionIndicatorUpdateRequired,
        ]);

        Ok(())
    }

    /// Update visual selection end position if in visual mode
    fn update_visual_selection_end(&mut self) {
        if self.mode() == EditorMode::Visual {
            let current_cursor = self.get_cursor_position();
            let current_pane = self.current_pane;

            // Update the selection end for the current pane
            if self.panes[current_pane].visual_selection_start.is_some() {
                self.panes[current_pane].visual_selection_end = Some(current_cursor);
                tracing::debug!("Updated visual selection end to {:?}", current_cursor);

                // BUGFIX: Emit pane redraw event to trigger visual selection rendering
                // Without this, visual selection highlighting won't appear because
                // only cursor events are emitted, not text re-rendering events
                self.emit_view_event([ViewEvent::PaneRedrawRequired { pane: current_pane }]);
                tracing::debug!("Emitted pane redraw event for visual selection update");
            }
        }
    }

    /// Synchronize display cursors for both panes
    pub(super) fn sync_display_cursors(&mut self) {
        for pane in [Pane::Request, Pane::Response] {
            let logical = self.panes[pane].buffer.cursor();
            if let Some(display_pos) = self.panes[pane]
                .display_cache
                .logical_to_display_position(logical.line, logical.column)
            {
                self.panes[pane].display_cursor = display_pos;
            }
        }
    }

    /// Get display cursor position for a pane
    pub(super) fn get_display_cursor(&self, pane: Pane) -> (usize, usize) {
        self.panes[pane].display_cursor
    }

    /// Set display cursor position for a pane
    pub(super) fn set_display_cursor(
        &mut self,
        pane: Pane,
        position: (usize, usize),
    ) -> Result<()> {
        self.panes[pane].display_cursor = position;

        // Convert back to logical position and update buffer
        if let Some(logical_pos) = self.panes[pane]
            .display_cache
            .display_to_logical_position(position.0, position.1)
        {
            let logical_position = LogicalPosition::new(logical_pos.0, logical_pos.1);
            self.panes[pane].buffer.set_cursor(logical_position);
        }

        // Update visual selection if in visual mode
        self.update_visual_selection_end();

        Ok(())
    }

    /// Synchronize display cursor with logical cursor position
    pub(super) fn sync_display_cursor_with_logical(&mut self, pane: Pane) -> Result<()> {
        let logical_pos = self.panes[pane].buffer.cursor();

        if let Some(display_pos) = self.panes[pane]
            .display_cache
            .logical_to_display_position(logical_pos.line, logical_pos.column)
        {
            self.panes[pane].display_cursor = display_pos;
        }

        Ok(())
    }

    /// Ensure cursor is visible within the viewport (handles scrolling)
    pub(super) fn ensure_cursor_visible(&mut self, pane: Pane) {
        let display_pos = self.get_display_cursor(pane);
        let (vertical_offset, horizontal_offset) = self.get_scroll_offset(pane);
        let pane_height = self.get_pane_display_height(pane);
        let content_width = self.get_content_width();

        let mut new_vertical_offset = vertical_offset;
        let mut new_horizontal_offset = horizontal_offset;

        // Vertical scrolling to keep cursor within visible area
        if display_pos.0 < vertical_offset {
            new_vertical_offset = display_pos.0;
        } else if display_pos.0 >= vertical_offset + pane_height && pane_height > 0 {
            // BUGFIX: Add pane_height > 0 check to prevent integer underflow in tests
            // Without this check, pane_height - 1 would underflow when pane_height is 0,
            // causing panics in test environments where terminal height is uninitialized
            new_vertical_offset = display_pos.0.saturating_sub(pane_height.saturating_sub(1));
        }

        // Horizontal scrolling
        if display_pos.1 < horizontal_offset {
            new_horizontal_offset = display_pos.1;
        } else if display_pos.1 >= horizontal_offset + content_width && content_width > 0 {
            // BUGFIX: Add content_width > 0 check to prevent integer underflow panic
            // This prevents crashes when content width is zero
            new_horizontal_offset = display_pos
                .1
                .saturating_sub(content_width.saturating_sub(1));
        }

        // Update scroll offset if changed
        if new_vertical_offset != vertical_offset || new_horizontal_offset != horizontal_offset {
            let old_offset = (vertical_offset, horizontal_offset);
            self.set_scroll_offset(pane, (new_vertical_offset, new_horizontal_offset));

            // Emit scroll changed event
            self.emit_view_event([ViewEvent::ScrollChanged {
                pane,
                old_offset: old_offset.0,
                new_offset: new_vertical_offset,
            }]);
        }
    }

    /// Get scroll offset for a pane
    pub(super) fn get_scroll_offset(&self, pane: Pane) -> (usize, usize) {
        self.panes[pane].scroll_offset
    }

    /// Set scroll offset for a pane
    pub(super) fn set_scroll_offset(&mut self, pane: Pane, offset: (usize, usize)) {
        self.panes[pane].scroll_offset = offset;
    }

    /// Move cursor to the beginning of the next word
    pub fn move_cursor_to_next_word(&mut self) -> Result<()> {
        let current_pane = self.current_pane;
        let current_display_pos = self.get_display_cursor(current_pane);
        let display_cache = self.get_display_cache(current_pane);

        tracing::debug!(
            "move_cursor_to_next_word: pane={:?}, current_pos={:?}",
            current_pane,
            current_display_pos
        );

        let mut current_line = current_display_pos.0;
        let mut current_col = current_display_pos.1;
        let mut moved = false;

        // Loop through display lines to find next word
        while current_line < display_cache.display_line_count() {
            if let Some(line_info) = display_cache.get_display_line(current_line) {
                let chars: Vec<char> = line_info.content.chars().collect();

                // If we're not at the end of this line, look for next word on current line
                if current_col < chars.len() {
                    let mut pos = current_col;

                    // If we're on a word character, skip to end of current word
                    if pos < chars.len() && (chars[pos].is_alphanumeric() || chars[pos] == '_') {
                        while pos < chars.len()
                            && (chars[pos].is_alphanumeric() || chars[pos] == '_')
                        {
                            pos += 1;
                        }
                    }
                    // If we're on whitespace or punctuation, skip it
                    else if pos < chars.len()
                        && !chars[pos].is_alphanumeric()
                        && chars[pos] != '_'
                    {
                        while pos < chars.len()
                            && !chars[pos].is_alphanumeric()
                            && chars[pos] != '_'
                        {
                            pos += 1;
                        }
                    }

                    // Skip any whitespace after word/punctuation
                    while pos < chars.len() && chars[pos].is_whitespace() {
                        pos += 1;
                    }

                    // If we found a word start on this line
                    if pos < chars.len() {
                        let new_pos = (current_line, pos);
                        tracing::debug!("move_cursor_to_next_word: found word at {:?}", new_pos);
                        self.set_display_cursor(current_pane, new_pos)?;
                        self.ensure_cursor_visible(current_pane);
                        moved = true;
                        break;
                    }
                }

                // Move to next line and start at beginning
                current_line += 1;
                current_col = 0;

                // If we moved to next line, look for first word on that line
                if current_line < display_cache.display_line_count() {
                    if let Some(next_line_info) = display_cache.get_display_line(current_line) {
                        let next_chars: Vec<char> = next_line_info.content.chars().collect();
                        let mut pos = 0;

                        // Skip leading whitespace
                        while pos < next_chars.len() && next_chars[pos].is_whitespace() {
                            pos += 1;
                        }

                        // If there's a word on this line
                        if pos < next_chars.len() {
                            let new_pos = (current_line, pos);
                            tracing::debug!(
                                "move_cursor_to_next_word: found word on next line at {:?}",
                                new_pos
                            );
                            self.set_display_cursor(current_pane, new_pos)?;
                            self.ensure_cursor_visible(current_pane);
                            moved = true;
                            break;
                        }
                    }
                }
            } else {
                break;
            }
        }

        if !moved {
            tracing::debug!(
                "move_cursor_to_next_word: no next word found, staying at current position"
            );
        }

        // Emit events if we moved
        if moved {
            self.emit_view_event([
                ViewEvent::CursorUpdateRequired { pane: current_pane },
                ViewEvent::PositionIndicatorUpdateRequired,
            ]);
        }

        Ok(())
    }

    /// Move cursor to the beginning of the previous word
    pub fn move_cursor_to_previous_word(&mut self) -> Result<()> {
        let current_pane = self.current_pane;
        let current_display_pos = self.get_display_cursor(current_pane);
        let display_cache = self.get_display_cache(current_pane);

        tracing::debug!(
            "move_cursor_to_previous_word: pane={:?}, current_pos={:?}",
            current_pane,
            current_display_pos
        );

        let mut current_line = current_display_pos.0;
        let mut current_col = current_display_pos.1;
        let mut moved = false;

        // Loop through display lines backwards to find previous word
        // Complex control flow with multiple break conditions requires loop/if structure
        #[allow(clippy::while_let_loop)]
        loop {
            if let Some(line_info) = display_cache.get_display_line(current_line) {
                let chars: Vec<char> = line_info.content.chars().collect();

                // If we're at the beginning of this line, move to previous line
                if current_col == 0 {
                    if current_line > 0 {
                        current_line -= 1;
                        if let Some(prev_line_info) = display_cache.get_display_line(current_line) {
                            current_col = prev_line_info.content.chars().count();
                            continue;
                        }
                    }
                    break; // Already at beginning of buffer
                }

                let mut pos = current_col.saturating_sub(1);

                // Skip trailing whitespace if we're starting on whitespace
                if pos < chars.len() && chars[pos].is_whitespace() {
                    while pos > 0 && chars[pos].is_whitespace() {
                        pos -= 1;
                    }
                    if pos == 0 && chars[pos].is_whitespace() {
                        let new_pos = (current_line, 0);
                        tracing::debug!(
                            "move_cursor_to_previous_word: found beginning at {:?}",
                            new_pos
                        );
                        self.set_display_cursor(current_pane, new_pos)?;
                        self.ensure_cursor_visible(current_pane);
                        moved = true;
                        break;
                    }
                }

                // Now we're at the end of a word, skip to beginning
                if pos < chars.len() && (chars[pos].is_alphanumeric() || chars[pos] == '_') {
                    while pos > 0 && (chars[pos].is_alphanumeric() || chars[pos] == '_') {
                        pos -= 1;
                    }
                    // If we stopped because of a non-word character, move forward one
                    if pos < chars.len() && !chars[pos].is_alphanumeric() && chars[pos] != '_' {
                        pos += 1;
                    }
                } else if pos < chars.len()
                    && !chars[pos].is_alphanumeric()
                    && chars[pos] != '_'
                    && !chars[pos].is_whitespace()
                {
                    // Skip punctuation
                    while pos > 0
                        && !chars[pos].is_alphanumeric()
                        && chars[pos] != '_'
                        && !chars[pos].is_whitespace()
                    {
                        pos -= 1;
                    }
                    // If we stopped because of a word character, that's where we want to be
                    if pos < chars.len() && (chars[pos].is_alphanumeric() || chars[pos] == '_') {
                        while pos > 0 && (chars[pos].is_alphanumeric() || chars[pos] == '_') {
                            pos -= 1;
                        }
                        if pos < chars.len() && !chars[pos].is_alphanumeric() && chars[pos] != '_' {
                            pos += 1;
                        }
                    } else {
                        pos += 1; // Move forward one from punctuation
                    }
                }

                let new_pos = (current_line, pos);
                tracing::debug!("move_cursor_to_previous_word: found word at {:?}", new_pos);
                self.set_display_cursor(current_pane, new_pos)?;
                self.ensure_cursor_visible(current_pane);
                moved = true;
                break;
            } else {
                break;
            }
        }

        if !moved {
            tracing::debug!(
                "move_cursor_to_previous_word: no previous word found, staying at current position"
            );
        }

        // Emit events if we moved
        if moved {
            self.emit_view_event([
                ViewEvent::CursorUpdateRequired { pane: current_pane },
                ViewEvent::PositionIndicatorUpdateRequired,
            ]);
        }

        Ok(())
    }

    /// Move cursor to the end of the current or next word
    pub fn move_cursor_to_end_of_word(&mut self) -> Result<()> {
        let current_pane = self.current_pane;
        let current_display_pos = self.get_display_cursor(current_pane);
        let display_cache = self.get_display_cache(current_pane);

        tracing::debug!(
            "move_cursor_to_end_of_word: pane={:?}, current_pos={:?}",
            current_pane,
            current_display_pos
        );

        let mut current_line = current_display_pos.0;
        let mut current_col = current_display_pos.1;
        let mut moved = false;

        // Loop through display lines to find end of word
        while current_line < display_cache.display_line_count() {
            if let Some(line_info) = display_cache.get_display_line(current_line) {
                let chars: Vec<char> = line_info.content.chars().collect();

                // If we're at the end of this line, move to next line
                if current_col >= chars.len() {
                    current_line += 1;
                    current_col = 0;
                    continue;
                }

                let mut pos = current_col;

                // If we're already at the end of a word, move forward to find the next word end
                if pos < chars.len() {
                    // If we're at the end of a word character, move to next word
                    if chars[pos].is_alphanumeric() || chars[pos] == '_' {
                        // Check if we're at the end of the current word
                        if pos + 1 >= chars.len()
                            || !(chars[pos + 1].is_alphanumeric() || chars[pos + 1] == '_')
                        {
                            // We're at the end of a word, move to the next word
                            pos += 1;
                        }
                    }
                    // If we're at the end of punctuation, move to next word
                    else if !chars[pos].is_whitespace() {
                        // Check if we're at the end of punctuation sequence
                        if pos + 1 >= chars.len()
                            || chars[pos + 1].is_whitespace()
                            || chars[pos + 1].is_alphanumeric()
                            || chars[pos + 1] == '_'
                        {
                            // We're at the end of punctuation, move to the next word
                            pos += 1;
                        }
                    }
                }

                // Skip whitespace to find next word
                while pos < chars.len() && chars[pos].is_whitespace() {
                    pos += 1;
                }

                // Now find the end of the current word/punctuation
                if pos < chars.len() && (chars[pos].is_alphanumeric() || chars[pos] == '_') {
                    // Move to end of word
                    while pos < chars.len() && (chars[pos].is_alphanumeric() || chars[pos] == '_') {
                        pos += 1;
                    }
                    // Move back one to be at the last character of the word
                    pos = pos.saturating_sub(1);
                } else if pos < chars.len() && !chars[pos].is_whitespace() {
                    // Move to end of punctuation sequence
                    while pos < chars.len()
                        && !chars[pos].is_whitespace()
                        && !chars[pos].is_alphanumeric()
                        && chars[pos] != '_'
                    {
                        pos += 1;
                    }
                    // Move back one to be at the last punctuation character
                    pos = pos.saturating_sub(1);
                }

                // If we found a valid position on this line and it's different from start
                if pos < chars.len() && pos != current_col {
                    let new_pos = (current_line, pos);
                    tracing::debug!(
                        "move_cursor_to_end_of_word: found word end at {:?}",
                        new_pos
                    );
                    self.set_display_cursor(current_pane, new_pos)?;
                    self.ensure_cursor_visible(current_pane);
                    moved = true;
                    break;
                }

                // Move to next line
                current_line += 1;
                current_col = 0;
            } else {
                break;
            }
        }

        if !moved {
            tracing::debug!(
                "move_cursor_to_end_of_word: no word end found, staying at current position"
            );
        }

        // Emit events if we moved
        if moved {
            self.emit_view_event([
                ViewEvent::CursorUpdateRequired { pane: current_pane },
                ViewEvent::PositionIndicatorUpdateRequired,
            ]);
        }

        Ok(())
    }

    /// Move cursor to a specific line number (1-based)
    pub fn move_cursor_to_line(&mut self, line_number: usize) -> Result<()> {
        if line_number == 0 {
            return Ok(());
        }

        let current_pane = self.current_pane;
        let buffer = &self.panes[current_pane].buffer;

        // Convert to 0-based line number for internal use
        let target_line = line_number.saturating_sub(1);

        // Get the buffer content to check line bounds
        let content = buffer.content();
        let line_count = content.line_count();

        // Clamp to actual number of lines
        let actual_target_line = if line_count == 0 {
            0
        } else {
            std::cmp::min(target_line, line_count - 1)
        };

        // Set cursor to beginning of target line
        let new_position = LogicalPosition {
            line: actual_target_line,
            column: 0,
        };

        self.panes[current_pane].buffer.set_cursor(new_position);

        // Sync display cursor and ensure visibility
        self.sync_display_cursor_with_logical(current_pane)?;
        self.ensure_cursor_visible(current_pane);

        // BUGFIX: Always emit scroll change event for line navigation to force pane redraw
        // This ensures cursor movement is visible even when no actual scrolling occurs
        let (current_v_offset, _current_h_offset) = self.get_scroll_offset(current_pane);
        self.emit_view_event([
            ViewEvent::ScrollChanged {
                pane: current_pane,
                old_offset: current_v_offset,
                new_offset: current_v_offset, // Same offset, but forces redraw
            },
            ViewEvent::CursorUpdateRequired { pane: current_pane },
            ViewEvent::PositionIndicatorUpdateRequired,
        ]);

        Ok(())
    }

    /// Get display height for a pane
    fn get_pane_display_height(&self, pane: Pane) -> usize {
        match pane {
            Pane::Request => self.request_pane_height as usize,
            Pane::Response => {
                if self.response.status_code().is_some() {
                    // BUGFIX: Use saturating_sub to prevent integer underflow panic
                    // This prevents crashes when terminal dimensions are smaller than expected
                    self.terminal_dimensions
                        .1
                        .saturating_sub(self.request_pane_height)
                        .saturating_sub(2) as usize
                // -2 for separator and status
                } else {
                    0
                }
            }
        }
    }
}
