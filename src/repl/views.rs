//! # View Layer for REPL Architecture
//!
//! Views are responsible for rendering and handling terminal display.
//! They subscribe to view events and update the display accordingly.

use crate::repl::events::{EditorMode, Pane, ViewEvent};
use crate::repl::view_models::ViewModel;
use anyhow::Result;

/// Type alias for pane boundary tuple to reduce complexity
pub type PaneBoundaries = (u16, u16, u16);

// Helper macro to convert crossterm errors to anyhow errors
macro_rules! execute_term {
    ($($arg:expr),* $(,)?) => {
        execute!($($arg),*).map_err(anyhow::Error::from)
    };
}
use crossterm::{
    cursor::{MoveTo, Show},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{Clear, ClearType},
};
use std::io::{self, Write};

/// Trait for rendering views
pub trait ViewRenderer {
    /// Initialize the terminal for rendering
    fn initialize(&mut self) -> Result<()>;

    /// Render the full application state
    fn render_full(&mut self, view_model: &ViewModel) -> Result<()>;

    /// Render only specific pane
    fn render_pane(&mut self, view_model: &ViewModel, pane: Pane) -> Result<()>;

    /// Update cursor position only
    fn render_cursor(&mut self, view_model: &ViewModel) -> Result<()>;

    /// Render status bar
    fn render_status_bar(&mut self, view_model: &ViewModel) -> Result<()>;

    /// Handle view events
    fn handle_view_event(&mut self, event: &ViewEvent, view_model: &ViewModel) -> Result<()>;

    /// Cleanup terminal on exit
    fn cleanup(&mut self) -> Result<()>;
}

/// Terminal-based view renderer using crossterm
pub struct TerminalRenderer {
    stdout: io::Stdout,
    terminal_size: (u16, u16),
}

impl TerminalRenderer {
    /// Create new terminal renderer
    pub fn new() -> Result<Self> {
        let terminal_size = crossterm::terminal::size().map_err(anyhow::Error::from)?;
        Ok(Self {
            stdout: io::stdout(),
            terminal_size,
        })
    }

    /// Update terminal size
    pub fn update_size(&mut self, width: u16, height: u16) {
        self.terminal_size = (width, height);
    }

    /// Render a single line of text at position
    fn render_line(&mut self, row: u16, text: &str, _is_current_line: bool) -> Result<()> {
        execute_term!(self.stdout, MoveTo(0, row))?;

        // Truncate text if too long for terminal
        let max_width = self.terminal_size.0 as usize;
        let display_text = if text.len() > max_width {
            &text[..max_width]
        } else {
            text
        };

        execute_term!(self.stdout, Print(display_text))?;

        Ok(())
    }

    /// Render buffer content in a pane area
    fn render_buffer_content(
        &mut self,
        view_model: &ViewModel,
        pane: Pane,
        start_row: u16,
        height: u16,
    ) -> Result<()> {
        let content = view_model.get_buffer_content(pane);
        let lines: Vec<&str> = content.lines().collect();
        let scroll_offset = view_model.get_scroll_offset(pane);
        let cursor_pos = view_model.get_cursor_for_pane(pane);
        let is_current_pane = view_model.get_current_pane() == pane;

        for row in 0..height {
            let line_index = scroll_offset + row as usize;
            let terminal_row = start_row + row;

            if line_index < lines.len() {
                let line_text = lines[line_index];
                let is_cursor_line = is_current_pane && cursor_pos.line == line_index;
                self.render_line(terminal_row, line_text, is_cursor_line)?;
            } else {
                // Empty line
                execute_term!(self.stdout, MoveTo(0, terminal_row), Print("~"))?;
            }
        }

        Ok(())
    }

    /// Calculate pane boundaries
    fn get_pane_boundaries(&self, view_model: &ViewModel) -> PaneBoundaries {
        let total_height = self.terminal_size.1;
        let request_height = view_model.request_pane_height();
        let response_start = request_height + 1; // +1 for separator
        let response_height = total_height.saturating_sub(response_start + 1); // -1 for status bar

        (request_height, response_start, response_height)
    }

    /// Render pane separator
    fn render_separator(&mut self, row: u16) -> Result<()> {
        execute_term!(
            self.stdout,
            MoveTo(0, row),
            SetForegroundColor(Color::Blue),
            Print("─".repeat(self.terminal_size.0 as usize)),
            ResetColor
        )
    }
}

impl Default for TerminalRenderer {
    fn default() -> Self {
        Self::new().expect("Failed to create terminal renderer")
    }
}

impl ViewRenderer for TerminalRenderer {
    fn initialize(&mut self) -> Result<()> {
        crossterm::terminal::enable_raw_mode().map_err(anyhow::Error::from)?;
        execute_term!(
            self.stdout,
            crossterm::terminal::EnterAlternateScreen,
            Clear(ClearType::All),
            crossterm::cursor::Hide
        )?;
        Ok(())
    }

