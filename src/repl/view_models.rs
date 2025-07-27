//! # ViewModel - Business Logic Coordinator
//!
//! The ViewModel contains all business logic and coordinates between models.
//! It's the central component that commands delegate to and that emits events.

use crate::repl::events::{EditorMode, EventBus, LogicalPosition, ModelEvent, Pane, ViewEvent};
use crate::repl::http::create_default_profile;
use crate::repl::models::{
    build_display_cache, BufferModel, DisplayCache, DisplayPosition, EditorModel, ResponseModel,
};
use anyhow::Result;
use bluenote::{HttpClient, HttpConnectionProfile};
use std::collections::HashMap;

/// Type alias for event bus option to reduce complexity
type EventBusOption = Option<Box<dyn EventBus>>;

/// Type alias for display line rendering data: (content, line_number, is_continuation)
pub type DisplayLineData = (String, Option<usize>, bool);

/// The central ViewModel that coordinates all business logic
pub struct ViewModel {
    // Core models
    editor: EditorModel,
    request_buffer: BufferModel,
    response_buffer: BufferModel,
    response: ResponseModel,

    // Display coordination (word wrap support)
    request_display_cache: DisplayCache,
    response_display_cache: DisplayCache,
    wrap_enabled: bool,
    request_display_cursor: DisplayPosition,
    response_display_cursor: DisplayPosition,
    request_display_scroll_offset: usize,
    response_display_scroll_offset: usize,

    // Display state
    terminal_width: u16,
    terminal_height: u16,
    request_pane_height: u16,

    // Ex command mode state (for :q, :w, etc.)
    ex_command_buffer: String,

    // HTTP client and configuration
    http_client: Option<HttpClient>,
    session_headers: HashMap<String, String>,
    verbose: bool,

    // Event bus for communication
    event_bus: EventBusOption,
}

impl ViewModel {
    /// Create new ViewModel with default state
    pub fn new() -> Self {
        // Try to create HTTP client with default profile
        let profile = create_default_profile();
        let http_client = HttpClient::new(&profile).ok();

        // Initialize display caches
        let mut request_display_cache = DisplayCache::new();
        let mut response_display_cache = DisplayCache::new();

        // Build initial caches with empty content
        let empty_lines: Vec<String> = vec![String::new()];
        if let Ok(cache) = build_display_cache(&empty_lines, 80, false) {
            request_display_cache = cache;
        }
        if let Ok(cache) = build_display_cache(&empty_lines, 80, false) {
            response_display_cache = cache;
        }

        let mut instance = Self {
            editor: EditorModel::new(),
            request_buffer: BufferModel::new(Pane::Request),
            response_buffer: BufferModel::new(Pane::Response),
            response: ResponseModel::new(),
            request_display_cache,
            response_display_cache,
            wrap_enabled: false,
            request_display_cursor: (0, 0),
            response_display_cursor: (0, 0),
            request_display_scroll_offset: 0,
            response_display_scroll_offset: 0,
            terminal_width: 80,
            terminal_height: 24,
            request_pane_height: 12,
            ex_command_buffer: String::new(),
            http_client,
            session_headers: HashMap::new(),
            verbose: false,
            event_bus: None,
        };

        // Ensure display cursors are synced with logical cursors at startup
        instance.sync_display_cursors();

        instance
    }

    /// Set the event bus for communication
    pub fn set_event_bus(&mut self, event_bus: Box<dyn EventBus>) {
        self.event_bus = Some(event_bus);
    }

    // =================================================================
    // Public API - Commands delegate to these methods
    // =================================================================

    /// Get current editor mode
    pub fn get_mode(&self) -> EditorMode {
        self.editor.mode()
    }

    /// Get current active pane
    pub fn get_current_pane(&self) -> Pane {
        self.editor.current_pane()
    }

    /// Get cursor position for current pane
    pub fn get_cursor_position(&self) -> LogicalPosition {
        match self.editor.current_pane() {
            Pane::Request => self.request_buffer.cursor(),
            Pane::Response => self.response_buffer.cursor(),
        }
    }

    /// Move cursor left in current pane (display coordinate based)
    pub fn move_cursor_left(&mut self) -> Result<()> {
        let current_pane = self.editor.current_pane();

        // Sync display cursor with current logical cursor position
        self.sync_display_cursor_with_logical(current_pane)?;

        let current_display_pos = self.get_display_cursor(current_pane);
        let display_cache = self.get_display_cache(current_pane);

        tracing::debug!(
            "move_cursor_left: pane={:?}, current_pos={:?}",
            current_pane,
            current_display_pos
        );

        // Check if we can move left within current display line
        if current_display_pos.1 > 0 {
            let new_display_pos = (current_display_pos.0, current_display_pos.1 - 1);
            tracing::debug!(
                "move_cursor_left: moving within line to {:?}",
                new_display_pos
            );
            self.set_display_cursor(current_pane, new_display_pos)?;
            self.emit_view_event(ViewEvent::CursorUpdateRequired { pane: current_pane });
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
                self.emit_view_event(ViewEvent::CursorUpdateRequired { pane: current_pane });
            }
        } else {
            tracing::debug!("move_cursor_left: already at beginning, no movement");
        }

