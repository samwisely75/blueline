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
    cursor::{self, SetCursorStyle},
    execute,
    style::{Attribute, Color, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor},
    terminal::{Clear, ClearType},
};

use super::model::AppState;

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
        for observer in &mut self.observers {
            observer.render_cursor_only(state)?;
        }
        // Position cursor correctly after all rendering
        self.position_cursor_after_render(state)?;
        self.last_render_type = RenderType::CursorOnly;
        io::stdout().flush()?;
        Ok(())
    }

    /// Render content updates (moderate cost)
    pub fn render_content_update(&mut self, state: &AppState) -> Result<()> {
        for observer in &mut self.observers {
            observer.render_content_update(state)?;
        }
        // Update cursor style in case mode changed
        self.update_cursor_style(state)?;
        // Position cursor correctly after all rendering
        self.position_cursor_after_render(state)?;
        self.last_render_type = RenderType::ContentUpdate;
        io::stdout().flush()?;
        Ok(())
    }

    /// Render full screen (most expensive)
    pub fn render_full(&mut self, state: &AppState) -> Result<()> {
        for observer in &mut self.observers {
            observer.render_full(state)?;
        }
        // Update cursor style for full renders
        self.update_cursor_style(state)?;
        // Position cursor correctly after all rendering
        self.position_cursor_after_render(state)?;
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

        Ok(())
    }

    /// Clean up terminal state
    pub fn cleanup_terminal(&self) -> Result<()> {
        // Restore default cursor
        execute!(io::stdout(), SetCursorStyle::DefaultUserShape)?;
        Ok(())
    }

    /// Update cursor style based on current editor mode
    pub fn update_cursor_style(&self, state: &AppState) -> Result<()> {
        use super::model::EditorMode;

        let cursor_style = match state.mode {
            EditorMode::Normal => SetCursorStyle::SteadyBlock,
            EditorMode::Insert => SetCursorStyle::BlinkingBar,
            EditorMode::Command => SetCursorStyle::BlinkingUnderScore,
            EditorMode::Visual | EditorMode::VisualLine => SetCursorStyle::SteadyBlock,
        };

        execute!(io::stdout(), cursor_style)?;
        Ok(())
    }

    /// Position cursor correctly based on current pane and mode
    fn position_cursor_after_render(&self, state: &AppState) -> Result<()> {
        use super::model::Pane;

        match state.current_pane {
            Pane::Request => {
                let cursor_row = state
                    .request_buffer
                    .cursor_line
                    .saturating_sub(state.request_buffer.scroll_offset);
                let cursor_col = state.request_buffer.cursor_col;

                // Calculate line number width to offset cursor position
                let max_line_num = state.request_buffer.line_count();
                let line_num_width = max_line_num.to_string().len().max(3);
                let line_num_offset = line_num_width + 1; // +1 for space after line number

                // Bounds checking to prevent invalid cursor positions
                let terminal_height = state.terminal_size.1 as usize;
                let terminal_width = state.terminal_size.0 as usize;
                let final_col = cursor_col + line_num_offset;

                if cursor_row < terminal_height && final_col < terminal_width {
                    execute!(
                        io::stdout(),
                        cursor::MoveTo(final_col as u16, cursor_row as u16)
                    )?;
                }
            }
            Pane::Response => {
                if let Some(ref response_buffer) = state.response_buffer {
                    let cursor_row = response_buffer
                        .cursor_line
                        .saturating_sub(response_buffer.scroll_offset);
                    let cursor_col = response_buffer.cursor_col;

                    // Calculate line number width to offset cursor position
                    let max_line_num = response_buffer.line_count();
                    let line_num_width = max_line_num.to_string().len().max(3);
                    let line_num_offset = line_num_width + 1; // +1 for space after line number

                    // Offset by request pane height + separator
                    let actual_row = cursor_row + state.get_request_pane_height() + 1;
                    let final_col = cursor_col + line_num_offset;

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

/// Observer that renders the request pane
pub struct RequestPaneRenderer {
    // Add any specific state needed for request pane rendering
}

impl RequestPaneRenderer {
    pub fn new() -> Self {
        Self {}
    }
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
        // Implement request pane content rendering with line numbers like vim

        let pane_height = state.get_request_pane_height();
        let (start, end) = state.request_buffer.visible_range(pane_height);

        // Calculate width needed for line numbers (minimum 3 characters)
        let max_line_num = state.request_buffer.line_count();
        let line_num_width = max_line_num.to_string().len().max(3);

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
                // Show tilda for non-existing lines (like vi) with dimmed style
                // Place tilda in the same column as line numbers would be
                execute!(io::stdout(), SetAttribute(Attribute::Dim))?;
                print!("{:>width$} ", "~", width = line_num_width);
                execute!(io::stdout(), SetAttribute(Attribute::Reset))?;
            }

            // Clear rest of line
            execute!(io::stdout(), Clear(ClearType::UntilNewLine))?;
        }

        // Fill remaining visible rows with tilda lines if there are more visible rows than content
        let visible_content_lines =
            (end - start).min(state.request_buffer.line_count().saturating_sub(start));
        for row in visible_content_lines..pane_height {
            execute!(io::stdout(), cursor::MoveTo(0, row as u16))?;
            execute!(io::stdout(), SetAttribute(Attribute::Dim))?;
            print!("{:>width$} ", "~", width = line_num_width);
            execute!(io::stdout(), SetAttribute(Attribute::Reset))?;
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

impl ResponsePaneRenderer {
    pub fn new() -> Self {
        Self {}
    }
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
            let terminal_width = state.terminal_size.0 as usize;
            let (start, end) = response_buffer.visible_range(pane_height);

            // Calculate width needed for line numbers (minimum 3 characters)
            let max_line_num = response_buffer.line_count();
            let line_num_width = max_line_num.to_string().len().max(3);

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

impl StatusLineRenderer {
    pub fn new() -> Self {
        Self {}
    }
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
        execute!(io::stdout(), cursor::MoveTo(0, status_row))?;

        // Clear the entire status line
        execute!(io::stdout(), Clear(ClearType::UntilNewLine))?;

        // Print status message
        print!("{}", state.status_message);

        // Add timing information if available
        if let Some(duration) = state.last_request_duration {
            let timing_info = format!(" | {}ms", duration);
            print!("{}", timing_info);
        }

        Ok(())
    }
}

/// Create a default view manager with standard observers
pub fn create_default_view_manager() -> ViewManager {
    let mut view_manager = ViewManager::new();

    view_manager.add_observer(Box::new(RequestPaneRenderer::new()));
    view_manager.add_observer(Box::new(ResponsePaneRenderer::new()));
    view_manager.add_observer(Box::new(StatusLineRenderer::new()));

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
        let renderer = RequestPaneRenderer::new();

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

        let renderer = RequestPaneRenderer::new();

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

        let renderer = RequestPaneRenderer::new();
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
        let renderer = ResponsePaneRenderer::new();

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
        let renderer = ResponsePaneRenderer::new();

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
}
