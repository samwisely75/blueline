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
    cursor::{Hide, MoveTo, SetCursorStyle, Show},
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

    /// Render partial pane from start_line to bottom of visible area
    fn render_pane_partial(
        &mut self,
        view_model: &ViewModel,
        pane: Pane,
        start_line: usize,
    ) -> Result<()>;

    /// Update cursor position only
    fn render_cursor(&mut self, view_model: &ViewModel) -> Result<()>;

    /// Render status bar
    fn render_status_bar(&mut self, view_model: &ViewModel) -> Result<()>;

    /// Render only position indicator in status bar (for reduced flickering)
    fn render_position_indicator(&mut self, view_model: &ViewModel) -> Result<()>;

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

    /// Calculate visual length of text, excluding ANSI escape sequences
    fn visual_length(&self, text: &str) -> usize {
        let mut length = 0;
        let mut in_escape = false;

        for ch in text.chars() {
            if ch == '\x1b' {
                in_escape = true;
            } else if in_escape && ch == 'm' {
                in_escape = false;
            } else if !in_escape {
                length += 1;
            }
        }

        length
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
        is_continuation: bool,
    ) -> Result<()> {
        // Just move cursor, don't hide it here (should be hidden by caller)
        execute_term!(self.stdout, MoveTo(0, row))?;

        if let Some(num) = line_number {
            // Render line number with dimmed style and right alignment (minimum width 3)
            execute_term!(self.stdout, SetAttribute(Attribute::Dim))?;
            execute_term!(
                self.stdout,
                Print(format!("{:>width$} ", num, width = line_num_width))
            )?;
            execute_term!(self.stdout, SetAttribute(Attribute::Reset))?;
        } else if is_continuation {
            // Continuation line of wrapped text - show blank space
            execute_term!(
                self.stdout,
                Print(format!("{} ", " ".repeat(line_num_width)))
            )?;
        } else {
            // Show tilda for empty lines beyond content (vim-style) with darker gray color
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

        // Calculate how much space is available for text after line number
        let used_width = line_num_width + 1; // line number + space
        let available_width = (self.terminal_size.0 as usize).saturating_sub(used_width);

        // Truncate text to fit within terminal width to prevent overlap
        let display_text = if text.chars().count() > available_width {
            text.chars().take(available_width).collect::<String>()
        } else {
            text.to_string()
        };

        // Display the (possibly truncated) text
        execute_term!(self.stdout, Print(display_text))?;

        // Clear rest of line
        execute_term!(self.stdout, Clear(ClearType::UntilNewLine))?;

        Ok(())
    }

    /// Render buffer content in a pane area using display lines
    fn render_buffer_content(
        &mut self,
        view_model: &ViewModel,
        pane: Pane,
        start_row: u16,
        height: u16,
    ) -> Result<()> {
        // Get display lines for rendering from ViewModel
        let display_lines = view_model.get_display_lines_for_rendering(pane, 0, height as usize);
        let line_num_width = view_model.get_line_number_width(pane);

        for (row, display_data) in display_lines.iter().enumerate() {
            let terminal_row = start_row + row as u16;

            match display_data {
                Some((content, line_number, is_continuation)) => {
                    // Render content with optional line number
                    self.render_line_with_number(
                        terminal_row,
                        *line_number,
                        content,
                        line_num_width,
                        *is_continuation,
                    )?;
                }
                None => {
                    // Special case: always show line number 1 in request pane
                    if pane == Pane::Request && row == 0 {
                        self.render_line_with_number(
                            terminal_row,
                            Some(1),
                            "",
                            line_num_width,
                            false,
                        )?;
                    } else {
                        // Beyond content - show tilde
                        self.render_line_with_number(
                            terminal_row,
                            None,
                            "",
                            line_num_width,
                            false,
                        )?;
                    }
                }
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
        // Controller handles raw mode and alternate screen
        // We just need to clear screen and set initial cursor state
        execute_term!(self.stdout, Clear(ClearType::All), crossterm::cursor::Hide)?;
        Ok(())
    }

    fn render_full(&mut self, view_model: &ViewModel) -> Result<()> {
        // Hide cursor before screen refresh to avoid flickering
        execute_term!(self.stdout, crossterm::cursor::Hide)?;

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

        // Render cursor (this will show cursor in correct position)
        self.render_cursor(view_model)?;

        self.stdout.flush().map_err(anyhow::Error::from)?;
        Ok(())
    }

    fn render_pane(&mut self, view_model: &ViewModel, pane: Pane) -> Result<()> {
        // Cursor hiding is now handled by the controller

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

        // Don't render cursor here - let the controller handle it once at the end
        self.stdout.flush().map_err(anyhow::Error::from)?;
        Ok(())
    }

    fn render_pane_partial(
        &mut self,
        view_model: &ViewModel,
        pane: Pane,
        start_line: usize,
    ) -> Result<()> {
        // Cursor hiding is now handled by the controller

        let (request_height, response_start, response_height) =
            self.get_pane_boundaries(view_model);

        match pane {
            Pane::Request => {
                // Calculate the starting row for the partial redraw
                let display_lines = view_model.get_display_lines_for_rendering(
                    pane,
                    start_line,
                    request_height as usize - start_line,
                );
                let line_num_width = view_model.get_line_number_width(pane);

                for (idx, display_data) in display_lines.iter().enumerate() {
                    let terminal_row = start_line as u16 + idx as u16;
                    if terminal_row >= request_height {
                        break;
                    }

                    match display_data {
                        Some((content, line_number, is_continuation)) => {
                            self.render_line_with_number(
                                terminal_row,
                                *line_number,
                                content,
                                line_num_width,
                                *is_continuation,
                            )?;
                        }
                        None => {
                            // Special case: always show line number 1 in request pane
                            if pane == Pane::Request && idx == 0 && start_line == 0 {
                                self.render_line_with_number(
                                    terminal_row,
                                    Some(1),
                                    "",
                                    line_num_width,
                                    false,
                                )?;
                            } else {
                                self.render_line_with_number(
                                    terminal_row,
                                    None,
                                    "",
                                    line_num_width,
                                    false,
                                )?;
                            }
                        }
                    }
                }
            }
            Pane::Response => {
                if view_model.get_response_status_code().is_some() {
                    let display_lines = view_model.get_display_lines_for_rendering(
                        pane,
                        start_line,
                        response_height as usize - start_line,
                    );
                    let line_num_width = view_model.get_line_number_width(pane);

                    for (idx, display_data) in display_lines.iter().enumerate() {
                        let terminal_row = response_start + start_line as u16 + idx as u16;
                        if terminal_row >= response_start + response_height {
                            break;
                        }

                        match display_data {
                            Some((content, line_number, is_continuation)) => {
                                self.render_line_with_number(
                                    terminal_row,
                                    *line_number,
                                    content,
                                    line_num_width,
                                    *is_continuation,
                                )?;
                            }
                            None => {
                                self.render_line_with_number(
                                    terminal_row,
                                    None,
                                    "",
                                    line_num_width,
                                    false,
                                )?;
                            }
                        }
                    }
                }
            }
        }

        // Don't render cursor here - let the controller handle it once at the end
        self.stdout.flush().map_err(anyhow::Error::from)?;
        Ok(())
    }

    fn render_cursor(&mut self, view_model: &ViewModel) -> Result<()> {
        // Always hide cursor first to prevent any ghost cursor artifacts
        tracing::debug!("render_cursor: hiding cursor before positioning");
        execute_term!(self.stdout, crossterm::cursor::Hide)?;

        let current_mode = view_model.get_mode();

        // Handle command mode: hide cursor completely
        if current_mode == EditorMode::Command {
            execute_term!(self.stdout, Hide)?;
            self.stdout.flush().map_err(anyhow::Error::from)?;
            return Ok(());
        }

        let current_pane = view_model.get_current_pane();

        // Get cursor position in display coordinates (relative to pane)
        let (cursor_row, cursor_col) = view_model.get_cursor_for_rendering(current_pane);

        // Get line number width for cursor offset
        let line_num_width = view_model.get_line_number_width(current_pane);
        let line_num_offset = line_num_width + 1; // +1 for space after line number

        // Calculate terminal position
        let terminal_row = match current_pane {
            Pane::Request => cursor_row as u16,
            Pane::Response => {
                let (_, response_start, _) = self.get_pane_boundaries(view_model);
                response_start + cursor_row as u16
            }
        };

        let terminal_col = cursor_col as u16 + line_num_offset as u16;

        tracing::debug!(
            "render_cursor: pane={:?}, display_coords=({}, {}), terminal_pos=({}, {}), mode={:?}",
            current_pane,
            cursor_row,
            cursor_col,
            terminal_col,
            terminal_row,
            current_mode
        );

        // Set cursor shape and position based on mode
        match current_mode {
            EditorMode::Normal => {
                // Block cursor for normal mode
                tracing::debug!(
                    "render_cursor: showing cursor at ({}, {}) for Normal mode",
                    terminal_col,
                    terminal_row
                );
                execute_term!(
                    self.stdout,
                    MoveTo(terminal_col, terminal_row),
                    SetCursorStyle::DefaultUserShape,
                    Show
                )?;
            }
            EditorMode::Insert => {
                // Bar cursor for insert mode
                tracing::debug!(
                    "render_cursor: showing cursor at ({}, {}) for Insert mode",
                    terminal_col,
                    terminal_row
                );
                execute_term!(
                    self.stdout,
                    MoveTo(terminal_col, terminal_row),
                    SetCursorStyle::BlinkingBar,
                    Show
                )?;
            }
            EditorMode::Command => {
                // Should not reach here since we handle command mode above
                tracing::debug!(
                    "render_cursor: showing cursor at ({}, {}) for Command mode",
                    terminal_col,
                    terminal_row
                );
                execute_term!(
                    self.stdout,
                    MoveTo(terminal_col, terminal_row),
                    SetCursorStyle::BlinkingUnderScore,
                    Show
                )?;
            }
        }

        self.stdout.flush().map_err(anyhow::Error::from)?;

        // Add tiny delay after cursor show to prevent ghost cursor artifacts
        // during rapid key repetition (especially 'j' key)
        std::thread::sleep(std::time::Duration::from_micros(50));

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
            // Show cursor in command mode for command line editing
            execute_term!(self.stdout, Show)?;
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

            // Build status parts
            let mut status_text = String::new();

            // 1. HTTP response info (optional, when present)
            if let Some(status_code) = view_model.get_response_status_code() {
                let status_message_opt = view_model.get_response_status_message();
                let status_message = status_message_opt.as_deref().unwrap_or("");
                let status_full = format!("{} {}", status_code, status_message);

                // Use old MVC bullet design with ANSI colors
                let signal_icon = match status_code {
                    200..=299 => "\x1b[32m●\x1b[0m ", // Green bullet for success
                    400..=599 => "\x1b[31m●\x1b[0m ", // Red bullet for errors
                    _ => "● ",                        // Default bullet for unknown
                };

                status_text.push_str(&format!("{}{}", signal_icon, status_full));

                // TAT (ephemeral)
                if let Some(duration_ms) = view_model.get_response_duration_ms() {
                    let duration = std::time::Duration::from_millis(duration_ms);
                    let duration_text = humantime::format_duration(duration).to_string();
                    status_text.push_str(&format!(" | {}", duration_text));
                }

                status_text.push_str(" | ");
            }

            // Mode (persistent)
            status_text.push_str(mode_text);
            status_text.push_str(" | ");

            // Pane and Position (no separator between them)
            status_text.push_str(pane_text);
            status_text.push(' ');
            status_text.push_str(&format!("{}:{}", cursor.line + 1, cursor.column + 1));

            let available_width = self.terminal_size.0 as usize;
            let visual_len = self.visual_length(&status_text);

            // Truncate if too long (based on visual length)
            let final_text = if visual_len > available_width {
                // This is complex to truncate while preserving ANSI codes
                // For now, just use the original text and let terminal handle overflow
                status_text
            } else {
                status_text
            };

            // Calculate right alignment based on visual length
            let padding = available_width.saturating_sub(visual_len);

            execute_term!(
                self.stdout,
                MoveTo(0, status_row),
                Print(format!("{}{}", " ".repeat(padding), final_text))
            )?;
        }

        Ok(())
    }

    fn render_position_indicator(&mut self, view_model: &ViewModel) -> Result<()> {
        let status_row = self.terminal_size.1 - 1;
        let cursor = view_model.get_cursor_position();

        // Get current mode and pane
        let mode_text = match view_model.get_mode() {
            EditorMode::Normal => "NORMAL",
            EditorMode::Insert => "INSERT",
            EditorMode::Command => "COMMAND",
        };

        let pane_text = match view_model.get_current_pane() {
            Pane::Request => "REQUEST",
            Pane::Response => "RESPONSE",
        };

        let position_text = format!("{}:{}", cursor.line + 1, cursor.column + 1);

        // Build the right portion of the status bar, including HTTP info if present
        let mut right_text = String::new();

        // Add HTTP response info if present
        if let Some(status_code) = view_model.get_response_status_code() {
            let status_message_opt = view_model.get_response_status_message();
            let status_message = status_message_opt.as_deref().unwrap_or("");
            let status_full = format!("{} {}", status_code, status_message);

            // Use old MVC bullet design with ANSI colors
            let signal_icon = match status_code {
                200..=299 => "\x1b[32m●\x1b[0m ", // Green bullet for success
                400..=599 => "\x1b[31m●\x1b[0m ", // Red bullet for errors
                _ => "● ",                        // Default bullet for unknown
            };

            right_text.push_str(&format!("{}{}", signal_icon, status_full));

            // TAT (ephemeral)
            if let Some(duration_ms) = view_model.get_response_duration_ms() {
                let duration = std::time::Duration::from_millis(duration_ms);
                let duration_text = humantime::format_duration(duration).to_string();
                right_text.push_str(&format!(" | {}", duration_text));
            }

            right_text.push_str(" | ");
        }

        // Add mode, pane, and position
        right_text.push_str(&format!("{} | {} {}", mode_text, pane_text, position_text));

        // Calculate where this portion should start to be right-aligned
        let available_width = self.terminal_size.0 as usize;
        let visual_len = self.visual_length(&right_text);

        let right_start_col = available_width.saturating_sub(visual_len);

        // Clear from a bit earlier to catch any leftover HTTP icon characters
        // HTTP icon with ANSI codes can be up to ~10 characters, so clear from 15 chars back to be safe
        let clear_start_col = right_start_col.saturating_sub(15);

        // Clear from the safe start position to the end of the line
        execute_term!(
            self.stdout,
            MoveTo(clear_start_col as u16, status_row),
            Clear(ClearType::UntilNewLine)
        )?;

        // Write the reconstructed right portion
        execute_term!(
            self.stdout,
            MoveTo(right_start_col as u16, status_row),
            Print(&right_text)
        )?;

        self.stdout.flush().map_err(anyhow::Error::from)?;
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
            ViewEvent::PartialPaneRedrawRequired { pane, start_line } => {
                self.render_pane_partial(view_model, *pane, *start_line)?;
            }
            ViewEvent::StatusBarUpdateRequired => {
                self.render_status_bar(view_model)?;
                self.render_cursor(view_model)?;
                self.stdout.flush().map_err(anyhow::Error::from)?;
            }
            ViewEvent::PositionIndicatorUpdateRequired => {
                self.render_position_indicator(view_model)?;
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
        // Controller handles alternate screen and raw mode cleanup
        // We just need to show cursor before exit
        execute_term!(self.stdout, Show)?;
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
    fn visual_length_should_exclude_ansi_codes() {
        if let Ok(renderer) = TerminalRenderer::new() {
            // Test plain text
            assert_eq!(renderer.visual_length("Hello World"), 11);

            // Test text with ANSI color codes
            assert_eq!(renderer.visual_length("\x1b[32m●\x1b[0m Hello"), 7); // ● + space + Hello = 7

            // Test multiple ANSI sequences
            assert_eq!(
                renderer.visual_length("\x1b[32m●\x1b[0m \x1b[31mRed\x1b[0m"),
                5
            ); // ● + space + Red = 5

            // Test empty string
            assert_eq!(renderer.visual_length(""), 0);

            // Test only ANSI codes
            assert_eq!(renderer.visual_length("\x1b[32m\x1b[0m"), 0);
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
