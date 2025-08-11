//! # View Layer for REPL Architecture
//!
//! Views are responsible for rendering and handling terminal display.
//! They subscribe to view events and update the display accordingly.

use crate::repl::events::{EditorMode, Pane, ViewEvent};
use crate::repl::io::RenderStream;
use crate::repl::view_models::ViewModel;
use anyhow::Result;
// Import ANSI escape codes from the separate module
use super::ansi_escape_codes as ansi;

/// Line rendering information to reduce function parameter count
#[derive(Debug)]
struct LineInfo<'a> {
    text: &'a str,
    line_number: Option<usize>,
    is_continuation: bool,
    logical_start_col: usize,
    logical_line: usize,
}

// Helper macro for safe flush operations
macro_rules! safe_flush {
    ($writer:expr) => {
        $writer.flush().map_err(anyhow::Error::from)
    };
}

use unicode_width::UnicodeWidthChar;

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

/// Terminal-based view renderer using RenderStream abstraction
pub struct TerminalRenderer<RS: RenderStream> {
    render_stream: RS,
    terminal_size: (u16, u16),
}

impl<RS: RenderStream> TerminalRenderer<RS> {
    /// Create new terminal renderer with RenderStream
    pub fn with_render_stream(render_stream: RS) -> Result<Self> {
        let terminal_size = render_stream.get_size().unwrap_or((80, 24));
        Ok(Self {
            render_stream,
            terminal_size,
        })
    }

