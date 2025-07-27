//! # ViewModel - Business Logic Coordinator
//!
//! The ViewModel contains all business logic and coordinates between models.
//! It's the central component that commands delegate to and that emits events.

use crate::repl::events::{EditorMode, EventBus, LogicalPosition, ModelEvent, Pane, ViewEvent};
use crate::repl::http::create_default_profile;
use crate::repl::models::{BufferModel, EditorModel, ResponseModel};
use anyhow::Result;
use bluenote::{HttpClient, HttpConnectionProfile};
use std::collections::HashMap;

/// Type alias for event bus option to reduce complexity
type EventBusOption = Option<Box<dyn EventBus>>;

/// The central ViewModel that coordinates all business logic
pub struct ViewModel {
    // Core models
    editor: EditorModel,
    request_buffer: BufferModel,
    response_buffer: BufferModel,
    response: ResponseModel,

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

        Self {
            editor: EditorModel::new(),
            request_buffer: BufferModel::new(Pane::Request),
            response_buffer: BufferModel::new(Pane::Response),
            response: ResponseModel::new(),
            terminal_width: 80,
            terminal_height: 24,
            request_pane_height: 12,
            ex_command_buffer: String::new(),
            http_client,
            session_headers: HashMap::new(),
            verbose: false,
            event_bus: None,
        }
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

    /// Move cursor left in current pane
    pub fn move_cursor_left(&mut self) -> Result<()> {
        let current_pane = self.editor.current_pane();
        let buffer = self.get_buffer_mut(current_pane);

        if let Some(event) = buffer.move_cursor_left() {
            self.emit_model_event(event);
            self.emit_view_event(ViewEvent::CursorUpdateRequired { pane: current_pane });
        }

        Ok(())
    }

    /// Move cursor right in current pane
    pub fn move_cursor_right(&mut self) -> Result<()> {
        let current_pane = self.editor.current_pane();
        let buffer = self.get_buffer_mut(current_pane);

        if let Some(event) = buffer.move_cursor_right() {
            self.emit_model_event(event);
            self.emit_view_event(ViewEvent::CursorUpdateRequired { pane: current_pane });
        }

        Ok(())
    }

    /// Move cursor up in current pane
    pub fn move_cursor_up(&mut self) -> Result<()> {
        let current_pane = self.editor.current_pane();
        let buffer = self.get_buffer_mut(current_pane);
        let current_pos = buffer.cursor();

        if current_pos.line > 0 {
            let new_pos = LogicalPosition::new(current_pos.line - 1, current_pos.column);
            if let Some(event) = buffer.set_cursor(new_pos) {
                self.emit_model_event(event);
                self.emit_view_event(ViewEvent::CursorUpdateRequired { pane: current_pane });
            }
        }

        Ok(())
    }

    /// Move cursor down in current pane
    pub fn move_cursor_down(&mut self) -> Result<()> {
        let current_pane = self.editor.current_pane();
        let buffer = self.get_buffer_mut(current_pane);
        let current_pos = buffer.cursor();

        if current_pos.line + 1 < buffer.content().line_count() {
            let new_pos = LogicalPosition::new(current_pos.line + 1, current_pos.column);
            if let Some(event) = buffer.set_cursor(new_pos) {
                self.emit_model_event(event);
                self.emit_view_event(ViewEvent::CursorUpdateRequired { pane: current_pane });
            }
        }

        Ok(())
    }

    /// Move cursor to end of current line
    pub fn move_cursor_to_end_of_line(&mut self) -> Result<()> {
        let current_pane = self.editor.current_pane();
        let buffer = self.get_buffer_mut(current_pane);
        let current_pos = buffer.cursor();
        let line_length = buffer.content().line_length(current_pos.line);
        let new_pos = LogicalPosition::new(current_pos.line, line_length);
        if let Some(event) = buffer.set_cursor(new_pos) {
            self.emit_model_event(event);
            self.emit_view_event(ViewEvent::CursorUpdateRequired { pane: current_pane });
        }
        Ok(())
    }

    /// Move cursor to start of current line
    pub fn move_cursor_to_start_of_line(&mut self) -> Result<()> {
        let current_pane = self.editor.current_pane();
        let buffer = self.get_buffer_mut(current_pane);
        let current_pos = buffer.cursor();
        let new_pos = LogicalPosition::new(current_pos.line, 0);
        if let Some(event) = buffer.set_cursor(new_pos) {
            self.emit_model_event(event);
            self.emit_view_event(ViewEvent::CursorUpdateRequired { pane: current_pane });
        }
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
    // Internal helper methods
    // =================================================================

    /// Get mutable buffer for pane
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

    /// Get scroll offset for a pane
    pub fn get_scroll_offset(&self, pane: Pane) -> usize {
        match pane {
            Pane::Request => self.request_buffer.scroll_offset(),
            Pane::Response => self.response_buffer.scroll_offset(),
        }
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
        vm.request_buffer.set_cursor(LogicalPosition::new(1, 5));

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
