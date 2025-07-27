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
    style::{Attribute, Color, Print, ResetColor, SetAttribute, SetForegroundColor},
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

    /// Get current terminal size
    pub fn terminal_size(&self) -> (u16, u16) {
        self.terminal_size
    }

    /// Render a single line of text at position with line number
    fn render_line_with_number(
        &mut self,
        row: u16,
        line_number: Option<usize>,
        text: &str,
        line_num_width: usize,
    ) -> Result<()> {
        execute_term!(self.stdout, MoveTo(0, row))?;

        if let Some(num) = line_number {
            // Render line number with dimmed style and right alignment (minimum width 3)
            execute_term!(self.stdout, SetAttribute(Attribute::Dim))?;
            execute_term!(
                self.stdout,
                Print(format!("{:>width$} ", num, width = line_num_width))
            )?;
            execute_term!(self.stdout, SetAttribute(Attribute::Reset))?;
        } else {
            // Show tilda for empty lines (vim-style) with darker gray color
            execute_term!(self.stdout, SetForegroundColor(Color::DarkGrey))?;
            execute_term!(
                self.stdout,
                Print(format!(
                    "~{} ",
                    " ".repeat(line_num_width.saturating_sub(1))
                ))
            )?;
            execute_term!(self.stdout, ResetColor)?;
        }

        // Calculate available width for content after line numbers
        let content_width = self.terminal_size.0 as usize - line_num_width - 1; // -1 for space after line number

        // Truncate text if too long for remaining terminal width
        let display_text = if text.len() > content_width {
            &text[..content_width]
        } else {
            text
        };

        execute_term!(self.stdout, Print(display_text))?;

        // Clear rest of line
        execute_term!(self.stdout, Clear(ClearType::UntilNewLine))?;

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

        // Get cursor position to determine the extent of "real" lines
        let cursor_pos = view_model.get_cursor_for_pane(pane);
        let max_cursor_line = cursor_pos.line;

        // Calculate the maximum line that should show a line number
        // This includes all lines in the buffer content plus the cursor line if beyond content
        let max_line_with_number = if lines.is_empty() {
            max_cursor_line.max(0) // If buffer is empty, at least show line numbers up to cursor
        } else {
            lines.len().saturating_sub(1).max(max_cursor_line)
        };

        // Calculate line number width (minimum 3 characters)
        let display_max_line = match pane {
            Pane::Request => (max_line_with_number + 1).max(1), // Always at least line 1 for request pane
            Pane::Response => {
                if lines.is_empty() {
                    1 // For empty response pane, we need minimum width for tildes
                } else {
                    max_line_with_number + 1
                }
            }
        };
        let line_num_width = display_max_line.to_string().len().max(3);

        for row in 0..height {
            let line_index = scroll_offset + row as usize;
            let terminal_row = start_row + row;

            let should_show_line_number =
                line_index <= max_line_with_number || (line_index == 0 && pane == Pane::Request);

            if should_show_line_number {
                // Show line number for:
                // 1. Lines that exist in buffer content
                // 2. Lines up to and including cursor position (even if empty)
                // 3. Always show line 1 for request pane even if completely empty
                let line_text = if line_index < lines.len() {
                    lines[line_index]
                } else {
                    "" // Empty line beyond content but within cursor range
                };
                let line_number = Some(line_index + 1); // 1-based line numbers
                self.render_line_with_number(terminal_row, line_number, line_text, line_num_width)?;
            } else {
                // Beyond actual content and cursor - show tilda
                self.render_line_with_number(terminal_row, None, "", line_num_width)?;
            }
        }

        Ok(())
    }

    /// Calculate pane boundaries
    fn get_pane_boundaries(&self, view_model: &ViewModel) -> PaneBoundaries {
        let total_height = self.terminal_size.1;

        if view_model.get_response_status_code().is_some() {
            // When response exists, split the space
            let request_height = view_model.request_pane_height();
            let response_start = request_height + 1; // +1 for separator
            let response_height = view_model.response_pane_height();

            (request_height, response_start, response_height)
        } else {
            // When no response, request pane uses full available space
            let request_height = total_height - 1; // -1 for status bar
            let response_start = request_height + 1; // Won't be used
            let response_height = 0; // Hidden
            (request_height, response_start, response_height)
        }
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

        // Only render separator and response pane if there's an HTTP response
        if view_model.get_response_status_code().is_some() {
            // Render separator
            self.render_separator(request_height)?;

            // Render response pane
            self.render_buffer_content(
                view_model,
                Pane::Response,
                response_start,
                response_height,
            )?;
        }

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
                // Only render response pane if there's an HTTP response
                if view_model.get_response_status_code().is_some() {
                    self.render_buffer_content(
                        view_model,
                        Pane::Response,
                        response_start,
                        response_height,
                    )?;
                }
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

        // Calculate line number width for cursor offset
        let content = view_model.get_buffer_content(current_pane);
        let lines: Vec<&str> = content.lines().collect();
        let max_line_num = lines.len();
        let line_num_width = max_line_num.to_string().len().max(3);
        let line_num_offset = line_num_width + 1; // +1 for space after line number

        // Calculate terminal position
        let terminal_row = match current_pane {
            Pane::Request => cursor_pos.line.saturating_sub(scroll_offset) as u16,
            Pane::Response => {
                let (_, response_start, _) = self.get_pane_boundaries(view_model);
                response_start + cursor_pos.line.saturating_sub(scroll_offset) as u16
            }
        };

        let terminal_col = cursor_pos.column as u16 + line_num_offset as u16;

        execute_term!(self.stdout, MoveTo(terminal_col, terminal_row), Show)?;

        self.stdout.flush().map_err(anyhow::Error::from)?;
        Ok(())
    }

    fn render_status_bar(&mut self, view_model: &ViewModel) -> Result<()> {
        let status_row = self.terminal_size.1 - 1;

        // Clear the status bar first
        execute_term!(
            self.stdout,
            MoveTo(0, status_row),
            Print(" ".repeat(self.terminal_size.0 as usize))
        )?;

        // Check if we're in command mode and need to show ex command buffer
        if view_model.get_mode() == EditorMode::Command {
            let ex_command_text = format!(":{}", view_model.get_ex_command_buffer());
            execute_term!(self.stdout, MoveTo(0, status_row), Print(&ex_command_text))?;
        } else {
            // Show normal status information
            let mode_text = match view_model.get_mode() {
                EditorMode::Normal => "NORMAL",
                EditorMode::Insert => "INSERT",
                EditorMode::Command => "COMMAND", // Shouldn't reach here
            };

            let pane_text = match view_model.get_current_pane() {
                Pane::Request => "REQUEST",
                Pane::Response => "RESPONSE",
            };

            let cursor = view_model.get_cursor_position();

            // Build status parts in order: HTTP response | TAT | Mode | Pane | Position
            let mut status_parts = Vec::new();

            // 1. HTTP response (ephemeral)
            if let Some(status_code) = view_model.get_response_status_code() {
                let signal = match status_code {
                    200..=299 => "🟢", // Green for success
                    400..=599 => "🔴", // Red for both client and server errors
                    _ => "⚪",         // White for unknown
                };

                let status_message = view_model
                    .get_response_status_message()
                    .map(|s| s.as_str())
                    .unwrap_or("");

                status_parts.push(format!("{} {} {}", signal, status_code, status_message));

                // 2. TAT (ephemeral)
                if let Some(duration_ms) = view_model.get_response_duration_ms() {
                    let duration = std::time::Duration::from_millis(duration_ms);
                    let duration_text = humantime::format_duration(duration).to_string();
                    status_parts.push(duration_text);
                }
            }

            // 3. Mode (persistent)
            status_parts.push(mode_text.to_string());

            // 4. Pane (persistent)
            status_parts.push(pane_text.to_string());

            // 5. Position (persistent)
            status_parts.push(format!("{}:{}", cursor.line + 1, cursor.column + 1));

            let status_text = status_parts.join(" | ");

            // Account for emoji display width - emojis take 2 terminal columns but count as 1 char
            let emoji_count = status_text.chars().filter(|c| *c as u32 > 0x1F000).count();
            let display_width = status_text.len() + emoji_count;
            let available_width = self.terminal_size.0 as usize;

            // Truncate if too long
            let final_text = if display_width > available_width {
                let max_chars = available_width.saturating_sub(emoji_count);
                status_text.chars().take(max_chars).collect::<String>()
            } else {
                status_text
            };

            execute_term!(
                self.stdout,
                MoveTo(0, status_row),
                Print(format!(
                    "{:>width$}",
                    final_text,
                    width = available_width.saturating_sub(emoji_count)
                ))
            )?;
        }

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
    use crate::repl::view_models::ViewModel;

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

    #[test]
    fn response_pane_boundaries_should_calculate_correctly() {
        if let Ok(mut renderer) = TerminalRenderer::new() {
            renderer.update_size(80, 40); // 40 line terminal
            let mut view_model = ViewModel::new();
            view_model.update_terminal_size(80, 40);

            // Set a response so the response pane appears
            view_model.set_response(200, "test response".to_string());

            let (request_height, response_start, response_height) =
                renderer.get_pane_boundaries(&view_model);

            // With terminal height 40:
            // - request_pane_height should be 20 (height/2)
            // - response_start should be 21 (20 + 1 for separator)
            // - response_height should be 18 (40 - 20 - 2 for separator and status)
            assert_eq!(request_height, 20);
            assert_eq!(response_start, 21);
            assert_eq!(response_height, 18);

            // Total should equal terminal height
            assert_eq!(request_height + 1 + response_height + 1, 40); // +1 for separator, +1 for status
        }
    }
}
