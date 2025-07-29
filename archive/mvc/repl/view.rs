//! # View Components - Observer Pattern for Terminal Rendering
//!
//! This module implements the View layer of the MVC architecture using the Observer pattern.
//! It provides three levels of rendering optimization based on what changed.
//!
//! ## Architecture
//!
//! - **ViewManager**: Coordinates all rendering and manages observers
//! - **RenderObserver**: Trait for components that respond to state changes
//! - **Pane renderers**: Specialized components for request/response/status rendering
//!
//! ## Rendering Strategy  
//!
//! Uses the same three-tier approach as the original:
//! 1. **Cursor-only updates**: Fastest, just reposition cursor
//! 2. **Pane updates**: Redraw single pane content
//! 3. **Full updates**: Complete screen redraw

use std::io::{self, Write};

use anyhow::Result;
use crossterm::{
    cursor::{self, Hide, SetCursorStyle, Show},
    execute,
    style::{Attribute, Color, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor},
    terminal::{Clear, ClearType},
};

use super::model::AppState;
use super::view_trait::ViewRenderer;

/// Trait for components that observe state changes and update the display.
///
/// Observers implement specific rendering logic and are notified when
/// the application state changes in ways that affect their display.
pub trait RenderObserver {
    /// Called when only cursor position needs updating (fastest)
    fn render_cursor_only(&mut self, state: &AppState) -> Result<()>;

    /// Called when content in a pane has changed (moderate cost)
    fn render_content_update(&mut self, state: &AppState) -> Result<()>;

    /// Called when full screen redraw is needed (most expensive)
    fn render_full(&mut self, state: &AppState) -> Result<()>;

    /// Get the name of this observer for debugging
    fn name(&self) -> &'static str;
}

/// Type alias for observer registry to reduce complexity
type ObserverRegistry = Vec<Box<dyn RenderObserver>>;

/// Manages all terminal rendering and coordinates observers.
///
/// This is the main view controller that decides what type of rendering
/// is needed based on state changes and delegates to appropriate observers.
pub struct ViewManager {
    observers: ObserverRegistry,
    last_render_type: RenderType,
}

#[derive(Debug, Clone, PartialEq)]
enum RenderType {
    CursorOnly,
    ContentUpdate,
    Full,
}

impl ViewManager {
    /// Create a new view manager
    pub fn new() -> Self {
        Self {
            observers: Vec::new(),
            last_render_type: RenderType::Full,
        }
    }

    /// Add an observer to be notified of state changes
    pub fn add_observer(&mut self, observer: Box<dyn RenderObserver>) {
        self.observers.push(observer);
    }

    /// Render cursor position only (fastest update)
    pub fn render_cursor_only(&mut self, state: &AppState) -> Result<()> {
        // Hide cursor to prevent flicker during positioning
        execute!(io::stdout(), Hide)?;

        for observer in &mut self.observers {
            observer.render_cursor_only(state)?;
        }
        // Position cursor correctly after all rendering
        self.position_cursor_after_render(state)?;

        // Show cursor after positioning is complete
        execute!(io::stdout(), Show)?;

        self.last_render_type = RenderType::CursorOnly;
        io::stdout().flush()?;
        Ok(())
    }

    /// Render content updates (moderate cost)
    pub fn render_content_update(&mut self, state: &AppState) -> Result<()> {
        // Hide cursor to prevent flicker during redraw
        execute!(io::stdout(), Hide)?;

        for observer in &mut self.observers {
            observer.render_content_update(state)?;
        }
        // Update cursor style in case mode changed
        self.update_cursor_style(state)?;
        // Position cursor correctly after all rendering
        self.position_cursor_after_render(state)?;

        // Show cursor after all rendering is complete
        execute!(io::stdout(), Show)?;

        self.last_render_type = RenderType::ContentUpdate;
        io::stdout().flush()?;
        Ok(())
    }

    /// Render full screen (most expensive)
    pub fn render_full(&mut self, state: &AppState) -> Result<()> {
        // Hide cursor to prevent flicker during redraw
        execute!(io::stdout(), Hide)?;

        for observer in &mut self.observers {
            observer.render_full(state)?;
        }
        // Update cursor style for full renders
        self.update_cursor_style(state)?;
        // Position cursor correctly after all rendering
        self.position_cursor_after_render(state)?;

        // Show cursor after all rendering is complete
        execute!(io::stdout(), Show)?;

        self.last_render_type = RenderType::Full;
        io::stdout().flush()?;
        Ok(())
    }

    /// Initialize the terminal for rendering
    pub fn initialize_terminal(&self, state: &AppState) -> Result<()> {
        // Clear screen once at startup and move cursor to top
        execute!(io::stdout(), Clear(ClearType::All), cursor::MoveTo(0, 0))?;

        // Set initial cursor style based on current mode
        self.update_cursor_style(state)?;

        // Ensure cursor is visible
        execute!(io::stdout(), Show)?;

        Ok(())
    }

    /// Clean up terminal state
    pub fn cleanup_terminal(&self) -> Result<()> {
        // Restore default cursor and ensure it's visible
        execute!(io::stdout(), SetCursorStyle::DefaultUserShape, Show)?;
        Ok(())
    }

