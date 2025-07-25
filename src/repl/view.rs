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
    style::{Color, ResetColor, SetBackgroundColor},
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
        // Calculate cursor position in request pane and move cursor there
        let cursor_row = state.request_buffer.cursor_line - state.request_buffer.scroll_offset;
        let cursor_col = state.request_buffer.cursor_col;

        // Only move cursor if we're in the request pane
        if state.current_pane == super::model::Pane::Request {
            execute!(
                io::stdout(),
                cursor::MoveTo(cursor_col as u16, cursor_row as u16)
            )?;
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
        // Implement request pane content rendering
        // This would be similar to the current render_pane_update logic
        // for the request pane only

        let pane_height = state.get_request_pane_height();
        let (start, end) = state.request_buffer.visible_range(pane_height);

        // Move to start of request pane and render visible lines
        for (row, line_idx) in (start..end).enumerate() {
            execute!(io::stdout(), cursor::MoveTo(0, row as u16))?;

            if let Some(line) = state.request_buffer.lines.get(line_idx) {
                // TODO: Add syntax highlighting for HTTP requests
                print!("{}", line);
            }

            // Clear rest of line
            execute!(io::stdout(), Clear(ClearType::UntilNewLine))?;
        }

        Ok(())
    }

    fn render_request_pane_full(&self, state: &AppState) -> Result<()> {
        // Render with full decorations (borders, focus indicators, etc.)

        // Add focus highlighting if this pane is active
        if state.current_pane == super::model::Pane::Request {
            execute!(io::stdout(), SetBackgroundColor(Color::DarkBlue))?;
        }

        self.render_request_pane_content(state)?;

        // Reset colors
        execute!(io::stdout(), ResetColor)?;

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
        // Only render cursor if we have a response and we're in response pane
        if let Some(ref response_buffer) = state.response_buffer {
            if state.current_pane == super::model::Pane::Response {
                let cursor_row = response_buffer.cursor_line - response_buffer.scroll_offset;
                let cursor_col = response_buffer.cursor_col;

                // Offset by request pane height + separator
                let actual_row = cursor_row + state.get_request_pane_height() + 1;
                execute!(
                    io::stdout(),
                    cursor::MoveTo(cursor_col as u16, actual_row as u16)
                )?;
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
        let response_start_row = state.get_request_pane_height() + 1; // +1 for separator

        if let Some(ref response_buffer) = state.response_buffer {
            let pane_height = state.get_response_pane_height();
            let (start, end) = response_buffer.visible_range(pane_height);

            for (row, line_idx) in (start..end).enumerate() {
                let actual_row = response_start_row + row;
                execute!(io::stdout(), cursor::MoveTo(0, actual_row as u16))?;

                if let Some(line) = response_buffer.lines.get(line_idx) {
                    // TODO: Add syntax highlighting for HTTP responses
                    print!("{}", line);
                }

                execute!(io::stdout(), Clear(ClearType::UntilNewLine))?;
            }
        } else {
            // Clear response area if no response
            let pane_height = state.get_response_pane_height();
            for row in 0..pane_height {
                let actual_row = response_start_row + row;
                execute!(io::stdout(), cursor::MoveTo(0, actual_row as u16))?;
                execute!(io::stdout(), Clear(ClearType::UntilNewLine))?;
            }
        }

        Ok(())
    }

    fn render_response_pane_full(&self, state: &AppState) -> Result<()> {
        // Add focus highlighting if this pane is active
        if state.current_pane == super::model::Pane::Response {
            execute!(io::stdout(), SetBackgroundColor(Color::DarkGreen))?;
        }

        self.render_response_pane_content(state)?;

        // Reset colors
        execute!(io::stdout(), ResetColor)?;

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
    fn render_cursor_only(&mut self, _state: &AppState) -> Result<()> {
        // Status line doesn't change on cursor-only moves
        Ok(())
    }

    fn render_content_update(&mut self, _state: &AppState) -> Result<()> {
        // Status line typically doesn't change on content updates
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

        // Add current pane indicator
        let pane_indicator = match state.current_pane {
            super::model::Pane::Request => " [REQ]",
            super::model::Pane::Response => " [RES]",
        };
        print!("{}", pane_indicator);

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
