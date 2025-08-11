//! # Status Line Model
//!
//! Encapsulates all state related to the status line display,
//! providing a clean interface for status bar rendering.

use crate::repl::events::{EditorMode, LogicalPosition, Pane};

/// Type alias for display position
type DisplayPosition = (usize, usize);

/// HTTP response information for status display
#[derive(Debug, Clone, Default)]
pub struct HttpStatus {
    /// HTTP status code (e.g., 200, 404)
    pub status_code: Option<u16>,
    /// HTTP status message (e.g., "OK", "Not Found")
    pub status_message: Option<String>,
    /// Request duration in milliseconds
    pub duration_ms: Option<u64>,
}

/// Status line model containing all status bar display state
#[derive(Debug, Clone)]
pub struct StatusLine {
    /// Temporary status message to display
    status_message: Option<String>,

    /// Ex command buffer (for :q, :set wrap, etc.)
    command_buffer: String,

    /// HTTP response status information
    http_status: HttpStatus,

    /// Profile name and path
    profile_name: String,
    profile_path: String,

    /// Current editor mode
    editor_mode: EditorMode,

    /// Previous editor mode (for restoring after command cancellation)
    previous_mode: EditorMode,

    /// Current pane and cursor position
    current_pane: Pane,
    cursor_position: LogicalPosition,

    /// Whether a request is currently executing
    is_executing: bool,

    /// Display/visual position marker for debugging purposes
    /// Format: (display_line, display_column)
    display_position: Option<DisplayPosition>,

    /// Whether to show display cursor position in status bar
    /// When false: shows "1:1" format only
    /// When true: shows "1:1 (1:1)" format with display position
    display_cursor_visible: bool,
}

impl StatusLine {
    /// Create a new StatusLine with default values
    pub fn new() -> Self {
        Self {
            status_message: None,
            command_buffer: String::new(),
            http_status: HttpStatus::default(),
            profile_name: "default".to_string(),
            profile_path: "~/.blueline/profile".to_string(),
            editor_mode: EditorMode::Normal,
            previous_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            cursor_position: LogicalPosition::zero(),
            is_executing: false,
            display_position: None,
            #[allow(clippy::disallowed_methods)]
            display_cursor_visible: std::env::var("BLUELINE_SHOW_DISP_CURSOR_POS").is_ok(), // Show display cursor position if env var is set
        }
    }

    // === Status Message Methods ===

    /// Set a temporary status message
    pub fn set_status_message<S: Into<String>>(&mut self, message: S) {
        self.status_message = Some(message.into());
    }

    /// Clear the status message
    pub fn clear_status_message(&mut self) {
        self.status_message = None;
    }

    /// Get the current status message
    pub fn status_message(&self) -> Option<&str> {
        self.status_message.as_deref()
    }

    // === Command Buffer Methods ===

    /// Get the command buffer content
    pub fn command_buffer(&self) -> &str {
        &self.command_buffer
    }

    /// Append a character to the command buffer
    pub fn append_to_command_buffer(&mut self, ch: char) {
        self.command_buffer.push(ch);
    }

    /// Remove the last character from the command buffer
    pub fn backspace_command_buffer(&mut self) {
        self.command_buffer.pop();
    }

    /// Clear the command buffer
    pub fn clear_command_buffer(&mut self) {
        self.command_buffer.clear();
    }

    /// Get the command buffer and clear it
    pub fn take_command_buffer(&mut self) -> String {
        std::mem::take(&mut self.command_buffer)
    }

    // === HTTP Status Methods ===

    /// Set HTTP response status
    pub fn set_http_status(&mut self, status_code: u16, status_message: String, duration_ms: u64) {
        self.http_status.status_code = Some(status_code);
        self.http_status.status_message = Some(status_message);
        self.http_status.duration_ms = Some(duration_ms);
    }

    /// Clear HTTP status
    pub fn clear_http_status(&mut self) {
        self.http_status = HttpStatus::default();
    }

    /// Get HTTP status code
    pub fn http_status_code(&self) -> Option<u16> {
        self.http_status.status_code
    }

    /// Get HTTP status message
    pub fn http_status_message(&self) -> Option<&str> {
        self.http_status.status_message.as_deref()
    }

    /// Get HTTP response duration
    pub fn http_duration_ms(&self) -> Option<u64> {
        self.http_status.duration_ms
    }

    /// Get full HTTP status info
    pub fn http_status(&self) -> &HttpStatus {
        &self.http_status
    }

    // === Profile Methods ===

    /// Set profile information
    pub fn set_profile(&mut self, name: String, path: String) {
        self.profile_name = name;
        self.profile_path = path;
    }

    /// Get profile name
    pub fn profile_name(&self) -> &str {
        &self.profile_name
    }

    /// Get profile path
    pub fn profile_path(&self) -> &str {
        &self.profile_path
    }

    // === Editor State Methods ===

    /// Set editor mode
    pub fn set_editor_mode(&mut self, mode: EditorMode) {
        self.previous_mode = self.editor_mode;
        self.editor_mode = mode;
    }

    /// Get editor mode
    pub fn editor_mode(&self) -> EditorMode {
        self.editor_mode
    }

    /// Get previous editor mode
    pub fn previous_mode(&self) -> EditorMode {
        self.previous_mode
    }