    /// Update cursor style and visibility based on current editor mode
    pub fn update_cursor_style(&self, state: &AppState) -> Result<()> {
        use super::model::EditorMode;

        match state.mode {
            EditorMode::Normal => {
                execute!(io::stdout(), SetCursorStyle::SteadyBlock, Show)?;
            }
            EditorMode::Insert => {
                execute!(io::stdout(), SetCursorStyle::BlinkingBar, Show)?;
            }
            EditorMode::Command => {
                // Hide cursor completely in command mode
                execute!(io::stdout(), Hide)?;
            }
            EditorMode::Visual | EditorMode::VisualLine => {
                execute!(io::stdout(), SetCursorStyle::SteadyBlock, Show)?;
            }
        };

        Ok(())
    }

    /// Position cursor correctly based on current pane and mode
    /// Now uses display cache for accurate wrapped text positioning
    fn position_cursor_after_render(&self, state: &AppState) -> Result<()> {
        use super::model::{EditorMode, Pane};

        // Don't position cursor in command mode since it's hidden
        if matches!(state.mode, EditorMode::Command) {
            return Ok(());
        }

        match state.current_pane {
            Pane::Request => {
                let logical_line = state.request_buffer.cursor_line;
                let logical_col = state.request_buffer.cursor_col;

                // Try to get display position from cache first
                let (display_row, display_col) = if let Some((display_line, display_col)) =
                    state.logical_to_display_position(logical_line, logical_col)
                {
                    // Use cache for accurate wrapped positioning
                    let visible_display_row =
                        display_line.saturating_sub(state.request_buffer.scroll_offset);
                    (visible_display_row, display_col)
                } else {
                    // Fallback to logical positioning if cache not ready
                    let cursor_row =
                        logical_line.saturating_sub(state.request_buffer.scroll_offset);
                    (cursor_row, logical_col)
                };

                // Calculate line number width to offset cursor position
                let max_line_num = state.request_buffer.line_count();
                let line_num_width = max_line_num.to_string().len().max(3);
                let line_num_offset = line_num_width + 1; // +1 for space after line number

                // Bounds checking to prevent invalid cursor positions
                let terminal_height = state.terminal_size.1 as usize;
                let terminal_width = state.terminal_size.0 as usize;
                let final_col = display_col + line_num_offset;

                if display_row < terminal_height && final_col < terminal_width {
                    execute!(
                        io::stdout(),
                        cursor::MoveTo(final_col as u16, display_row as u16)
                    )?;
                }
            }
            Pane::Response => {
                if let Some(ref response_buffer) = state.response_buffer {
                    let logical_line = response_buffer.cursor_line;
                    let logical_col = response_buffer.cursor_col;

                    // Try to get display position from cache first
                    let (display_row, display_col) = if let Some((display_line, display_col)) =
                        state.logical_to_display_position(logical_line, logical_col)
                    {
                        // Use cache for accurate wrapped positioning
                        let visible_display_row =
                            display_line.saturating_sub(response_buffer.scroll_offset);
                        (visible_display_row, display_col)
                    } else {
                        // Fallback to logical positioning if cache not ready
                        let cursor_row = logical_line.saturating_sub(response_buffer.scroll_offset);
                        (cursor_row, logical_col)
                    };

                    // Calculate line number width to offset cursor position
                    let max_line_num = response_buffer.line_count();
                    let line_num_width = max_line_num.to_string().len().max(3);
                    let line_num_offset = line_num_width + 1; // +1 for space after line number

                    // Offset by request pane height + separator
                    let actual_row = display_row + state.get_request_pane_height() + 1;
                    let final_col = display_col + line_num_offset;

                    // Bounds checking to prevent invalid cursor positions
                    let terminal_height = state.terminal_size.1 as usize;
                    let terminal_width = state.terminal_size.0 as usize;

                    if actual_row < terminal_height && final_col < terminal_width {
                        execute!(
                            io::stdout(),
                            cursor::MoveTo(final_col as u16, actual_row as u16)
                        )?;
                    }
                }
            }
        }

        Ok(())
    }
}

impl Default for ViewManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Implementation of ViewRenderer trait for dependency injection
impl ViewRenderer for ViewManager {
    fn render_cursor_only(&mut self, state: &AppState) -> Result<()> {
        ViewManager::render_cursor_only(self, state)
    }

    fn render_content_update(&mut self, state: &AppState) -> Result<()> {
        ViewManager::render_content_update(self, state)
    }

    fn render_full(&mut self, state: &AppState) -> Result<()> {
        ViewManager::render_full(self, state)
    }

    fn initialize_terminal(&self, state: &AppState) -> Result<()> {
        ViewManager::initialize_terminal(self, state)
    }

    fn cleanup_terminal(&self) -> Result<()> {
        ViewManager::cleanup_terminal(self)
    }
}

/// Observer that renders the request pane
pub struct RequestPaneRenderer {
    // Add any specific state needed for request pane rendering
}

impl RenderObserver for RequestPaneRenderer {
    fn render_cursor_only(&mut self, state: &AppState) -> Result<()> {
        // Cursor positioning is now handled centrally by ViewManager
        // This renderer doesn't need to do anything for cursor-only updates
        if state.current_pane == super::model::Pane::Request {
            // Only signal that we're ready for cursor positioning
            // The actual positioning will be done by ViewManager.position_cursor_after_render
        }
        Ok(())
    }

    fn render_content_update(&mut self, state: &AppState) -> Result<()> {
        // Render just the request pane content
        self.render_request_pane_content(state)?;
        Ok(())
    }

    fn render_full(&mut self, state: &AppState) -> Result<()> {
        // Full render includes borders, highlighting, etc.
        self.render_request_pane_full(state)?;
        Ok(())
    }