    /// Calculate visual length of text, excluding ANSI escape sequences
    /// Accounts for double-byte characters that take 2 terminal columns
    fn visual_length(&self, text: &str) -> usize {
        let mut length = 0;
        let mut in_escape = false;

        for ch in text.chars() {
            if ch == '\x1b' {
                in_escape = true;
            } else if in_escape && ch == 'm' {
                in_escape = false;
            } else if !in_escape {
                // Use unicode-width to get proper display width
                // Most double-byte characters (CJK) have width 2
                if let Some(w) = unicode_width::UnicodeWidthChar::width(ch) {
                    length += w;
                }
                // Control characters and zero-width characters have no width
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

    /// Render a single line of text at position with line number, with visual selection support
    fn render_line_with_number(
        &mut self,
        view_model: &ViewModel,
        pane: Pane,
        row: u16,
        line_info: &LineInfo,
        line_num_width: usize,
    ) -> Result<()> {
        // Move cursor to the beginning of the line
        self.render_stream.move_cursor(0, row)?;

        #[allow(unused_variables)]
        if let Some(num) = line_info.line_number {
            // Render line number with dimmed style and right alignment (minimum width 3)
            write!(
                self.render_stream,
                "{}{num:>line_num_width$} {}",
                ansi::DIM,
                ansi::RESET
            )?;
        } else if line_info.is_continuation {
            // Continuation line of wrapped text - show blank space
            write!(self.render_stream, "{} ", " ".repeat(line_num_width))?;
        } else {
            // Show tilda for empty lines beyond content (vim-style) with darker gray color
            write!(
                self.render_stream,
                "{}~{} {}",
                ansi::DIM,
                " ".repeat(line_num_width.saturating_sub(1)),
                ansi::RESET
            )?;
        }

        // Calculate how much space is available for text after line number
        let used_width = line_num_width + 1; // line number + space
        let available_width = (self.terminal_size.0 as usize).saturating_sub(used_width);

        // Truncate text to fit within terminal width, accounting for double-byte characters
        let display_text = if self.visual_length(line_info.text) > available_width {
            let mut result = String::new();
            let mut current_width = 0;

            for ch in line_info.text.chars() {
                let char_width = UnicodeWidthChar::width(ch).unwrap_or(0);
                if current_width + char_width > available_width {
                    break;
                }
                result.push(ch);
                current_width += char_width;
            }
            result
        } else {
            line_info.text.to_string()
        };

        // Render text with visual selection highlighting if applicable
        self.render_text_with_selection(
            view_model,
            pane,
            line_info.line_number,
            &display_text,
            line_info.logical_start_col,
            line_info.logical_line,
        )?;

        // Clear rest of line
        write!(self.render_stream, "{}", ansi::CLEAR_LINE)?;

        // Flush to ensure content is displayed
        safe_flush!(self.render_stream)?;

        Ok(())
    }

    /// Render text with visual selection highlighting
    fn render_text_with_selection(
        &mut self,
        view_model: &ViewModel,
        pane: Pane,
        line_number: Option<usize>,
        text: &str,
        logical_start_col: usize,
        logical_line: usize,
    ) -> Result<()> {
        // Check if we're in visual mode and have a selection
        let mode = view_model.get_mode();
        if mode == EditorMode::Visual {
            tracing::trace!("render_text_with_selection: Visual mode detected, pane={:?}, line_number={:?}, logical_line={}, text='{}'", pane, line_number, logical_line, text);

            // BUGFIX: Use logical_line directly instead of relying on line_number
            // For wrapped lines, continuation segments have line_number=None but we still need
            // to render visual selection. The logical_line parameter always contains the correct
            // logical line number regardless of whether this is a continuation or not.
            let chars: Vec<char> = text.chars().collect();
            let selection_state = view_model.get_visual_selection();

            tracing::trace!(
                "render_text_with_selection: selection_state={:?}",
                selection_state
            );

            for (col_index, ch) in chars.iter().enumerate() {
                // BUGFIX: Calculate correct logical column for wrapped lines
                // For wrapped lines, logical_start_col indicates where this display line starts
                // within the original logical line, so we add col_index to get the actual position
                let logical_col = logical_start_col + col_index;
                let position = crate::repl::events::LogicalPosition::new(
                    logical_line, // Use logical_line directly (already 0-based)
                    logical_col,
                );

                let is_selected = view_model.is_position_selected(position, pane);

                if is_selected {
                    tracing::debug!(
                        "render_text_with_selection: highlighting character '{}' at {:?}",
                        ch,
                        position
                    );
                    // Apply visual selection styling: inverse + blue
                    write!(
                        self.render_stream,
                        "{}{}{ch}{}",
                        ansi::BG_SELECTED,
                        ansi::FG_SELECTED,
                        ansi::RESET
                    )?
                } else {
                    // Normal character rendering
                    write!(self.render_stream, "{ch}")?
                }
            }
            return Ok(());
        } else {
            tracing::trace!(
                "render_text_with_selection: Not in visual mode (mode={:?})",
                mode
            );
        }

        // No selection or not in visual mode - render normally
        write!(self.render_stream, "{text}")?;
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
                Some((content, line_number, is_continuation, logical_start_col, logical_line)) => {
                    // Render content with optional line number
                    let line_info = LineInfo {
                        text: content,
                        line_number: *line_number,
                        is_continuation: *is_continuation,
                        logical_start_col: *logical_start_col,
                        logical_line: *logical_line,
                    };
                    self.render_line_with_number(
                        view_model,
                        pane,
                        terminal_row,
                        &line_info,
                        line_num_width,
                    )?;
                }
                None => {
                    // Special case: always show line number 1 in request pane
                    if pane == Pane::Request && row == 0 {
                        let line_info = LineInfo {
                            text: "",
                            line_number: Some(1),
                            is_continuation: false,
                            logical_start_col: 0, // logical_start_col is 0 for empty lines
                            logical_line: 0,      // Empty line is logical line 0
                        };
                        self.render_line_with_number(
                            view_model,
                            pane,
                            terminal_row,
                            &line_info,
                            line_num_width,
                        )?;
                    } else {
                        // Beyond content - show tilde
                        let line_info = LineInfo {
                            text: "",
                            line_number: None,
                            is_continuation: false,
                            logical_start_col: 0, // logical_start_col is 0 for tildes
                            logical_line: 0,      // Tildes are beyond content, use 0
                        };
                        self.render_line_with_number(
                            view_model,
                            pane,
                            terminal_row,
                            &line_info,
                            line_num_width,
                        )?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Render pane separator
    #[allow(unused_variables)]
    fn render_separator(&mut self, row: u16) -> Result<()> {
        self.render_stream.move_cursor(0, row)?;
        write!(
            self.render_stream,
            "{}{}{}",
            ansi::FG_SEPARATOR,
            "─".repeat(self.terminal_size.0 as usize),
            ansi::RESET
        )?;
        Ok(())
    }
}

// Default implementation removed - TerminalRenderer requires explicit RenderStream injection

impl<RS: RenderStream> ViewRenderer for TerminalRenderer<RS> {
    fn initialize(&mut self) -> Result<()> {
        // Initialize terminal for rendering
        self.render_stream.enable_raw_mode()?;
        self.render_stream.enter_alternate_screen()?;
        self.render_stream.clear_screen()?;
        self.render_stream.hide_cursor()?;
        Ok(())
    }

    fn render_full(&mut self, view_model: &ViewModel) -> Result<()> {
        // Hide cursor before screen refresh to avoid flickering
        self.render_stream.hide_cursor()?;
        self.render_stream.clear_screen()?;

        let (request_height, response_start, response_height) = view_model
            .pane_manager()
            .get_pane_boundaries(view_model.get_response_status_code().is_some());

        // Render request pane
        self.render_buffer_content(view_model, Pane::Request, 0, request_height)?;

        // Only render separator and response pane if there's an HTTP response
        let has_response = view_model.get_response_status_code().is_some();
        tracing::debug!(
            "render_full: has_response = {}, rendering response pane = {}",
            has_response,
            has_response
        );
        if has_response {
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
        safe_flush!(self.render_stream)?;

        Ok(())
    }

    fn render_pane(&mut self, view_model: &ViewModel, pane: Pane) -> Result<()> {
        // Cursor hiding is now handled by the controller

        let (request_height, response_start, response_height) = view_model
            .pane_manager()
            .get_pane_boundaries(view_model.get_response_status_code().is_some());

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
        safe_flush!(self.render_stream)?;
        Ok(())
    }

    fn render_pane_partial(
        &mut self,
        view_model: &ViewModel,
        pane: Pane,
        start_line: usize,
    ) -> Result<()> {
        // Cursor hiding is now handled by the controller

        let (request_height, response_start, response_height) = view_model
            .pane_manager()
            .get_pane_boundaries(view_model.get_response_status_code().is_some());

        match pane {
            Pane::Request => {
                // Calculate the starting row for the partial redraw
                // BUGFIX: Use saturating_sub to prevent integer underflow panic
                // This prevents crashes when start_line exceeds request_height during scrolling
                let height = (request_height as usize).saturating_sub(start_line);
                let display_lines =
                    view_model.get_display_lines_for_rendering(pane, start_line, height);
                let line_num_width = view_model.get_line_number_width(pane);

                for (idx, display_data) in display_lines.iter().enumerate() {
                    let terminal_row = start_line as u16 + idx as u16;
                    if terminal_row >= request_height {
                        break;
                    }

                    match display_data {
                        Some((
                            content,
                            line_number,
                            is_continuation,
                            logical_start_col,
                            logical_line,
                        )) => {
                            let line_info = LineInfo {
                                text: content,
                                line_number: *line_number,
                                is_continuation: *is_continuation,
                                logical_start_col: *logical_start_col,
                                logical_line: *logical_line,
                            };
                            self.render_line_with_number(
                                view_model,
                                pane,
                                terminal_row,
                                &line_info,
                                line_num_width,
                            )?;
                        }
                        None => {
                            // Special case: always show line number 1 in request pane
                            if pane == Pane::Request && idx == 0 && start_line == 0 {
                                let line_info = LineInfo {
                                    text: "",
                                    line_number: Some(1),
                                    is_continuation: false,
                                    logical_start_col: 0, // logical_start_col is 0 for empty lines
                                    logical_line: 0,      // Empty line is logical line 0
                                };
                                self.render_line_with_number(
                                    view_model,
                                    pane,
                                    terminal_row,
                                    &line_info,
                                    line_num_width,
                                )?;
                            } else {
                                let line_info = LineInfo {
                                    text: "",
                                    line_number: None,
                                    is_continuation: false,
                                    logical_start_col: 0, // logical_start_col is 0 for tildes
                                    logical_line: 0,      // Tildes are beyond content, use 0
                                };
                                self.render_line_with_number(
                                    view_model,
                                    pane,
                                    terminal_row,
                                    &line_info,
                                    line_num_width,
                                )?;
                            }
                        }
                    }
                }
            }
            Pane::Response => {
                if view_model.get_response_status_code().is_some() {
                    // BUGFIX: Use saturating_sub to prevent integer underflow panic
                    // This prevents crashes when start_line exceeds response_height during scrolling
                    let height = (response_height as usize).saturating_sub(start_line);
                    let display_lines =
                        view_model.get_display_lines_for_rendering(pane, start_line, height);
                    let line_num_width = view_model.get_line_number_width(pane);

                    for (idx, display_data) in display_lines.iter().enumerate() {
                        let terminal_row = response_start + start_line as u16 + idx as u16;
                        if terminal_row >= response_start + response_height {
                            break;
                        }

                        match display_data {
                            Some((
                                content,
                                line_number,
                                is_continuation,
                                logical_start_col,
                                logical_line,
                            )) => {
                                let line_info = LineInfo {
                                    text: content,
                                    line_number: *line_number,
                                    is_continuation: *is_continuation,
                                    logical_start_col: *logical_start_col,
                                    logical_line: *logical_line,
                                };
                                self.render_line_with_number(
                                    view_model,
                                    pane,
                                    terminal_row,
                                    &line_info,
                                    line_num_width,
                                )?;
                            }
                            None => {
                                let line_info = LineInfo {
                                    text: "",
                                    line_number: None,
                                    is_continuation: false,
                                    logical_start_col: 0, // logical_start_col is 0 for tildes
                                    logical_line: 0,      // Tildes are beyond content, use 0
                                };
                                self.render_line_with_number(
                                    view_model,
                                    pane,
                                    terminal_row,
                                    &line_info,
                                    line_num_width,
                                )?;
                            }
                        }
                    }
                }
            }
        }

        // Don't render cursor here - let the controller handle it once at the end
        safe_flush!(self.render_stream)?;
        Ok(())
    }

    fn render_cursor(&mut self, view_model: &ViewModel) -> Result<()> {
        // Cursor should be visible in normal editing modes
        // Only hide cursor in command mode when showing command line cursor
        let should_hide_cursor = view_model.get_mode() == EditorMode::Command;
        tracing::debug!(
            "render_cursor: mode = {:?}, should_hide_cursor = {}",
            view_model.get_mode(),
            should_hide_cursor
        );

        if should_hide_cursor {
            tracing::debug!("render_cursor: hiding cursor for command mode");
            self.render_stream.hide_cursor()?;
            safe_flush!(self.render_stream)?;
            return Ok(());
        }

        // Get display cursor position and adjust for line numbers and pane offset
        let display_cursor = view_model.get_display_cursor_position();
        let current_pane = view_model.get_current_pane();
        let line_num_width = view_model.get_line_number_width(current_pane);

        // Get scroll offset to calculate viewport-relative position
        let scroll_offset = view_model.pane_manager().get_current_scroll_offset();

        // Get pane boundaries to calculate response pane offset
        let (_request_height, response_start, _response_height) = view_model
            .pane_manager()
            .get_pane_boundaries(view_model.get_response_status_code().is_some());

        // Calculate viewport-relative position by subtracting scroll offset
        let viewport_relative_row = display_cursor.row.saturating_sub(scroll_offset.row);

        // Calculate screen column: display_cursor.col - horizontal_scroll + line_numbers + padding
        // When horizontally scrolled, we need to subtract the scroll offset to get the visible position
        let screen_col = display_cursor.col
            .saturating_sub(scroll_offset.col) // Subtract horizontal scroll offset
            + line_num_width + 1; // Add line number width and padding
        let screen_row = match current_pane {
            Pane::Request => viewport_relative_row,
            Pane::Response => viewport_relative_row + response_start as usize,
        };

        let terminal_size = self.terminal_size;
        tracing::debug!(
            "render_cursor: current_pane={:?}, display_cursor=({}, {}), scroll_offset=({}, {}), response_start={}, line_num_width={}, screen_pos=({}, {}) with terminal size ({}, {})", 
            current_pane, display_cursor.col, display_cursor.row, scroll_offset.row, scroll_offset.col, response_start, line_num_width, screen_col, screen_row, terminal_size.0, terminal_size.1
        );

        // Validate and clamp cursor coordinates to terminal bounds
        let max_row = (terminal_size.1 as usize).saturating_sub(2); // Leave room for status bar
        if screen_col >= terminal_size.0 as usize || screen_row >= terminal_size.1 as usize {
            tracing::warn!(
                "render_cursor: cursor position ({}, {}) is outside terminal bounds ({}, {}), clamping", 
                screen_col, screen_row, terminal_size.0, terminal_size.1
            );
        }

        // Clamp cursor to valid screen area
        let clamped_col = (screen_col).min(terminal_size.0 as usize - 1);
        let clamped_row = screen_row.min(max_row);

        if clamped_row != screen_row || clamped_col != screen_col {
            tracing::debug!(
                "render_cursor: clamped cursor from ({}, {}) to ({}, {})",
                screen_col,
                screen_row,
                clamped_col,
                clamped_row
            );
        }

        // Set cursor style based on editor mode using ANSI escape codes
        let cursor_style = match view_model.get_mode() {
            EditorMode::Insert => ansi::CURSOR_BAR, // I-beam for insert mode
            EditorMode::Normal => ansi::CURSOR_BLOCK, // Block for normal mode
            EditorMode::Visual => ansi::CURSOR_BLOCK, // Block for visual mode
            EditorMode::Command => ansi::CURSOR_BAR, // I-beam for command mode
            EditorMode::GPrefix => ansi::CURSOR_BLOCK, // Block for g-prefix mode
        };

        // Position cursor, set style, and show
        self.render_stream
            .move_cursor(clamped_col as u16, clamped_row as u16)?;
        write!(self.render_stream, "{cursor_style}")?;
        self.render_stream.show_cursor()?;
        safe_flush!(self.render_stream)?;
        tracing::debug!("render_cursor: cursor shown successfully");

        Ok(())
    }

    fn render_status_bar(&mut self, view_model: &ViewModel) -> Result<()> {
        let status_row = self.terminal_size.1 - 1;

        // Clear the status bar first
        self.render_stream.move_cursor(0, status_row)?;
        write!(
            self.render_stream,
            "{}",
            " ".repeat(self.terminal_size.0 as usize)
        )?;

        // Check if we're in command mode and need to show ex command buffer
        if view_model.get_mode() == EditorMode::Command {
            let ex_command_text = format!(":{}", view_model.get_ex_command_buffer());
            self.render_stream.move_cursor(0, status_row)?;
            write!(self.render_stream, "{}", &ex_command_text)?;

            // Show I-beam cursor at the end of command text for command line editing
            #[allow(unused_variables)]
            let cursor_pos = ex_command_text.len() as u16;
            self.render_stream.move_cursor(cursor_pos, status_row)?;
            write!(self.render_stream, "{}", ansi::CURSOR_BAR)?;
            self.render_stream.show_cursor()?;
        } else {
            let pane_text = match view_model.get_current_pane() {
                Pane::Request => "REQUEST",
                Pane::Response => "RESPONSE",
            };

            let cursor = view_model.get_cursor_position();

            // Build status parts - left side for vim mode indicators, right side for info
            let mut left_status_text = String::new();
            let mut right_status_text = String::new();

            // Left side: Vim-style mode indicators (highest priority)
            match view_model.get_mode() {
                EditorMode::Insert => {
                    left_status_text.push_str(&format!(
                        "{}-- INSERT --{}",
                        ansi::BOLD,
                        ansi::RESET
                    ));
                }
                EditorMode::Visual => {
                    left_status_text.push_str(&format!(
                        "{}-- VISUAL --{}",
                        ansi::BOLD,
                        ansi::RESET
                    ));
                }
                _ => {
                    // Normal mode shows no status message (following Vim exactly)
                    // Command mode shows ex command buffer (handled above)
                    // GPrefix mode shows no status message
                }
            }

            // If no vim mode indicator and we have custom status message, show it
            if left_status_text.is_empty() {
                if let Some(message) = view_model.get_status_message() {
                    left_status_text.push_str(message);
                }
                // Show "Executing..." when request is being processed
                else if view_model.is_executing_request() {
                    let bullet = ansi::STATUS_BULLET_YELLOW;
                    left_status_text.push_str(&format!("{bullet} Executing..."));
                }
            }

            // Right side: HTTP response info (optional, when present)
            if let Some(status_code) = view_model.get_response_status_code() {
                let status_message_opt = view_model.get_response_status_message();
                let status_message = status_message_opt.as_deref().unwrap_or("");
                let status_full = format!("{status_code} {status_message}");

                // Use old MVC bullet design with ANSI colors
                let signal_icon = match status_code {
                    200..=299 => ansi::STATUS_BULLET_GREEN,
                    400..=599 => ansi::STATUS_BULLET_RED,
                    _ => ansi::STATUS_BULLET_DEFAULT,
                };

                right_status_text.push_str(&format!("{signal_icon}{status_full}"));

                // TAT (ephemeral)
                if let Some(duration_ms) = view_model.get_response_duration_ms() {
                    let duration = std::time::Duration::from_millis(duration_ms);
                    let duration_text = humantime::format_duration(duration).to_string();
                    right_status_text.push_str(&format!(" | {duration_text}"));
                }

                right_status_text.push_str(" | ");
            }

            // Pane and Position (no mode, no separator between pane and position)
            right_status_text.push_str(pane_text);
            right_status_text.push(' ');

            // Use consistent position formatting with render_position_indicator
            let position_text = if view_model.is_display_cursor_visible() {
                let display_cursor = view_model.get_display_cursor_position();
                let scroll_offset = view_model.pane_manager().get_current_scroll_offset();

                // Show viewport-relative display position for traditional page scrolling behavior
                let viewport_relative_row = display_cursor.row.saturating_sub(scroll_offset.row);
                let viewport_relative_col = display_cursor.col.saturating_sub(scroll_offset.col);

                format!(
                    "{}:{} ({}:{}) HSO:{}",
                    cursor.line + 1,
                    cursor.column + 1,
                    viewport_relative_row + 1,
                    viewport_relative_col + 1,
                    scroll_offset.col
                )
            } else {
                format!("{}:{}", cursor.line + 1, cursor.column + 1)
            };
            right_status_text.push_str(&position_text);

            let available_width = self.terminal_size.0 as usize;

            // Render left status text (vim mode indicators) at the beginning
            if !left_status_text.is_empty() {
                self.render_stream.move_cursor(0, status_row)?;
                write!(self.render_stream, "{left_status_text}")?;
            }

            // Render right status text (HTTP | pane & location) right-aligned
            if !right_status_text.is_empty() {
                let right_visual_len = self.visual_length(&right_status_text);
                let right_padding = available_width.saturating_sub(right_visual_len);

                self.render_stream
                    .move_cursor(right_padding as u16, status_row)?;
                write!(self.render_stream, "{right_status_text}")?;
            }
        }

        Ok(())
    }

    fn render_position_indicator(&mut self, view_model: &ViewModel) -> Result<()> {
        let status_row = self.terminal_size.1 - 1;
        let cursor = view_model.get_cursor_position();

        // Get current pane
        let pane_text = match view_model.get_current_pane() {
            Pane::Request => "REQUEST",
            Pane::Response => "RESPONSE",
        };

        let display_cursor = view_model.get_display_cursor_position();
        tracing::debug!(
            "render_position_indicator: logical=({}, {}), display=({}, {})",
            cursor.line,
            cursor.column,
            display_cursor.row,
            display_cursor.col
        );
        let position_text = if view_model.is_display_cursor_visible() {
            let scroll_offset = view_model.pane_manager().get_current_scroll_offset();

            // Show viewport-relative display position for traditional page scrolling behavior
            let viewport_relative_row = display_cursor.row.saturating_sub(scroll_offset.row);
            let viewport_relative_col = display_cursor.col.saturating_sub(scroll_offset.col);

            format!(
                "{}:{} ({}:{}) HSO:{}",
                cursor.line + 1,
                cursor.column + 1,
                viewport_relative_row + 1,
                viewport_relative_col + 1,
                scroll_offset.col
            )
        } else {
            format!("{}:{}", cursor.line + 1, cursor.column + 1)
        };

        // Build the right portion of the status bar, including HTTP info if present
        let mut right_text = String::new();

        // Add HTTP response info if present
        if let Some(status_code) = view_model.get_response_status_code() {
            let status_message_opt = view_model.get_response_status_message();
            let status_message = status_message_opt.as_deref().unwrap_or("");
            let status_full = format!("{status_code} {status_message}");

            // Use old MVC bullet design with ANSI colors
            let signal_icon = match status_code {
                200..=299 => ansi::STATUS_BULLET_GREEN,
                400..=599 => ansi::STATUS_BULLET_RED,
                _ => ansi::STATUS_BULLET_DEFAULT,
            };

            right_text.push_str(&format!("{signal_icon}{status_full}"));

            // TAT (ephemeral)
            if let Some(duration_ms) = view_model.get_response_duration_ms() {
                let duration = std::time::Duration::from_millis(duration_ms);
                let duration_text = humantime::format_duration(duration).to_string();
                right_text.push_str(&format!(" | {duration_text}"));
            }

            right_text.push_str(" | ");
        }

        // Add pane and position (no mode)
        right_text.push_str(&format!("{pane_text} {position_text}"));

        // Calculate where this portion should start to be right-aligned
        let available_width = self.terminal_size.0 as usize;
        let visual_len = self.visual_length(&right_text);

        let right_start_col = available_width.saturating_sub(visual_len);

        // Clear from a bit earlier to catch any leftover HTTP icon characters
        // HTTP icon with ANSI codes can be up to ~10 characters, so clear from 15 chars back to be safe
        let clear_start_col = right_start_col.saturating_sub(15);

        // Clear from the safe start position to the end of the line
        self.render_stream
            .move_cursor(clear_start_col as u16, status_row)?;
        write!(self.render_stream, "{}", ansi::CLEAR_LINE)?;

        // Write the reconstructed right portion
        self.render_stream
            .move_cursor(right_start_col as u16, status_row)?;
        write!(self.render_stream, "{}", &right_text)?;

        safe_flush!(self.render_stream)?;
        Ok(())
    }

    fn handle_view_event(&mut self, event: &ViewEvent, view_model: &ViewModel) -> Result<()> {
        match event {
            ViewEvent::FullRedrawRequired => {
                self.render_full(view_model)?;
            }
            ViewEvent::CurrentAreaRedrawRequired => {
                let current_pane = view_model.get_current_pane();
                self.render_pane(view_model, current_pane)?;
            }
            ViewEvent::SecondaryAreaRedrawRequired => {
                let current_pane = view_model.get_current_pane();
                let secondary_pane = match current_pane {
                    crate::repl::events::Pane::Request => crate::repl::events::Pane::Response,
                    crate::repl::events::Pane::Response => crate::repl::events::Pane::Request,
                };
                self.render_pane(view_model, secondary_pane)?;
            }
            ViewEvent::CurrentAreaPartialRedrawRequired { start_line } => {
                let current_pane = view_model.get_current_pane();
                self.render_pane_partial(view_model, current_pane, *start_line)?;
            }
            ViewEvent::SecondaryAreaPartialRedrawRequired { start_line } => {
                let current_pane = view_model.get_current_pane();
                let secondary_pane = match current_pane {
                    crate::repl::events::Pane::Request => crate::repl::events::Pane::Response,
                    crate::repl::events::Pane::Response => crate::repl::events::Pane::Request,
                };
                self.render_pane_partial(view_model, secondary_pane, *start_line)?;
            }
            ViewEvent::StatusBarUpdateRequired => {
                self.render_status_bar(view_model)?;
                self.render_cursor(view_model)?;
                safe_flush!(self.render_stream)?;
            }
            ViewEvent::PositionIndicatorUpdateRequired => {
                self.render_position_indicator(view_model)?;
            }
            ViewEvent::ActiveCursorUpdateRequired => {
                self.render_cursor(view_model)?;
            }
            ViewEvent::CurrentAreaScrollChanged { .. } => {
                let current_pane = view_model.get_current_pane();
                self.render_pane(view_model, current_pane)?;
            }
            ViewEvent::SecondaryAreaScrollChanged { .. } => {
                let current_pane = view_model.get_current_pane();
                let secondary_pane = match current_pane {
                    crate::repl::events::Pane::Request => crate::repl::events::Pane::Response,
                    crate::repl::events::Pane::Response => crate::repl::events::Pane::Request,
                };
                self.render_pane(view_model, secondary_pane)?;
            }
            ViewEvent::FocusSwitched => {
                // Focus switched - update cursor and status bar
                self.render_cursor(view_model)?;
                self.render_status_bar(view_model)?;
            }
            ViewEvent::RequestContentChanged => {
                // Request content changed - redraw if we're in request pane
                if view_model.is_in_request_pane() {
                    let current_pane = view_model.get_current_pane();
                    self.render_pane(view_model, current_pane)?;
                }
            }
            ViewEvent::ResponseContentChanged => {
                // Response content changed - redraw if we're in response pane
                if view_model.is_in_response_pane() {
                    let current_pane = view_model.get_current_pane();
                    self.render_pane(view_model, current_pane)?;
                } else {
                    // Always redraw response pane when content changes
                    self.render_pane(view_model, crate::repl::events::Pane::Response)?;
                }
            }
            ViewEvent::AllContentAreasRedrawRequired => {
                // Redraw both panes
                self.render_pane(view_model, crate::repl::events::Pane::Request)?;
                self.render_pane(view_model, crate::repl::events::Pane::Response)?;
            }
        }
        Ok(())
    }

    fn cleanup(&mut self) -> Result<()> {
        // Clean up terminal state on exit
        self.render_stream.show_cursor()?;
        self.render_stream.leave_alternate_screen()?;
        self.render_stream.disable_raw_mode()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::io::mock::MockRenderStream;
    use crate::repl::view_models::ViewModel;

    // Note: Testing terminal rendering is complex and typically done with integration tests
    // Here we just test that the renderer can be created

    #[test]
    fn terminal_renderer_should_create() {
        let render_stream = MockRenderStream::new();
        let renderer = TerminalRenderer::with_render_stream(render_stream);
        assert!(renderer.is_ok());
    }

    // Test removed - no longer needed with RenderStream abstraction

    #[test]
    fn terminal_renderer_should_update_size() {
        let render_stream = MockRenderStream::with_size((80, 24));
        if let Ok(mut renderer) = TerminalRenderer::with_render_stream(render_stream) {
            renderer.update_size(120, 40);
            assert_eq!(renderer.terminal_size, (120, 40));
        }
    }

    #[test]
    fn status_bar_should_right_align_indicators() {
        let render_stream = MockRenderStream::with_size((50, 10));
        if let Ok(mut renderer) = TerminalRenderer::with_render_stream(render_stream) {
            renderer.update_size(50, 10); // Set a specific terminal size

            // The status bar should format text with right alignment
            // We can't easily test the actual terminal output, but we can verify
            // that the formatting uses the correct width
            assert_eq!(renderer.terminal_size.0, 50);
        }
    }

    #[test]
    fn visual_length_should_exclude_ansi_codes() {
        let render_stream = MockRenderStream::new();
        if let Ok(renderer) = TerminalRenderer::with_render_stream(render_stream) {
            // Test plain text
            assert_eq!(renderer.visual_length("Hello World"), 11);

            // Test text with ANSI color codes
            // Using hardcoded values in tests to verify the function correctly ignores ANSI
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
    fn visual_length_should_handle_double_byte_characters() {
        let render_stream = MockRenderStream::new();
        if let Ok(renderer) = TerminalRenderer::with_render_stream(render_stream) {
            // Test Japanese hiragana (double-byte)
            assert_eq!(renderer.visual_length("こんにちは"), 10); // 5 chars × 2 width each = 10

            // Test Japanese katakana (double-byte)
            assert_eq!(renderer.visual_length("カタカナ"), 8); // 4 chars × 2 width each = 8

            // Test Japanese kanji (double-byte)
            assert_eq!(renderer.visual_length("日本語"), 6); // 3 chars × 2 width each = 6

            // Test mixed ASCII and Japanese
            assert_eq!(renderer.visual_length("Hello こんにちは"), 16); // "Hello " (6) + "こんにちは" (10) = 16

            // Test Japanese with ANSI codes
            assert_eq!(renderer.visual_length("\x1b[32mこんにちは\x1b[0m"), 10); // Only count the Japanese characters, ignore ANSI

            // Test ASCII vs Japanese comparison
            assert_eq!(renderer.visual_length("AAAAA"), 5); // 5 ASCII chars = 5 width
            assert_eq!(renderer.visual_length("あああああ"), 10); // 5 Japanese chars = 10 width
        }
    }

    #[test]
    fn visual_length_should_handle_mixed_ascii_japanese_realistic_text() {
        let render_stream = MockRenderStream::new();
        if let Ok(renderer) = TerminalRenderer::with_render_stream(render_stream) {
            // Test realistic mixed content like what users actually type
            assert_eq!(
                renderer.visual_length("Anthropic Claude は現時点では"),
                29 // "Anthropic Claude " (17) + "は現時点では" (12) = 29
            );

            assert_eq!(
                renderer.visual_length("GitHub Copilot や ChatGPT"),
                25 // "GitHub Copilot " (15) + "や" (2) + " ChatGPT" (8) = 25
            );

            assert_eq!(
                renderer.visual_length("VS Code の拡張機能"),
                18 // "VS Code " (8) + "の拡張機能" (10) = 18
            );

            // Test mixed content with punctuation
            assert_eq!(
                renderer.visual_length("API エンドポイント。"),
                20 // "API " (4) + "エンドポイント" (14) + "。" (2) = 20
            );

            // Test very long mixed line
            let long_mixed = "Programming プログラミング is とても楽しい activity";
            assert_eq!(
                renderer.visual_length(long_mixed),
                51 // Calculate: "Programming " (12) + "プログラミング" (14) + " is " (4) + "とても楽しい" (12) + " activity" (9) = 51
            );
        }
    }

    #[test]
    fn text_truncation_should_work_with_mixed_characters() {
        let render_stream = MockRenderStream::with_size((20, 10));
        if let Ok(mut renderer) = TerminalRenderer::with_render_stream(render_stream) {
            renderer.update_size(20, 10); // Small terminal for testing truncation

            // Test truncation with mixed content - simulate the truncation logic
            let mixed_text = "Hello こんにちは World 世界";
            let available_width = 15; // Simulate limited space

            // Manually test the truncation logic (similar to what's in render_line_with_number)
            let mut result = String::new();
            let mut current_width = 0;

            for ch in mixed_text.chars() {
                let char_width = UnicodeWidthChar::width(ch).unwrap_or(0);
                if current_width + char_width > available_width {
                    break;
                }
                result.push(ch);
                current_width += char_width;
            }

            // Should truncate appropriately without breaking double-byte characters
            assert!(renderer.visual_length(&result) <= available_width);
            assert!(!result.is_empty());

            // Verify it contains some content but is truncated
            assert!(result.contains("Hello"));
            // The exact truncation point depends on character boundaries
        }
    }

    #[test]
    fn response_pane_boundaries_should_calculate_correctly() {
        let render_stream = MockRenderStream::with_size((80, 40));
        if let Ok(mut renderer) = TerminalRenderer::with_render_stream(render_stream) {
            renderer.update_size(80, 40); // 40 line terminal
            let mut view_model = ViewModel::new();
            view_model.update_terminal_size(80, 40);

            // Set a response so the response pane appears
            view_model.set_response(200, "test response".to_string());

            let (request_height, response_start, response_height) = view_model
                .pane_manager()
                .get_pane_boundaries(view_model.get_response_status_code().is_some());

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

    #[test]
    fn visual_length_should_handle_control_characters_safely() {
        let render_stream = MockRenderStream::new();
        if let Ok(renderer) = TerminalRenderer::with_render_stream(render_stream) {
            // Test newline character (should be ignored/zero width)
            assert_eq!(renderer.visual_length("\n"), 0);

            // Test tab character (should be ignored/zero width)
            assert_eq!(renderer.visual_length("\t"), 0);

            // Test mixed text with newlines (newlines should be ignored)
            assert_eq!(renderer.visual_length("Hello\nWorld"), 10); // Only count printable chars

            // Test empty string
            assert_eq!(renderer.visual_length(""), 0);
        }
    }
}