    /// Set current pane
    pub fn set_current_pane(&mut self, pane: Pane) {
        self.current_pane = pane;
    }

    /// Get current pane
    pub fn current_pane(&self) -> Pane {
        self.current_pane
    }

    /// Set cursor position
    pub fn set_cursor_position(&mut self, position: LogicalPosition) {
        self.cursor_position = position;
    }

    /// Get cursor position
    pub fn cursor_position(&self) -> LogicalPosition {
        self.cursor_position
    }

    // === Execution State Methods ===

    /// Set whether a request is executing
    pub fn set_executing(&mut self, executing: bool) {
        self.is_executing = executing;
    }

    /// Check if a request is executing
    pub fn is_executing(&self) -> bool {
        self.is_executing
    }

    // === Display Position Methods (for debugging) ===

    /// Set display position marker
    pub fn set_display_position(&mut self, position: Option<DisplayPosition>) {
        self.display_position = position;
    }

    /// Get display position marker
    pub fn display_position(&self) -> Option<DisplayPosition> {
        self.display_position
    }

    /// Set whether display cursor position is visible in status bar
    pub fn set_display_cursor_visible(&mut self, visible: bool) {
        self.display_cursor_visible = visible;
    }

    /// Check if display cursor position is visible in status bar
    pub fn is_display_cursor_visible(&self) -> bool {
        self.display_cursor_visible
    }
}

impl Default for StatusLine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_line_creation() {
        let status = StatusLine::new();
        assert_eq!(status.status_message(), None);
        assert_eq!(status.command_buffer(), "");
        assert_eq!(status.http_status_code(), None);
        assert_eq!(status.profile_name(), "default");
        assert_eq!(status.editor_mode(), EditorMode::Normal);
        assert_eq!(status.current_pane(), Pane::Request);
        assert!(!status.is_executing());
        assert_eq!(status.display_position(), None);
        assert!(!status.is_display_cursor_visible());
    }

    #[test]
    fn test_status_message_operations() {
        let mut status = StatusLine::new();

        status.set_status_message("Test message");
        assert_eq!(status.status_message(), Some("Test message"));

        status.clear_status_message();
        assert_eq!(status.status_message(), None);
    }

    #[test]
    fn test_command_buffer_operations() {
        let mut status = StatusLine::new();

        status.append_to_command_buffer('q');
        status.append_to_command_buffer('u');
        status.append_to_command_buffer('i');
        status.append_to_command_buffer('t');
        assert_eq!(status.command_buffer(), "quit");

        status.backspace_command_buffer();
        assert_eq!(status.command_buffer(), "qui");

        let command = status.take_command_buffer();
        assert_eq!(command, "qui");
        assert_eq!(status.command_buffer(), "");
    }

    #[test]
    fn test_http_status_operations() {
        let mut status = StatusLine::new();

        status.set_http_status(200, "OK".to_string(), 150);
        assert_eq!(status.http_status_code(), Some(200));
        assert_eq!(status.http_status_message(), Some("OK"));
        assert_eq!(status.http_duration_ms(), Some(150));

        status.clear_http_status();
        assert_eq!(status.http_status_code(), None);
        assert_eq!(status.http_status_message(), None);
        assert_eq!(status.http_duration_ms(), None);
    }

    #[test]
    fn test_profile_operations() {
        let mut status = StatusLine::new();

        status.set_profile(
            "production".to_string(),
            "/etc/blueline/prod.ini".to_string(),
        );
        assert_eq!(status.profile_name(), "production");
        assert_eq!(status.profile_path(), "/etc/blueline/prod.ini");
    }

    #[test]
    fn test_editor_state_operations() {
        let mut status = StatusLine::new();

        status.set_editor_mode(EditorMode::Insert);
        assert_eq!(status.editor_mode(), EditorMode::Insert);
        assert_eq!(status.previous_mode(), EditorMode::Normal); // Previous mode should be tracked

        status.set_editor_mode(EditorMode::Visual);
        assert_eq!(status.editor_mode(), EditorMode::Visual);
        assert_eq!(status.previous_mode(), EditorMode::Insert); // Previous mode should update

        status.set_current_pane(Pane::Response);
        assert_eq!(status.current_pane(), Pane::Response);

        let pos = LogicalPosition::new(10, 5);
        status.set_cursor_position(pos);
        assert_eq!(status.cursor_position(), pos);
    }

    #[test]
    fn test_execution_state() {
        let mut status = StatusLine::new();

        assert!(!status.is_executing());

        status.set_executing(true);
        assert!(status.is_executing());

        status.set_executing(false);
        assert!(!status.is_executing());
    }

    #[test]
    fn test_display_position() {
        let mut status = StatusLine::new();

        assert_eq!(status.display_position(), None);

        status.set_display_position(Some((10, 20)));
        assert_eq!(status.display_position(), Some((10, 20)));

        status.set_display_position(None);
        assert_eq!(status.display_position(), None);
    }

    #[test]
    fn test_display_cursor_visibility() {
        let mut status = StatusLine::new();

        // Should be hidden by default
        assert!(!status.is_display_cursor_visible());

        status.set_display_cursor_visible(true);
        assert!(status.is_display_cursor_visible());

        status.set_display_cursor_visible(false);
        assert!(!status.is_display_cursor_visible());
    }
}