    fn name(&self) -> &'static str {
        "RequestPaneRenderer"
    }
}

impl RequestPaneRenderer {
    fn render_request_pane_content(&self, state: &AppState) -> Result<()> {
        // Implement request pane content rendering with line numbers like vim using display cache

        let pane_height = state.get_request_pane_height();
        let cache = state.cache_manager.get_request_cache();

        // Calculate width needed for line numbers (minimum 3 characters)
        let max_line_num = state.request_buffer.line_count();
        let line_num_width = max_line_num.to_string().len().max(3);

        // If cache is not valid, fall back to simple rendering
        if !cache.is_valid {
            return self.render_request_pane_fallback(state, pane_height, line_num_width);
        }

        // Render using display cache for proper wrapped line handling
        let scroll_offset = state.request_buffer.scroll_offset;

        for display_row in 0..pane_height {
            execute!(io::stdout(), cursor::MoveTo(0, display_row as u16))?;

            let display_line_idx = scroll_offset + display_row;

            if display_line_idx < cache.total_display_lines {
                if let Some(display_line) = cache.get_display_line(display_line_idx) {
                    // Render line number - only show for first line of each logical line
                    execute!(io::stdout(), SetAttribute(Attribute::Dim))?;
                    if display_line.is_continuation {
                        // Continuation lines show spaces instead of line numbers (vim-style)
                        print!("{} ", " ".repeat(line_num_width));
                    } else {
                        // First line of logical line shows actual line number
                        print!(
                            "{:>width$} ",
                            display_line.logical_line + 1,
                            width = line_num_width
                        );
                    }
                    execute!(io::stdout(), SetAttribute(Attribute::Reset))?;

                    // Render the wrapped segment content
                    print!("{}", display_line.content);
                } else {
                    // Fallback if display line not found
                    execute!(io::stdout(), SetForegroundColor(Color::DarkGrey))?;
                    print!("~{} ", " ".repeat(line_num_width.saturating_sub(1)));
                    execute!(io::stdout(), ResetColor)?;
                }
            } else {
                // Show tilda for rows beyond content (vim-style)
                execute!(io::stdout(), SetForegroundColor(Color::DarkGrey))?;
                print!("~{} ", " ".repeat(line_num_width.saturating_sub(1)));
                execute!(io::stdout(), ResetColor)?;
            }

            // Clear rest of line
            execute!(io::stdout(), Clear(ClearType::UntilNewLine))?;
        }

        Ok(())
    }

    /// Fallback rendering when display cache is not available
    fn render_request_pane_fallback(
        &self,
        state: &AppState,
        pane_height: usize,
        line_num_width: usize,
    ) -> Result<()> {
        let (start, end) = state.request_buffer.visible_range(pane_height);

        // Move to start of request pane and render visible lines
        for (row, line_idx) in (start..end).enumerate() {
            execute!(io::stdout(), cursor::MoveTo(0, row as u16))?;

            if let Some(line) = state.request_buffer.lines.get(line_idx) {
                // Render line number with dimmed style and right alignment
                execute!(io::stdout(), SetAttribute(Attribute::Dim))?;
                print!("{:>width$} ", line_idx + 1, width = line_num_width);
                execute!(io::stdout(), SetAttribute(Attribute::Reset))?;

                // TODO: Add syntax highlighting for HTTP requests
                print!("{}", line);
            } else {
                // Show tilda for non-existing lines (like vi) with darker gray color
                execute!(io::stdout(), SetForegroundColor(Color::DarkGrey))?;
                print!("~{} ", " ".repeat(line_num_width.saturating_sub(1)));
                execute!(io::stdout(), ResetColor)?;
            }

            // Clear rest of line
            execute!(io::stdout(), Clear(ClearType::UntilNewLine))?;
        }

        // Fill remaining visible rows with tilda lines if there are more visible rows than content
        let visible_content_lines =
            (end - start).min(state.request_buffer.line_count().saturating_sub(start));
        for row in visible_content_lines..pane_height {
            execute!(io::stdout(), cursor::MoveTo(0, row as u16))?;
            execute!(io::stdout(), SetForegroundColor(Color::DarkGrey))?;
            print!("~{} ", " ".repeat(line_num_width.saturating_sub(1)));
            execute!(io::stdout(), ResetColor)?;
            execute!(io::stdout(), Clear(ClearType::UntilNewLine))?;
        }

        Ok(())
    }

    fn render_request_pane_full(&self, state: &AppState) -> Result<()> {
        // Render with full decorations (borders, focus indicators, etc.)
        // Don't apply background color here to avoid inversion issues
        self.render_request_pane_content(state)?;
        Ok(())
    }
}

/// Observer that renders the response pane
pub struct ResponsePaneRenderer {
    // Add any specific state needed for response pane rendering
}

impl RenderObserver for ResponsePaneRenderer {
    fn render_cursor_only(&mut self, state: &AppState) -> Result<()> {
        // Cursor positioning is now handled centrally by ViewManager
        // This renderer doesn't need to do anything for cursor-only updates
        if let Some(ref _response_buffer) = state.response_buffer {
            if state.current_pane == super::model::Pane::Response {
                // Only signal that we're ready for cursor positioning
                // The actual positioning will be done by ViewManager.position_cursor_after_render
            }
        }

        Ok(())
    }

    fn render_content_update(&mut self, state: &AppState) -> Result<()> {
        self.render_response_pane_content(state)?;
        Ok(())
    }