        Ok(())
    }

    /// Move cursor right in current pane (display coordinate based)
    pub fn move_cursor_right(&mut self) -> Result<()> {
        let current_pane = self.editor.current_pane();

        // Sync display cursor with current logical cursor position
        self.sync_display_cursor_with_logical(current_pane)?;

        let current_display_pos = self.get_display_cursor(current_pane);
        let display_cache = self.get_display_cache(current_pane);

        tracing::debug!(
            "move_cursor_right: pane={:?}, current_pos={:?}",
            current_pane,
            current_display_pos
        );

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
                self.emit_view_event(ViewEvent::CursorUpdateRequired { pane: current_pane });
            } else if current_display_pos.0 + 1 < display_cache.display_line_count() {
                // Move to beginning of next display line
                let new_display_pos = (current_display_pos.0 + 1, 0);
                tracing::debug!(
                    "move_cursor_right: moving to next line {:?}",
                    new_display_pos
                );
                self.set_display_cursor(current_pane, new_display_pos)?;
                self.emit_view_event(ViewEvent::CursorUpdateRequired { pane: current_pane });
            } else {
                tracing::debug!("move_cursor_right: already at end, no movement");
            }
        } else {
            tracing::debug!("move_cursor_right: invalid display line, no movement");
        }

        Ok(())
    }

    /// Move cursor up in current pane (display line based)
    pub fn move_cursor_up(&mut self) -> Result<()> {
        let current_pane = self.editor.current_pane();
        let current_display_pos = self.get_display_cursor(current_pane);
        let display_cache = self.get_display_cache(current_pane);

        if let Some(new_display_pos) =
            display_cache.move_up(current_display_pos.0, current_display_pos.1)
        {
            self.set_display_cursor(current_pane, new_display_pos)?;
            self.emit_view_event(ViewEvent::CursorUpdateRequired { pane: current_pane });
        }

        Ok(())
    }

    /// Move cursor down in current pane (display line based)
    pub fn move_cursor_down(&mut self) -> Result<()> {
        let current_pane = self.editor.current_pane();
        let current_display_pos = self.get_display_cursor(current_pane);
        let display_cache = self.get_display_cache(current_pane);

        tracing::debug!(
            "move_cursor_down: pane={:?}, current_pos={:?}, cache_lines={}, cache_valid={}",
            current_pane,
            current_display_pos,
            display_cache.display_lines.len(),
            display_cache.is_valid
        );

        if let Some(new_display_pos) =
            display_cache.move_down(current_display_pos.0, current_display_pos.1)
        {
            tracing::debug!(
                "move_cursor_down: move_down succeeded, new_pos={:?}",
                new_display_pos
            );

            self.set_display_cursor(current_pane, new_display_pos)?;

            let final_display_pos = self.get_display_cursor(current_pane);
            let logical_pos = self.get_cursor_position();
            tracing::debug!(
                "move_cursor_down: after set - display={:?}, logical=({}, {})",
                final_display_pos,
                logical_pos.line,
                logical_pos.column
            );

            self.emit_view_event(ViewEvent::CursorUpdateRequired { pane: current_pane });
        } else {
            tracing::debug!("move_cursor_down: move_down FAILED - no new position available");
        }

        Ok(())
    }

    /// Move cursor to end of current display line
    pub fn move_cursor_to_end_of_line(&mut self) -> Result<()> {
        let current_pane = self.editor.current_pane();
        let current_display_pos = self.get_display_cursor(current_pane);
        let display_cache = self.get_display_cache(current_pane);

        if let Some(display_line) = display_cache.get_display_line(current_display_pos.0) {
            let line_length = display_line.content.chars().count();
            let new_display_pos = (current_display_pos.0, line_length);
            self.set_display_cursor(current_pane, new_display_pos)?;
            self.emit_view_event(ViewEvent::CursorUpdateRequired { pane: current_pane });
        }

        Ok(())
    }

    /// Move cursor to start of current display line
    pub fn move_cursor_to_start_of_line(&mut self) -> Result<()> {
        let current_pane = self.editor.current_pane();
        let current_display_pos = self.get_display_cursor(current_pane);
        let new_display_pos = (current_display_pos.0, 0);

        self.set_display_cursor(current_pane, new_display_pos)?;
        self.emit_view_event(ViewEvent::CursorUpdateRequired { pane: current_pane });

        Ok(())
    }

    /// Set cursor position in current pane
    pub fn set_cursor_position(&mut self, position: LogicalPosition) -> Result<()> {
        let current_pane = self.editor.current_pane();
        let buffer = match current_pane {
            Pane::Request => &mut self.request_buffer,
            Pane::Response => &mut self.response_buffer,
        };

        if let Some(event) = buffer.set_cursor(position) {
            self.emit_model_event(event);

            // Synchronize display cursor with the new logical position
            let display_cache = self.get_display_cache(current_pane);
            if let Some(display_pos) =
                display_cache.logical_to_display_position(position.line, position.column)
            {
                match current_pane {
                    Pane::Request => self.request_display_cursor = display_pos,
                    Pane::Response => self.response_display_cursor = display_pos,
                }

                // Ensure cursor is visible after position change
                self.ensure_cursor_visible(current_pane);
            } else {
                tracing::warn!(
                    "Failed to map logical position {:?} to display position in pane {:?}",
                    position,
                    current_pane
                );
            }

            self.emit_view_event(ViewEvent::CursorUpdateRequired { pane: current_pane });
        }

        Ok(())
    }

    /// Switch to different pane
    pub fn switch_pane(&mut self, pane: Pane) -> Result<()> {
        if let Some(event) = self.editor.set_current_pane(pane) {
            self.emit_model_event(event);
            self.emit_view_event(ViewEvent::FullRedrawRequired);
        }
        Ok(())
    }

    /// Change editor mode
    pub fn change_mode(&mut self, mode: EditorMode) -> Result<()> {
        if let Some(event) = self.editor.set_mode(mode) {
            self.emit_model_event(event);
            self.emit_view_event(ViewEvent::StatusBarUpdateRequired);
        }
        Ok(())
    }

    /// Insert character at cursor in current pane
    pub fn insert_char(&mut self, ch: char) -> Result<()> {
        // Only allow text insertion in request pane and insert mode
        if self.editor.current_pane() != Pane::Request || self.editor.mode() != EditorMode::Insert {
            return Ok(());
        }

        let event = self.request_buffer.insert_char(ch);
        self.emit_model_event(event);

        // Rebuild request display cache after content change
        let content_width = self.get_content_width();
        let request_lines = self.request_buffer.content().lines().to_vec();
        if let Ok(cache) = build_display_cache(&request_lines, content_width, self.wrap_enabled) {
            self.request_display_cache = cache;
        }

        // Sync display cursor with logical cursor
        self.sync_display_cursors();

        self.emit_view_event(ViewEvent::PaneRedrawRequired {
            pane: Pane::Request,
        });

        Ok(())
    }

    /// Insert text at cursor in current pane
    pub fn insert_text(&mut self, text: &str) -> Result<()> {
        // Only allow text insertion in request pane and insert mode
        if self.editor.current_pane() != Pane::Request || self.editor.mode() != EditorMode::Insert {
            return Ok(());
        }

        let event = self.request_buffer.insert_text(text);
        self.emit_model_event(event);

        // Rebuild request display cache after content change
        let content_width = self.get_content_width();
        let request_lines = self.request_buffer.content().lines().to_vec();
        if let Ok(cache) = build_display_cache(&request_lines, content_width, self.wrap_enabled) {
            self.request_display_cache = cache;
        }

        // Sync display cursor with logical cursor
        self.sync_display_cursors();

        self.emit_view_event(ViewEvent::PaneRedrawRequired {
            pane: Pane::Request,
        });

        Ok(())
    }

    /// Delete character before cursor (backspace)
    pub fn delete_char_before_cursor(&mut self) -> Result<()> {
        // Only allow deletion in request pane and insert mode
        if self.editor.current_pane() != Pane::Request || self.editor.mode() != EditorMode::Insert {
            return Ok(());
        }

        let current_pos = self.request_buffer.cursor();

        if current_pos.column > 0 {
            // Delete character in current line
            let delete_pos = LogicalPosition::new(current_pos.line, current_pos.column - 1);
            let range = crate::repl::events::LogicalRange::single_char(delete_pos);

            if let Some(event) = self
                .request_buffer
                .content_mut()
                .delete_range(Pane::Request, range)
            {
                // Move cursor back
                self.request_buffer.set_cursor(delete_pos);

                self.emit_model_event(event);

                // Rebuild request display cache after content change
                let content_width = self.get_content_width();
                let request_lines = self.request_buffer.content().lines().to_vec();
                if let Ok(cache) =
                    build_display_cache(&request_lines, content_width, self.wrap_enabled)
                {
                    self.request_display_cache = cache;
                }
                self.sync_display_cursors();

                self.emit_view_event(ViewEvent::PaneRedrawRequired {
                    pane: Pane::Request,
                });
            }
        } else if current_pos.line > 0 {
            // Join with previous line
            let prev_line_length = self
                .request_buffer
                .content()
                .line_length(current_pos.line - 1);
            let new_cursor = LogicalPosition::new(current_pos.line - 1, prev_line_length);

            // Get current line content
            if let Some(current_line) = self.request_buffer.content().get_line(current_pos.line) {
                let current_line_content = current_line.clone();

                // Delete current line and append to previous
                let range = crate::repl::events::LogicalRange::new(
                    LogicalPosition::new(current_pos.line - 1, prev_line_length),
                    LogicalPosition::new(current_pos.line + 1, 0),
                );

                if self
                    .request_buffer
                    .content_mut()
                    .delete_range(Pane::Request, range)
                    .is_some()
                {
                    // Insert the content at the end of previous line
                    self.request_buffer.content_mut().insert_text(
                        Pane::Request,
                        new_cursor,
                        &current_line_content,
                    );

                    self.request_buffer.set_cursor(new_cursor);

                    // Rebuild request display cache after content change
                    let content_width = self.get_content_width();
                    let request_lines = self.request_buffer.content().lines().to_vec();
                    if let Ok(cache) =
                        build_display_cache(&request_lines, content_width, self.wrap_enabled)
                    {
                        self.request_display_cache = cache;
                    }
                    self.sync_display_cursors();

                    self.emit_view_event(ViewEvent::PaneRedrawRequired {
                        pane: Pane::Request,
                    });
                }
            }
        }

        Ok(())
    }

    /// Delete character after cursor (delete key)
    pub fn delete_char_after_cursor(&mut self) -> Result<()> {
        // Only allow deletion in request pane and insert mode
        if self.editor.current_pane() != Pane::Request || self.editor.mode() != EditorMode::Insert {
            return Ok(());
        }

        let current_pos = self.request_buffer.cursor();
        let current_line_length = self.request_buffer.content().line_length(current_pos.line);

        if current_pos.column < current_line_length {
            // Delete character in current line
            let range = crate::repl::events::LogicalRange::single_char(current_pos);

            if let Some(event) = self
                .request_buffer
                .content_mut()
                .delete_range(Pane::Request, range)
            {
                self.emit_model_event(event);

                // Rebuild request display cache after content change
                let content_width = self.get_content_width();
                let request_lines = self.request_buffer.content().lines().to_vec();
                if let Ok(cache) =
                    build_display_cache(&request_lines, content_width, self.wrap_enabled)
                {
                    self.request_display_cache = cache;
                }
                self.sync_display_cursors();

                self.emit_view_event(ViewEvent::PaneRedrawRequired {
                    pane: Pane::Request,
                });
            }
        }
        // Note: We don't handle joining with next line for simplicity
        // That would be a more complex operation

        Ok(())
    }

    /// Update terminal size
    pub fn update_terminal_size(&mut self, width: u16, height: u16) {
        if self.terminal_width != width || self.terminal_height != height {
            self.terminal_width = width;
            self.terminal_height = height;
            self.request_pane_height = height / 2; // Split screen in half

            // Rebuild display caches if width changed (affects wrapping)
            if let Err(e) = self.rebuild_display_caches() {
                tracing::warn!(
                    "Failed to rebuild display caches after terminal resize: {}",
                    e
                );
            }

            self.emit_view_event(ViewEvent::FullRedrawRequired);
        }
    }

    /// Get request buffer content as text
    pub fn get_request_text(&self) -> String {
        self.request_buffer.content().get_text()
    }

    /// Get response buffer content as text
    pub fn get_response_text(&self) -> String {
        self.response_buffer.content().get_text()
    }

    /// Set response content (from HTTP response)
    pub fn set_response(&mut self, status_code: u16, body: String) {
        self.response.set_status_code(status_code);
        self.response.set_body(body.clone());

        // Update response buffer content
        self.response_buffer.content_mut().set_text(&body);
        self.response_buffer.set_cursor(LogicalPosition::zero());

        // Rebuild response display cache
        let content_width = self.get_content_width();
        let response_lines = self.response_buffer.content().lines().to_vec();
        if let Ok(cache) = build_display_cache(&response_lines, content_width, self.wrap_enabled) {
            self.response_display_cache = cache;
        }

        // Reset response display cursor and scroll
        self.response_display_cursor = (0, 0);
        self.response_display_scroll_offset = 0;

        let event = ModelEvent::ResponseReceived { status_code, body };
        self.emit_model_event(event);
        self.emit_view_event(ViewEvent::PaneRedrawRequired {
            pane: Pane::Response,
        });
    }

    /// Set response from HttpResponse object
    pub fn set_response_from_http(&mut self, response: &bluenote::HttpResponse) {
        let status_code = response.status().as_u16();
        let status_message = response
            .status()
            .canonical_reason()
            .unwrap_or("")
            .to_string();
        let duration_ms = response.duration_ms();
        let body = response.body().to_string();

        self.response.set_status_code(status_code);
        self.response.set_status_message(status_message);
        self.response.set_duration_ms(duration_ms);
        self.response.set_body(body.clone());

        // Update response buffer content
        self.response_buffer.content_mut().set_text(&body);
        self.response_buffer.set_cursor(LogicalPosition::zero());

        // Rebuild response display cache
        let content_width = self.get_content_width();
        let response_lines = self.response_buffer.content().lines().to_vec();
        if let Ok(cache) = build_display_cache(&response_lines, content_width, self.wrap_enabled) {
            self.response_display_cache = cache;
        }

        // Reset response display cursor and scroll
        self.response_display_cursor = (0, 0);
        self.response_display_scroll_offset = 0;

        let event = ModelEvent::ResponseReceived { status_code, body };
        self.emit_model_event(event);
        self.emit_view_event(ViewEvent::PaneRedrawRequired {
            pane: Pane::Response,
        });
    }

    /// Set HTTP client with custom profile
    pub fn set_http_client(&mut self, profile: &impl HttpConnectionProfile) -> Result<()> {
        self.http_client = Some(HttpClient::new(profile)?);
        Ok(())
    }

    /// Add session header
    pub fn add_session_header(&mut self, key: String, value: String) {
        self.session_headers.insert(key, value);
    }

    /// Remove session header
    pub fn remove_session_header(&mut self, key: &str) {
        self.session_headers.remove(key);
    }

    /// Set verbose mode
    pub fn set_verbose(&mut self, verbose: bool) {
        self.verbose = verbose;
    }

    /// Get verbose mode
    pub fn is_verbose(&self) -> bool {
        self.verbose
    }

    /// Get HTTP client reference
    pub fn http_client(&self) -> Option<&HttpClient> {
        self.http_client.as_ref()
    }

    /// Add character to ex command buffer
    pub fn add_ex_command_char(&mut self, ch: char) -> Result<()> {
        self.ex_command_buffer.push(ch);
        self.emit_view_event(ViewEvent::StatusBarUpdateRequired);
        Ok(())
    }

    /// Remove character from ex command buffer (backspace)
    pub fn backspace_ex_command(&mut self) -> Result<()> {
        if !self.ex_command_buffer.is_empty() {
            self.ex_command_buffer.pop();
            self.emit_view_event(ViewEvent::StatusBarUpdateRequired);
        } else {
            // If buffer is empty, exit command mode
            self.change_mode(EditorMode::Normal)?;
        }
        Ok(())
    }

    /// Execute ex command and clear buffer
    pub fn execute_ex_command(&mut self) -> Result<Vec<crate::repl::commands::CommandEvent>> {
        let command = self.ex_command_buffer.trim();
        let mut events = Vec::new();

        // Handle ex commands
        match command {
            "q" => {
                // Quit the application
                events.push(crate::repl::commands::CommandEvent::QuitRequested);
            }
            "q!" => {
                // Force quit the application
                events.push(crate::repl::commands::CommandEvent::QuitRequested);
            }
            "set wrap" => {
                // Enable word wrap
                if let Err(e) = self.set_wrap_enabled(true) {
                    tracing::warn!("Failed to enable word wrap: {}", e);
                }
            }
            "set nowrap" => {
                // Disable word wrap
                if let Err(e) = self.set_wrap_enabled(false) {
                    tracing::warn!("Failed to disable word wrap: {}", e);
                }
            }
            "" => {
                // Empty command, just exit command mode
            }
            _ => {
                // Unknown command - could emit an error event in future
                tracing::warn!("Unknown ex command: {}", command);
            }
        }

        // Clear buffer and exit command mode
        self.ex_command_buffer.clear();
        self.change_mode(EditorMode::Normal)?;

        Ok(events)
    }

    /// Get ex command buffer for display
    pub fn get_ex_command_buffer(&self) -> &str {
        &self.ex_command_buffer
    }

    /// Get HTTP response status code for display
    pub fn get_response_status_code(&self) -> Option<u16> {
        self.response.status_code()
    }

    /// Get HTTP response status message for display
    pub fn get_response_status_message(&self) -> Option<&String> {
        self.response.status_message()
    }

    /// Get HTTP response duration in milliseconds for display
    pub fn get_response_duration_ms(&self) -> Option<u64> {
        self.response.duration_ms()
    }

    // =================================================================
    // Display Cache and Word Wrap Management
    // =================================================================

    /// Get wrap enabled state
    pub fn is_wrap_enabled(&self) -> bool {
        self.wrap_enabled
    }

    /// Set wrap enabled state and rebuild caches
    pub fn set_wrap_enabled(&mut self, enabled: bool) -> Result<()> {
        if self.wrap_enabled != enabled {
            self.wrap_enabled = enabled;
            self.rebuild_display_caches()?;
            self.emit_view_event(ViewEvent::FullRedrawRequired);
        }
        Ok(())
    }

    /// Rebuild display caches for both panes
    fn rebuild_display_caches(&mut self) -> Result<()> {
        let content_width = self.get_content_width();

        // Rebuild request cache
        let request_lines = self.request_buffer.content().lines().to_vec();
        self.request_display_cache =
            build_display_cache(&request_lines, content_width, self.wrap_enabled)?;

        // Rebuild response cache
        let response_lines = self.response_buffer.content().lines().to_vec();
        self.response_display_cache =
            build_display_cache(&response_lines, content_width, self.wrap_enabled)?;

        // Update display cursors to match logical positions
        self.sync_display_cursors();

        Ok(())
    }

    /// Calculate content width (terminal width minus line number space)
    fn get_content_width(&self) -> usize {
        // Reserve space for line numbers (minimum 3 digits + 1 space)
        let line_num_width = 4;
        (self.terminal_width as usize).saturating_sub(line_num_width)
    }

    /// Sync display cursors with logical buffer cursors
    fn sync_display_cursors(&mut self) {
        // Update request display cursor
        let request_logical = self.request_buffer.cursor();
        if let Some(display_pos) = self
            .request_display_cache
            .logical_to_display_position(request_logical.line, request_logical.column)
        {
            self.request_display_cursor = display_pos;
        }

        // Update response display cursor
        let response_logical = self.response_buffer.cursor();
        if let Some(display_pos) = self
            .response_display_cache
            .logical_to_display_position(response_logical.line, response_logical.column)
        {
            self.response_display_cursor = display_pos;
        }
    }

    /// Get display cursor for a pane
    pub fn get_display_cursor(&self, pane: Pane) -> DisplayPosition {
        match pane {
            Pane::Request => self.request_display_cursor,
            Pane::Response => self.response_display_cursor,
        }
    }

    /// Get display cache for a pane
    pub fn get_display_cache(&self, pane: Pane) -> &DisplayCache {
        match pane {
            Pane::Request => &self.request_display_cache,
            Pane::Response => &self.response_display_cache,
        }
    }

    /// Set display cursor and sync logical cursor
    fn set_display_cursor(&mut self, pane: Pane, display_pos: DisplayPosition) -> Result<()> {
        match pane {
            Pane::Request => {
                self.request_display_cursor = display_pos;
                if let Some((logical_line, logical_col)) = self
                    .request_display_cache
                    .display_to_logical_position(display_pos.0, display_pos.1)
                {
                    let logical_pos = LogicalPosition::new(logical_line, logical_col);
                    self.request_buffer.set_cursor(logical_pos);
                }
            }
            Pane::Response => {
                self.response_display_cursor = display_pos;
                if let Some((logical_line, logical_col)) = self
                    .response_display_cache
                    .display_to_logical_position(display_pos.0, display_pos.1)
                {
                    let logical_pos = LogicalPosition::new(logical_line, logical_col);
                    self.response_buffer.set_cursor(logical_pos);
                }
            }
        }

        // Ensure cursor is visible after move
        self.ensure_cursor_visible(pane);
        Ok(())
    }

    /// Sync display cursor with current logical cursor position
    fn sync_display_cursor_with_logical(&mut self, pane: Pane) -> Result<()> {
        let logical_pos = self.get_cursor_for_pane(pane);
        let display_cache = self.get_display_cache(pane);

        if let Some(display_pos) =
            display_cache.logical_to_display_position(logical_pos.line, logical_pos.column)
        {
            match pane {
                Pane::Request => self.request_display_cursor = display_pos,
                Pane::Response => self.response_display_cursor = display_pos,
            }
        }
        Ok(())
    }

    /// Ensure cursor is visible in the current pane by adjusting scroll offset
    fn ensure_cursor_visible(&mut self, pane: Pane) {
        let cursor_display_line = match pane {
            Pane::Request => self.request_display_cursor.0,
            Pane::Response => self.response_display_cursor.0,
        };

        let scroll_offset = match pane {
            Pane::Request => self.request_display_scroll_offset,
            Pane::Response => self.response_display_scroll_offset,
        };

        let pane_height = match pane {
            Pane::Request => self.request_pane_height() as usize,
            Pane::Response => self.response_pane_height() as usize,
        };

        let visible_start = scroll_offset;
        let visible_end = scroll_offset + pane_height.saturating_sub(1);

        // If cursor is above visible area, scroll up
        if cursor_display_line < visible_start {
            match pane {
                Pane::Request => self.request_display_scroll_offset = cursor_display_line,
                Pane::Response => self.response_display_scroll_offset = cursor_display_line,
            }
        }
        // If cursor is below visible area, scroll down
        else if cursor_display_line > visible_end {
            let new_offset = cursor_display_line.saturating_sub(pane_height.saturating_sub(1));
            match pane {
                Pane::Request => self.request_display_scroll_offset = new_offset,
                Pane::Response => self.response_display_scroll_offset = new_offset,
            }
        }
    }

    // =================================================================
    // Internal helper methods
    // =================================================================

    /// Get mutable buffer for pane
    #[allow(dead_code)]
    fn get_buffer_mut(&mut self, pane: Pane) -> &mut BufferModel {
        match pane {
            Pane::Request => &mut self.request_buffer,
            Pane::Response => &mut self.response_buffer,
        }
    }

    /// Emit model event through event bus
    fn emit_model_event(&mut self, event: ModelEvent) {
        if let Some(event_bus) = &mut self.event_bus {
            event_bus.publish_model_event(event);
        }
    }

    /// Emit view event through event bus
    fn emit_view_event(&mut self, event: ViewEvent) {
        if let Some(event_bus) = &mut self.event_bus {
            event_bus.publish_view_event(event);
        }
    }

    // =================================================================
    // Read-only accessors for view layer
    // =================================================================

    /// Get terminal dimensions
    pub fn terminal_size(&self) -> (u16, u16) {
        (self.terminal_width, self.terminal_height)
    }

    /// Get request pane height
    pub fn request_pane_height(&self) -> u16 {
        // If response pane is hidden, request pane uses full available space
        if self.response.status_code().is_some() {
            // When response exists, use configured split height
            self.request_pane_height
        } else {
            // When no response, request pane uses full available space
            self.terminal_height - 1 // -1 for status bar, no separator needed
        }
    }

    /// Get response pane height
    pub fn response_pane_height(&self) -> u16 {
        // Hide response pane until there's an actual HTTP response
        if self.response.status_code().is_some() {
            // When response exists, calculate remaining space after request pane, separator, and status bar
            self.terminal_height
                .saturating_sub(self.request_pane_height + 2) // configured split height + separator + status bar
        } else {
            0 // Hidden when no response
        }
    }

    /// Get buffer content for a pane
    pub fn get_buffer_content(&self, pane: Pane) -> String {
        match pane {
            Pane::Request => self.request_buffer.content().get_text(),
            Pane::Response => self.response_buffer.content().get_text(),
        }
    }

    /// Get cursor for a pane
    pub fn get_cursor_for_pane(&self, pane: Pane) -> LogicalPosition {
        match pane {
            Pane::Request => self.request_buffer.cursor(),
            Pane::Response => self.response_buffer.cursor(),
        }
    }

    /// Get scroll offset for a pane (display line based)
    pub fn get_scroll_offset(&self, pane: Pane) -> usize {
        match pane {
            Pane::Request => self.request_display_scroll_offset,
            Pane::Response => self.response_display_scroll_offset,
        }
    }

    /// Get display lines for rendering (for views layer)
    pub fn get_display_lines_for_rendering(
        &self,
        pane: Pane,
        start_row: usize,
        row_count: usize,
    ) -> Vec<Option<DisplayLineData>> {
        let display_cache = self.get_display_cache(pane);
        let scroll_offset = self.get_scroll_offset(pane);
        let mut result = Vec::new();

        for row in 0..row_count {
            let display_line_idx = scroll_offset + start_row + row;

            if let Some(display_line) = display_cache.get_display_line(display_line_idx) {
                // Show logical line number only for first segment of wrapped lines
                let line_number = if display_line.is_continuation {
                    None
                } else {
                    Some(display_line.logical_line + 1) // 1-based line numbers
                };
                // Third parameter indicates if this is a continuation line (true) or beyond content (false)
                result.push(Some((
                    display_line.content.clone(),
                    line_number,
                    display_line.is_continuation,
                )));
            } else {
                // Beyond content - show tilde (false indicates this is beyond content, not continuation)
                result.push(None);
            }
        }

        result
    }

    /// Get cursor position for rendering (display coordinates)
    pub fn get_cursor_for_rendering(&self, pane: Pane) -> (usize, usize) {
        let display_cursor = self.get_display_cursor(pane);
        let scroll_offset = self.get_scroll_offset(pane);

        // Return relative position within visible area
        let visible_row = display_cursor.0.saturating_sub(scroll_offset);
        (visible_row, display_cursor.1)
    }

    /// Get line number width for consistent formatting
    pub fn get_line_number_width(&self, pane: Pane) -> usize {
        let logical_lines = match pane {
            Pane::Request => self.request_buffer.content().line_count(),
            Pane::Response => self.response_buffer.content().line_count(),
        };
        logical_lines.to_string().len().max(3)
    }
}