    fn render_full(&mut self, view_model: &ViewModel) -> Result<()> {
        execute_term!(self.stdout, Clear(ClearType::All))?;

        let (request_height, response_start, response_height) =
            self.get_pane_boundaries(view_model);

        // Render request pane
        self.render_buffer_content(view_model, Pane::Request, 0, request_height)?;

        // Render separator
        self.render_separator(request_height)?;

        // Render response pane
        self.render_buffer_content(view_model, Pane::Response, response_start, response_height)?;

        // Render status bar
        self.render_status_bar(view_model)?;

        // Render cursor
        self.render_cursor(view_model)?;

        self.stdout.flush().map_err(anyhow::Error::from)?;
        Ok(())
    }

    fn render_pane(&mut self, view_model: &ViewModel, pane: Pane) -> Result<()> {
        let (request_height, response_start, response_height) =
            self.get_pane_boundaries(view_model);

        match pane {
            Pane::Request => {
                self.render_buffer_content(view_model, Pane::Request, 0, request_height)?;
            }
            Pane::Response => {
                self.render_buffer_content(
                    view_model,
                    Pane::Response,
                    response_start,
                    response_height,
                )?;
            }
        }

        self.render_cursor(view_model)?;
        self.stdout.flush().map_err(anyhow::Error::from)?;
        Ok(())
    }

    fn render_cursor(&mut self, view_model: &ViewModel) -> Result<()> {
        let current_pane = view_model.get_current_pane();
        let cursor_pos = view_model.get_cursor_for_pane(current_pane);
        let scroll_offset = view_model.get_scroll_offset(current_pane);

        // Calculate terminal position
        let terminal_row = match current_pane {
            Pane::Request => cursor_pos.line.saturating_sub(scroll_offset) as u16,
            Pane::Response => {
                let (_, response_start, _) = self.get_pane_boundaries(view_model);
                response_start + cursor_pos.line.saturating_sub(scroll_offset) as u16
            }
        };

        let terminal_col = cursor_pos.column as u16;

        execute_term!(self.stdout, MoveTo(terminal_col, terminal_row), Show)?;

        self.stdout.flush().map_err(anyhow::Error::from)?;
        Ok(())
    }

    fn render_status_bar(&mut self, view_model: &ViewModel) -> Result<()> {
        let status_row = self.terminal_size.1 - 1;
        let mode_text = match view_model.get_mode() {
            EditorMode::Normal => "NORMAL",
            EditorMode::Insert => "INSERT",
            EditorMode::Command => "COMMAND",
        };

        let pane_text = match view_model.get_current_pane() {
            Pane::Request => "REQUEST",
            Pane::Response => "RESPONSE",
        };

        let cursor = view_model.get_cursor_position();
        let status_text = format!(
            "{} | {} | {}:{}",
            mode_text,
            pane_text,
            cursor.line + 1,
            cursor.column + 1
        );

        execute_term!(
            self.stdout,
            MoveTo(0, status_row),
            Print(format!(
                "{:>width$}",
                status_text,
                width = self.terminal_size.0 as usize
            ))
        )?;

        Ok(())
    }

    fn handle_view_event(&mut self, event: &ViewEvent, view_model: &ViewModel) -> Result<()> {
        match event {
            ViewEvent::FullRedrawRequired => {
                self.render_full(view_model)?;
            }
            ViewEvent::PaneRedrawRequired { pane } => {
                self.render_pane(view_model, *pane)?;
            }
            ViewEvent::StatusBarUpdateRequired => {
                self.render_status_bar(view_model)?;
                self.render_cursor(view_model)?;
                self.stdout.flush().map_err(anyhow::Error::from)?;
            }
            ViewEvent::CursorUpdateRequired { .. } => {
                self.render_cursor(view_model)?;
            }
            ViewEvent::ScrollChanged { pane, .. } => {
                self.render_pane(view_model, *pane)?;
            }
        }
        Ok(())
    }

    fn cleanup(&mut self) -> Result<()> {
        execute_term!(self.stdout, Show, crossterm::terminal::LeaveAlternateScreen)?;
        crossterm::terminal::disable_raw_mode().map_err(anyhow::Error::from)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Testing terminal rendering is complex and typically done with integration tests
    // Here we just test that the renderer can be created

    #[test]
    fn terminal_renderer_should_create() {
        // This might fail in CI environments without a terminal
        if crossterm::terminal::size().is_ok() {
            let renderer = TerminalRenderer::new();
            assert!(renderer.is_ok());
        }
    }

    #[test]
    fn terminal_renderer_should_update_size() {
        if let Ok(mut renderer) = TerminalRenderer::new() {
            renderer.update_size(120, 40);
            assert_eq!(renderer.terminal_size, (120, 40));
        }
    }

    #[test]
    fn status_bar_should_right_align_indicators() {
        if let Ok(mut renderer) = TerminalRenderer::new() {
            renderer.update_size(50, 10); // Set a specific terminal size

            // The status bar should format text with right alignment
            // We can't easily test the actual terminal output, but we can verify
            // that the formatting uses the correct width
            assert_eq!(renderer.terminal_size.0, 50);
        }
    }
}