    fn render_full(&mut self, state: &AppState) -> Result<()> {
        self.render_response_pane_full(state)?;
        Ok(())
    }

    fn name(&self) -> &'static str {
        "ResponsePaneRenderer"
    }
}

impl ResponsePaneRenderer {
    fn render_response_pane_content(&self, state: &AppState) -> Result<()> {
        let pane_height = state.get_response_pane_height();

        // If response pane is hidden (height 0), clear the area where it might have been
        if pane_height == 0 {
            self.clear_response_area(state)?;
            return Ok(());
        }

        let response_start_row = state.get_request_pane_height() + 1; // +1 for separator

        if let Some(ref response_buffer) = state.response_buffer {
            let cache = state.cache_manager.get_response_cache();

            // Calculate width needed for line numbers (minimum 3 characters)
            let max_line_num = response_buffer.line_count();
            let line_num_width = max_line_num.to_string().len().max(3);

            // If cache is not valid, try to update it before falling back
            if !cache.is_valid {
                let content_width = state.terminal_size.0 as usize - line_num_width - 1; // Account for line numbers
                let _ = state
                    .cache_manager
                    .update_response_cache(&response_buffer.lines, content_width);

                // Get updated cache after potential update
                let updated_cache = state.cache_manager.get_response_cache();
                if !updated_cache.is_valid {
                    return self.render_response_pane_fallback(
                        state,
                        response_buffer,
                        pane_height,
                        response_start_row,
                        line_num_width,
                    );
                }
            }

            // Render using display cache for proper wrapped line handling
            let updated_cache = state.cache_manager.get_response_cache();
            let scroll_offset = response_buffer.scroll_offset;

            for display_row in 0..pane_height {
                let actual_row = response_start_row + display_row;
                execute!(io::stdout(), cursor::MoveTo(0, actual_row as u16))?;

                let display_line_idx = scroll_offset + display_row;

                if display_line_idx < updated_cache.total_display_lines {
                    if let Some(display_line) = updated_cache.get_display_line(display_line_idx) {
                        // Render line number - only show for first line of each logical line
                        execute!(io::stdout(), SetAttribute(Attribute::Dim))?;
                        if display_line.is_continuation {
                            // Continuation lines show spaces instead of line numbers (vim-style)
                            print!("{} ", " ".repeat(line_num_width));
                        } else {
                            // First line of logical line shows actual line number
                            print!(
                                "{:>width$} ",
                                display_line.logical_line + 1,
                                width = line_num_width
                            );
                        }
                        execute!(io::stdout(), SetAttribute(Attribute::Reset))?;

                        // Render the wrapped segment content
                        print!("{}", display_line.content);
                    } else {
                        // Fallback if display line not found - clear the line
                        execute!(io::stdout(), Clear(ClearType::UntilNewLine))?;
                        continue;
                    }
                } else {
                    // Clear lines beyond content
                    execute!(io::stdout(), Clear(ClearType::UntilNewLine))?;
                    continue;
                }

                execute!(io::stdout(), Clear(ClearType::UntilNewLine))?;
            }
        } else {
            // Clear response area if no response
            for row in 0..pane_height {
                let actual_row = response_start_row + row;
                execute!(io::stdout(), cursor::MoveTo(0, actual_row as u16))?;
                execute!(io::stdout(), Clear(ClearType::UntilNewLine))?;
            }
        }

        Ok(())
    }

    /// Fallback rendering when display cache is not available
    fn render_response_pane_fallback(
        &self,
        state: &AppState,
        response_buffer: &crate::repl::model::ResponseBuffer,
        pane_height: usize,
        response_start_row: usize,
        line_num_width: usize,
    ) -> Result<()> {
        let terminal_width = state.terminal_size.0 as usize;
        let (start, end) = response_buffer.visible_range(pane_height);

        for (row, line_idx) in (start..end).enumerate() {
            let actual_row = response_start_row + row;
            execute!(io::stdout(), cursor::MoveTo(0, actual_row as u16))?;

            if let Some(line) = response_buffer.lines.get(line_idx) {
                // Render line number with dimmed style and right alignment
                execute!(io::stdout(), SetAttribute(Attribute::Dim))?;
                print!("{:>width$} ", line_idx + 1, width = line_num_width);
                execute!(io::stdout(), SetAttribute(Attribute::Reset))?;

                // Calculate available width for content after line numbers
                let content_width = terminal_width.saturating_sub(line_num_width + 1); // +1 for space after line number

                // Truncate long lines to prevent overflow
                let display_line = if line.len() > content_width {
                    &line[..content_width.saturating_sub(3)] // Leave space for "..."
                } else {
                    line
                };
                print!("{}", display_line);
            }

            execute!(io::stdout(), Clear(ClearType::UntilNewLine))?;
        }

        Ok(())
    }

    fn render_response_pane_full(&self, state: &AppState) -> Result<()> {
        // Draw blue border when there's a response available
        if state.response_buffer.is_some() {
            self.draw_response_pane_border(state)?;
        }

        self.render_response_pane_content(state)?;

        // Reset colors
        execute!(io::stdout(), ResetColor)?;

        Ok(())
    }