impl Default for ViewModel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn view_model_should_create_with_defaults() {
        let vm = ViewModel::new();

        assert_eq!(vm.get_mode(), EditorMode::Normal);
        assert_eq!(vm.get_current_pane(), Pane::Request);
        assert_eq!(vm.get_cursor_position(), LogicalPosition::zero());
    }

    #[test]
    fn view_model_should_switch_panes() {
        let mut vm = ViewModel::new();

        vm.switch_pane(Pane::Response).unwrap();

        assert_eq!(vm.get_current_pane(), Pane::Response);
    }

    #[test]
    fn view_model_should_change_mode() {
        let mut vm = ViewModel::new();

        vm.change_mode(EditorMode::Insert).unwrap();

        assert_eq!(vm.get_mode(), EditorMode::Insert);
    }

    #[test]
    fn view_model_should_move_cursor_left() {
        let mut vm = ViewModel::new();
        vm.change_mode(EditorMode::Insert).unwrap();
        vm.insert_text("hello").unwrap();

        vm.move_cursor_left().unwrap();

        let cursor = vm.get_cursor_position();
        assert_eq!(cursor.column, 4); // Moved from 5 to 4
    }

    #[test]
    fn view_model_should_move_cursor_right() {
        let mut vm = ViewModel::new();
        vm.change_mode(EditorMode::Insert).unwrap();
        vm.insert_text("hello").unwrap();
        vm.request_buffer.set_cursor(LogicalPosition::new(0, 2));

        vm.move_cursor_right().unwrap();

        let cursor = vm.get_cursor_position();
        assert_eq!(cursor.column, 3); // Moved from 2 to 3
    }

    #[test]
    fn view_model_should_insert_char_in_insert_mode() {
        let mut vm = ViewModel::new();
        vm.change_mode(EditorMode::Insert).unwrap();

        vm.insert_char('a').unwrap();

        let text = vm.get_request_text();
        assert_eq!(text, "a");

        let cursor = vm.get_cursor_position();
        assert_eq!(cursor.column, 1);
    }

    #[test]
    fn view_model_should_not_insert_char_in_normal_mode() {
        let mut vm = ViewModel::new();
        // Stay in Normal mode

        vm.insert_char('a').unwrap();

        let text = vm.get_request_text();
        assert_eq!(text, ""); // Should be empty
    }

    #[test]
    fn view_model_should_update_terminal_size() {
        let mut vm = ViewModel::new();

        vm.update_terminal_size(120, 40);

        let (width, height) = vm.terminal_size();
        assert_eq!(width, 120);
        assert_eq!(height, 40);
        // When no response, request pane takes full available space (height - 1 for status bar)
        assert_eq!(vm.request_pane_height(), 39);
    }

    #[test]
    fn view_model_should_move_cursor_to_end_of_line() {
        let mut vm = ViewModel::new();
        vm.change_mode(EditorMode::Insert).unwrap();
        vm.insert_text("hello world").unwrap();
        // Move cursor to middle of line
        vm.request_buffer.set_cursor(LogicalPosition::new(0, 5));

        // Test move to end of line
        vm.move_cursor_to_end_of_line().unwrap();
        let cursor = vm.get_cursor_position();
        assert_eq!(cursor.column, 11); // Should be at end of "hello world"
        assert_eq!(cursor.line, 0);
    }

    #[test]
    fn view_model_should_move_cursor_to_end_of_empty_line() {
        let mut vm = ViewModel::new();
        // Start with empty buffer
        vm.move_cursor_to_end_of_line().unwrap();
        let cursor = vm.get_cursor_position();
        assert_eq!(cursor.column, 0); // Should stay at 0 for empty line
        assert_eq!(cursor.line, 0);
    }

    #[test]
    fn view_model_should_move_cursor_to_end_of_multiline_text() {
        let mut vm = ViewModel::new();
        vm.change_mode(EditorMode::Insert).unwrap();
        vm.insert_text("line one\nline two is longer\nline three")
            .unwrap();
        // Move to middle line, middle position
        vm.set_cursor_position(LogicalPosition::new(1, 5)).unwrap();

        // Test move to end of middle line
        vm.move_cursor_to_end_of_line().unwrap();
        let cursor = vm.get_cursor_position();

        assert_eq!(cursor.column, 18); // Should be at end of "line two is longer"
        assert_eq!(cursor.line, 1);
    }

    #[test]
    fn view_model_should_move_cursor_to_start_of_line() {
        let mut vm = ViewModel::new();
        vm.change_mode(EditorMode::Insert).unwrap();
        vm.insert_text("hello world").unwrap();
        // Move cursor to middle of line
        vm.request_buffer.set_cursor(LogicalPosition::new(0, 6));

        // Test move to start of line
        vm.move_cursor_to_start_of_line().unwrap();
        let cursor = vm.get_cursor_position();
        assert_eq!(cursor.column, 0); // Should be at start of line
        assert_eq!(cursor.line, 0);
    }

    #[test]
    fn view_model_should_move_cursor_to_start_of_empty_line() {
        let mut vm = ViewModel::new();
        // Start with empty buffer
        vm.move_cursor_to_start_of_line().unwrap();
        let cursor = vm.get_cursor_position();
        assert_eq!(cursor.column, 0); // Should stay at 0 for empty line
        assert_eq!(cursor.line, 0);
    }

    #[test]
    fn view_model_should_set_response() {
        let mut vm = ViewModel::new();

        vm.set_response(200, "test response".to_string());

        assert_eq!(vm.get_response_text(), "test response");
        assert_eq!(vm.response.status_code(), Some(200));
    }
}