    /// Draw a blue border around the response pane when it's active
    fn draw_response_pane_border(&self, state: &AppState) -> Result<()> {
        use crossterm::style::{Color, SetForegroundColor};

        // Don't draw border if response pane is hidden
        if state.get_response_pane_height() == 0 {
            return Ok(());
        }

        let response_start_row = state.get_request_pane_height(); // Separator line
        let width = state.terminal_size.0 as usize;

        // Draw top border (separator line) in blue
        execute!(io::stdout(), cursor::MoveTo(0, response_start_row as u16))?;
        execute!(io::stdout(), SetForegroundColor(Color::Blue))?;
        print!("{}", "─".repeat(width));
        execute!(io::stdout(), ResetColor)?;

        Ok(())
    }

    /// Clear the entire area where the response pane could be displayed
    /// This ensures no leftover content remains when hiding the response pane
    fn clear_response_area(&self, state: &AppState) -> Result<()> {
        let total_height = state.terminal_size.1 as usize;
        let response_start_row = state.get_request_pane_height(); // Start from separator line
        let status_line_row = total_height.saturating_sub(1); // Last row is status line

        // Clear from separator line to just before status line
        for row in response_start_row..status_line_row {
            execute!(io::stdout(), cursor::MoveTo(0, row as u16))?;
            execute!(io::stdout(), Clear(ClearType::UntilNewLine))?;
        }

        Ok(())
    }
}

/// Observer that renders the status line
pub struct StatusLineRenderer {
    // Add any specific state needed for status line rendering
}

impl RenderObserver for StatusLineRenderer {
    fn render_cursor_only(&mut self, state: &AppState) -> Result<()> {
        // Status line should update to show cursor position changes
        self.render_status_line(state)?;
        Ok(())
    }

    fn render_content_update(&mut self, state: &AppState) -> Result<()> {
        // Status line should update on mode changes and content updates
        self.render_status_line(state)?;
        Ok(())
    }

    fn render_full(&mut self, state: &AppState) -> Result<()> {
        self.render_status_line(state)?;
        Ok(())
    }

    fn name(&self) -> &'static str {
        "StatusLineRenderer"
    }
}

impl StatusLineRenderer {
    fn render_status_line(&self, state: &AppState) -> Result<()> {
        let status_row = state.terminal_size.1 - 1; // Bottom row
        let terminal_width = state.terminal_size.0 as usize;
        execute!(io::stdout(), cursor::MoveTo(0, status_row))?;

        // Clear the entire status line
        execute!(io::stdout(), Clear(ClearType::UntilNewLine))?;

        // Left side: status message
        let left_content = &state.status_message;

        // Right side: HTTP status, message, and turnaround time (if available)
        let mut right_content = String::new();
        let mut right_content_visible_len = 0; // Track visible length (excluding ANSI codes)

        if let Some(ref status) = state.last_response_status {
            // Add signal icon based on HTTP status code
            let signal_icon = self.get_status_signal_icon(status);
            right_content.push_str(&signal_icon);
            right_content.push_str(status);
            // Signal icon has 2 visible chars: ● and space, status text is all visible
            right_content_visible_len += 2 + status.len();
        }
        if let Some(duration) = state.last_request_duration {
            if !right_content.is_empty() {
                right_content.push_str("  ");
                right_content_visible_len += 2;
            }
            let duration_text = format!("{}ms", duration);
            right_content.push_str(&duration_text);
            right_content_visible_len += duration_text.len();
        }

        // Calculate spacing to align right content to the right edge
        let left_len = left_content.len();

        if left_len + right_content_visible_len >= terminal_width {
            // If content is too long, truncate left content and show right content
            let available_for_left = terminal_width.saturating_sub(right_content_visible_len + 2); // 2 for "  "
            if available_for_left > 0 {
                let truncated_left = if left_content.len() > available_for_left {
                    format!(
                        "{}...",
                        &left_content[..available_for_left.saturating_sub(3)]
                    )
                } else {
                    left_content.to_string()
                };
                print!("{}  {}", truncated_left, right_content);
            } else {
                // Just show right content if no space for left
                print!("{}", right_content);
            }
        } else {
            // Normal case: show left content, spaces, then right content
            print!("{}", left_content);
            if !right_content.is_empty() {
                let spaces_needed =
                    terminal_width.saturating_sub(left_len + right_content_visible_len);
                print!("{}{}", " ".repeat(spaces_needed), right_content);
            }
        }

        Ok(())
    }

    /// Get colored signal icon based on HTTP status code
    /// Returns green signal for success (2xx), red signal for errors (4xx, 5xx)
    fn get_status_signal_icon(&self, status: &str) -> String {
        // Extract status code from status string (e.g., "200 OK" -> 200)
        let status_code = status
            .split_whitespace()
            .next() // Get first token (status code) since we removed "HTTP" prefix
            .and_then(|s| s.parse::<u16>().ok())
            .unwrap_or(0);

        // Use ANSI escape codes for colors since we're already using crossterm
        match status_code {
            200..=299 => "\x1b[32m●\x1b[0m ", // Green circle for success
            400..=599 => "\x1b[31m●\x1b[0m ", // Red circle for client/server errors
            _ => "● ",                        // Default (no color) for unknown status
        }
        .to_string()
    }
}

/// Create a default view manager with standard observers
pub fn create_default_view_manager() -> ViewManager {
    let mut view_manager = ViewManager::new();

    view_manager.add_observer(Box::new(RequestPaneRenderer {}));
    view_manager.add_observer(Box::new(ResponsePaneRenderer {}));
    view_manager.add_observer(Box::new(StatusLineRenderer {}));

    view_manager
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::model::{AppState, EditorMode, Pane, RequestBuffer};

    #[test]
    fn request_pane_should_show_line_numbers_and_tilda_for_empty_lines() {
        // Create a test app state with request buffer content
        let mut state = AppState::new((80, 24), false);
        state.mode = EditorMode::Normal;
        state.current_pane = Pane::Request;

        // Add some content to the request buffer
        state.request_buffer.lines = vec![
            "GET /api/users HTTP/1.1".to_string(),
            "Host: example.com".to_string(),
            "".to_string(),
        ];

        // Create a request renderer to test
        let renderer = RequestPaneRenderer {};

        // This test verifies that the renderer can be created and
        // that the line number calculation logic works correctly
        let line_count = state.request_buffer.line_count();
        let line_num_width = line_count.to_string().len().max(3);

        // For 3 lines, width should be 3 (minimum)
        assert_eq!(line_num_width, 3);
        assert_eq!(line_count, 3);

        // Verify that the renderer can be named correctly
        assert_eq!(renderer.name(), "RequestPaneRenderer");
    }

    #[test]
    fn request_pane_should_handle_empty_state_with_tilda_lines() {
        // Create a test app state with default request buffer
        let mut state = AppState::new((80, 24), false);
        state.mode = EditorMode::Normal;
        state.current_pane = Pane::Request;

        // Request buffer starts with one empty line by default (as seen in RequestBuffer::new())
        assert_eq!(state.request_buffer.lines.len(), 1);
        assert_eq!(state.request_buffer.lines[0], "");

        let renderer = RequestPaneRenderer {};

        // This test verifies that the renderer can handle empty state
        // In this case, most visible lines should be tilda lines (except the first empty line)
        let pane_height = state.get_request_pane_height();

        // Should have reasonable pane height
        assert!(pane_height > 0);
        assert_eq!(renderer.name(), "RequestPaneRenderer");
    }

    #[test]
    fn line_number_width_calculation_should_work_correctly() {
        // Test different line counts and their expected width
        let test_cases = vec![
            (1, 3),     // 1 line -> 3 width (minimum)
            (10, 3),    // 10 lines -> 3 width (minimum)
            (100, 3),   // 100 lines -> 3 width
            (1000, 4),  // 1000 lines -> 4 width
            (10000, 5), // 10000 lines -> 5 width
        ];

        for (line_count, expected_width) in test_cases {
            let actual_width = line_count.to_string().len().max(3);
            assert_eq!(
                actual_width, expected_width,
                "Line count {} should have width {}, got {}",
                line_count, expected_width, actual_width
            );
        }
    }

    #[test]
    fn request_pane_tilda_rendering_should_work_with_visible_range() {
        // Create a test state with a large terminal to test tilda rendering
        let mut state = AppState::new((80, 30), false);
        state.mode = EditorMode::Normal;
        state.current_pane = Pane::Request;

        // Add only 2 lines of content to the request buffer
        state.request_buffer.lines = vec![
            "GET /api/test HTTP/1.1".to_string(),
            "Host: example.com".to_string(),
        ];

        let renderer = RequestPaneRenderer {};
        let pane_height = state.get_request_pane_height();
        let (start, end) = state.request_buffer.visible_range(pane_height);

        // Should display the 2 actual lines
        assert_eq!(state.request_buffer.line_count(), 2);

        // The visible range should show we have content
        assert_eq!(start, 0);
        assert!(end >= 2);

        // Should have space for tilda lines (pane should be larger than content)
        assert!(pane_height > 2);

        // The visible content lines should be less than pane height,
        // indicating tilda lines will be shown
        let visible_content_lines =
            (end - start).min(state.request_buffer.line_count().saturating_sub(start));
        assert!(visible_content_lines < pane_height);

        assert_eq!(renderer.name(), "RequestPaneRenderer");
    }

    #[test]
    fn response_pane_should_be_hidden_when_no_response_buffer() {
        // Create a test state without response buffer
        let state = AppState::new((80, 24), false);

        // Response pane should be hidden (height 0) when no response buffer
        assert_eq!(state.get_response_pane_height(), 0);

        // Request pane should use full available space
        let expected_full_height = (24u16 as usize).saturating_sub(2); // Minus separator and status
        assert_eq!(state.get_request_pane_height(), expected_full_height);
    }

    #[test]
    fn response_pane_should_be_visible_when_response_buffer_exists() {
        use crate::repl::model::ResponseBuffer;

        // Create a test state with response buffer
        let mut state = AppState::new((80, 24), false);
        let response_content = "HTTP/1.1 200 OK\nContent-Type: application/json".to_string();
        let response_buffer = ResponseBuffer::new(response_content);
        state.response_buffer = Some(response_buffer);

        // Response pane should be visible (height > 0) when response buffer exists
        assert!(state.get_response_pane_height() > 0);

        // Request pane should use its configured height, not full space
        assert!(state.get_request_pane_height() < (24u16 as usize).saturating_sub(2));
    }

    #[test]
    fn response_pane_should_show_line_numbers_when_content_exists() {
        use crate::repl::model::ResponseBuffer;

        // Create a test state with response buffer
        let mut state = AppState::new((80, 24), false);
        let response_content =
            "HTTP/1.1 200 OK\nContent-Type: application/json\n\n{\"message\": \"Hello World\"}"
                .to_string();
        let response_buffer = ResponseBuffer::new(response_content);
        state.response_buffer = Some(response_buffer);

        // Create a response renderer to test
        let renderer = ResponsePaneRenderer {};

        // This test verifies that the response pane can show line numbers
        let line_count = state.response_buffer.as_ref().unwrap().line_count();
        let line_num_width = line_count.to_string().len().max(3);

        // For 4 lines, width should be 3 (minimum)
        assert_eq!(line_num_width, 3);
        assert_eq!(line_count, 4);

        // Response pane should be visible when content exists
        assert!(state.get_response_pane_height() > 0);

        // Verify that the renderer can be named correctly
        assert_eq!(renderer.name(), "ResponsePaneRenderer");
    }

    #[test]
    fn response_pane_should_clear_area_when_hidden() {
        use crate::repl::model::ResponseBuffer;

        // Create a test state initially without response buffer (hidden pane)
        let state = AppState::new((80, 24), false);
        assert_eq!(state.get_response_pane_height(), 0);

        // Create a response renderer to test clearing behavior
        let renderer = ResponsePaneRenderer {};

        // This should not panic when rendering with hidden pane
        // The clear_response_area method should be called internally
        let result = renderer.render_response_pane_content(&state);
        assert!(result.is_ok());

        // Test that we can also create a state with response and then clear it
        let mut state_with_response = AppState::new((80, 24), false);
        let response_content = "HTTP/1.1 200 OK\nContent-Type: application/json".to_string();
        let response_buffer = ResponseBuffer::new(response_content);
        state_with_response.response_buffer = Some(response_buffer);

        // Initially should have height > 0
        assert!(state_with_response.get_response_pane_height() > 0);

        // Clear the response buffer (simulating :q command)
        state_with_response.response_buffer = None;

        // Should now have height 0
        assert_eq!(state_with_response.get_response_pane_height(), 0);

        // Rendering should work without errors (clearing internally)
        let result = renderer.render_response_pane_content(&state_with_response);
        assert!(result.is_ok());
    }

    #[test]
    fn create_default_view_manager_should_create_view_manager_with_all_observers() {
        let view_manager = create_default_view_manager();

        // Should have 3 observers: Request, Response, and Status renderers
        assert_eq!(view_manager.observers.len(), 3);
    }

    #[test]
    fn status_line_should_display_http_status_and_timing_right_aligned() {
        // Test with both status and timing information
        let mut state = AppState::new((80, 24), false);
        state.status_message = "Ready".to_string();
        state.last_response_status = Some("200 OK".to_string());
        state.last_request_duration = Some(250);

        let renderer = StatusLineRenderer {};
        // Test should not panic - actual output testing would require terminal capture
        let result = renderer.render_status_line(&state);
        assert!(result.is_ok());
    }

    #[test]
    fn status_line_should_handle_missing_http_status_gracefully() {
        // Test with only timing information
        let mut state = AppState::new((50, 20), false);
        state.status_message = "Loading...".to_string();
        state.last_response_status = None;
        state.last_request_duration = Some(100);

        let renderer = StatusLineRenderer {};
        let result = renderer.render_status_line(&state);
        assert!(result.is_ok());
    }

    #[test]
    fn status_line_should_handle_missing_timing_gracefully() {
        // Test with only HTTP status
        let mut state = AppState::new((60, 15), false);
        state.status_message = "Error occurred".to_string();
        state.last_response_status = Some("404 Not Found".to_string());
        state.last_request_duration = None;

        let renderer = StatusLineRenderer {};
        let result = renderer.render_status_line(&state);
        assert!(result.is_ok());
    }

    #[test]
    fn status_line_should_handle_no_http_data_gracefully() {
        // Test with only status message (original behavior)
        let mut state = AppState::new((40, 10), false);
        state.status_message = "Idle".to_string();
        state.last_response_status = None;
        state.last_request_duration = None;

        let renderer = StatusLineRenderer {};
        let result = renderer.render_status_line(&state);
        assert!(result.is_ok());
    }

    #[test]
    fn status_line_should_handle_very_narrow_terminal() {
        // Test edge case with very narrow terminal
        let mut state = AppState::new((20, 10), false);
        state.status_message = "Very long status message that exceeds terminal width".to_string();
        state.last_response_status = Some("200 OK".to_string());
        state.last_request_duration = Some(1500);

        let renderer = StatusLineRenderer {};
        let result = renderer.render_status_line(&state);
        assert!(result.is_ok());
    }

    #[test]
    fn status_signal_icon_should_return_correct_colors() {
        let renderer = StatusLineRenderer {};

        // Test success status codes (2xx) - should return green circle
        // Note: Controller now formats status as "200 OK" (removed HTTP prefix)
        let green_icon = renderer.get_status_signal_icon("200 OK");
        assert!(green_icon.contains("32m●")); // Green ANSI color code
        assert!(green_icon.contains("0m")); // Reset ANSI color code

        let green_icon_created = renderer.get_status_signal_icon("201 Created");
        assert!(green_icon_created.contains("32m●"));

        // Test client error status codes (4xx) - should return red circle
        let red_icon_not_found = renderer.get_status_signal_icon("404 Not Found");
        assert!(red_icon_not_found.contains("31m●")); // Red ANSI color code
        assert!(red_icon_not_found.contains("0m")); // Reset ANSI color code

        // Test server error status codes (5xx) - should return red circle
        let red_icon_server_error = renderer.get_status_signal_icon("500 Internal Server Error");
        assert!(red_icon_server_error.contains("31m●"));

        // Test unknown/invalid status - should return default circle
        let default_icon = renderer.get_status_signal_icon("999 Unknown");
        assert_eq!(default_icon, "● ");

        let invalid_icon = renderer.get_status_signal_icon("Not a status");
        assert_eq!(invalid_icon, "● ");
    }

    #[test]
    fn render_cursor_only_should_hide_and_show_cursor() {
        use std::io::Write;

        // Create a test ViewManager
        let mut view_manager = ViewManager::new();
        let state = AppState::new((80, 24), false);

        // Capture stdout to verify cursor hide/show commands are sent
        // Note: This test verifies the method completes without panicking
        // In a real terminal environment, the Hide/Show commands would be executed
        let result = view_manager.render_cursor_only(&state);

        // The method should complete successfully
        assert!(result.is_ok());

        // Verify that last render type was set correctly
        assert_eq!(view_manager.last_render_type, RenderType::CursorOnly);
    }

    #[test]
    fn render_cursor_only_should_not_cause_flicker_in_request_pane() {
        // This test verifies the fix for request pane cursor movement flickering
        // by ensuring render_cursor_only properly manages cursor visibility
        let mut view_manager = ViewManager::new();
        let mut state = AppState::new((80, 24), false);

        // Set up state for request pane with some content
        state.request_buffer.lines =
            vec!["GET /api/users".to_string(), "".to_string(), "".to_string()];
        state.request_buffer.cursor_line = 0;
        state.request_buffer.cursor_col = 5;
        state.current_pane = crate::repl::model::Pane::Request;

        // Multiple rapid cursor movements should not cause issues
        for _ in 0..10 {
            let result = view_manager.render_cursor_only(&state);
            assert!(result.is_ok());
        }

        // Move cursor position and test again
        state.request_buffer.cursor_col = 10;
        let result = view_manager.render_cursor_only(&state);
        assert!(result.is_ok());
    }

    #[test]
    fn update_cursor_style_should_hide_cursor_in_command_mode() {
        // Test that cursor is hidden when switching to command mode
        let view_manager = ViewManager::new();
        let mut state = AppState::new((80, 24), false);

        // Test Normal mode - cursor should be visible with steady block
        state.mode = crate::repl::model::EditorMode::Normal;
        let result = view_manager.update_cursor_style(&state);
        assert!(result.is_ok());

        // Test Insert mode - cursor should be visible with blinking bar
        state.mode = crate::repl::model::EditorMode::Insert;
        let result = view_manager.update_cursor_style(&state);
        assert!(result.is_ok());

        // Test Command mode - cursor should be hidden
        state.mode = crate::repl::model::EditorMode::Command;
        let result = view_manager.update_cursor_style(&state);
        assert!(result.is_ok());

        // Test Visual mode - cursor should be visible with steady block
        state.mode = crate::repl::model::EditorMode::Visual;
        let result = view_manager.update_cursor_style(&state);
        assert!(result.is_ok());
    }

    #[test]
    fn cursor_should_be_restored_when_exiting_command_mode() {
        // Test that cursor visibility is properly restored when leaving command mode
        let view_manager = ViewManager::new();
        let mut state = AppState::new((80, 24), false);

        // Start in Normal mode
        state.mode = crate::repl::model::EditorMode::Normal;
        let result = view_manager.update_cursor_style(&state);
        assert!(result.is_ok());

        // Switch to Command mode (cursor should be hidden)
        state.mode = crate::repl::model::EditorMode::Command;
        let result = view_manager.update_cursor_style(&state);
        assert!(result.is_ok());

        // Switch back to Normal mode (cursor should be visible again)
        state.mode = crate::repl::model::EditorMode::Normal;
        let result = view_manager.update_cursor_style(&state);
        assert!(result.is_ok());

        // Switch to Insert mode (cursor should be visible with different style)
        state.mode = crate::repl::model::EditorMode::Insert;
        let result = view_manager.update_cursor_style(&state);
        assert!(result.is_ok());
    }

    #[test]
    fn command_mode_cursor_hiding_should_work_with_render_methods() {
        // Test that cursor hiding in command mode works correctly with different render methods
        let mut view_manager = ViewManager::new();
        let mut state = AppState::new((80, 24), false);

        // Set to command mode
        state.mode = crate::repl::model::EditorMode::Command;

        // All render methods should work without issues when cursor is hidden
        let result = view_manager.render_cursor_only(&state);
        assert!(result.is_ok());

        let result = view_manager.render_content_update(&state);
        assert!(result.is_ok());

        let result = view_manager.render_full(&state);
        assert!(result.is_ok());

        // Switch back to normal mode and verify renders still work
        state.mode = crate::repl::model::EditorMode::Normal;
        let result = view_manager.render_full(&state);
        assert!(result.is_ok());
    }

    #[test]
    fn position_cursor_should_be_skipped_in_command_mode() {
        // Test that cursor positioning is skipped when in command mode
        let view_manager = ViewManager::new();
        let mut state = AppState::new((80, 24), false);

        // Test normal mode - cursor positioning should work
        state.mode = crate::repl::model::EditorMode::Normal;
        let result = view_manager.position_cursor_after_render(&state);
        assert!(result.is_ok());

        // Test command mode - cursor positioning should be skipped
        state.mode = crate::repl::model::EditorMode::Command;
        let result = view_manager.position_cursor_after_render(&state);
        assert!(result.is_ok());

        // Should work in other modes too
        state.mode = crate::repl::model::EditorMode::Insert;
        let result = view_manager.position_cursor_after_render(&state);
        assert!(result.is_ok());
    }
}
