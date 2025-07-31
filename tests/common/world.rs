use super::terminal_state::TerminalState;
use anyhow::Result;
use cucumber::World;
use std::collections::HashMap;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use vte::Parser;
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Type alias for captured stdout buffer
type CapturedOutput = Arc<Mutex<Vec<u8>>>;

/// Type alias for render statistics tuple
type RenderStats = (usize, usize, usize, usize);

/// A writer that captures output for VTE parsing in tests
#[derive(Clone)]
pub struct VteWriter {
    pub captured_output: CapturedOutput,
}

impl VteWriter {
    pub fn new(captured_output: CapturedOutput) -> Self {
        Self { captured_output }
    }
}

impl Write for VteWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.captured_output.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        // Nothing to flush since we're just collecting in memory
        Ok(())
    }
}

/// Represents the current mode of the REPL
#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    Normal,
    Insert,
    Command,
}

/// Represents which pane is currently active
#[derive(Debug, Clone, PartialEq)]
pub enum ActivePane {
    Request,
    Response,
}

/// Represents cursor position in the buffer
#[derive(Debug, Clone, PartialEq)]
pub struct CursorPosition {
    pub line: usize,
    pub column: usize,
}

/// Represents the application state for testing
#[derive(World)]
#[world(init = Self::new)]
pub struct BluelineWorld {
    /// Current mode (Normal, Insert, Command)
    pub mode: Mode,

    /// Currently active pane
    pub active_pane: ActivePane,

    /// Request buffer content (lines of text)
    pub request_buffer: Vec<String>,

    /// Response buffer content (lines of text)
    pub response_buffer: Vec<String>,

    /// Current cursor position
    pub cursor_position: CursorPosition,

    /// Command buffer for command mode
    pub command_buffer: String,

    /// Last executed HTTP request
    pub last_request: Option<String>,

    /// Last HTTP response
    pub last_response: Option<String>,

    /// Last error message
    pub last_error: Option<String>,

    /// Mock HTTP server for testing
    pub mock_server: Option<MockServer>,

    /// CLI flags used when starting
    pub cli_flags: Vec<String>,

    /// Profile configuration for testing
    #[allow(dead_code)]
    pub profile_config: HashMap<String, String>,

    /// Application exit status
    pub app_exited: bool,

    /// Whether app exited without saving
    pub force_quit: bool,

    /// Flag to track if Ctrl+W was recently pressed for pane navigation
    pub ctrl_w_pressed: bool,

    /// Flag to track if first 'g' was pressed for gg navigation
    pub first_g_pressed: bool,

    /// Captured stdout bytes for terminal state reconstruction
    pub stdout_capture: CapturedOutput,

    /// VTE parser for terminal escape sequences
    pub vte_parser: Parser,

    /// Terminal renderer with VTE writer for capturing output
    pub terminal_renderer: Option<blueline::TerminalRenderer<VteWriter>>,
}

impl BluelineWorld {
    pub fn new() -> Self {
        Self {
            mode: Mode::Normal,
            active_pane: ActivePane::Request,
            request_buffer: Vec::new(),
            response_buffer: Vec::new(),
            cursor_position: CursorPosition { line: 0, column: 0 },
            command_buffer: String::new(),
            last_request: None,
            last_response: None,
            last_error: None,
            mock_server: None,
            cli_flags: Vec::new(),
            profile_config: HashMap::new(),
            app_exited: false,
            force_quit: false,
            ctrl_w_pressed: false,
            first_g_pressed: false,
            stdout_capture: Arc::new(Mutex::new(Vec::new())),
            vte_parser: Parser::new(),
            terminal_renderer: None,
        }
    }

    /// Initialize the terminal renderer with VTE writer for testing
    pub fn init_terminal_renderer(&mut self) -> Result<()> {
        let vte_writer = VteWriter::new(self.stdout_capture.clone());
        self.terminal_renderer = Some(blueline::TerminalRenderer::with_writer(vte_writer)?);
        Ok(())
    }

    /// Set up mock HTTP server for testing HTTP requests
    pub async fn setup_mock_server(&mut self) -> Result<()> {
        let server = MockServer::start().await;

        // Setup default mock responses
        Mock::given(wiremock::matchers::method("GET"))
            .and(wiremock::matchers::path("/api/users"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
                {"id": 1, "name": "John Doe"},
                {"id": 2, "name": "Jane Smith"}
            ])))
            .mount(&server)
            .await;

        Mock::given(wiremock::matchers::method("POST"))
            .and(wiremock::matchers::path("/api/users"))
            .respond_with(
                ResponseTemplate::new(201)
                    .set_body_json(serde_json::json!({"id": 3, "name": "John Doe"})),
            )
            .mount(&server)
            .await;

        Mock::given(wiremock::matchers::method("GET"))
            .and(wiremock::matchers::path("/api/status"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"status": "ok", "version": "1.0.0"})),
            )
            .mount(&server)
            .await;

        self.mock_server = Some(server);
        Ok(())
    }

    /// Get the base URL for the mock server
    #[allow(dead_code)]
    pub fn mock_server_url(&self) -> String {
        self.mock_server
            .as_ref()
            .map(|server| server.uri())
            .unwrap_or_else(|| "http://localhost:8080".to_string())
    }

    /// Simulate key press in the REPL
    pub fn press_key(&mut self, key: &str) -> Result<()> {
        // Handle Ctrl+W pane navigation specially
        if key == "Ctrl+W" {
            // Set a flag to indicate next key is for pane navigation
            self.ctrl_w_pressed = true;
            return Ok(());
        }

        // Handle post-Ctrl+W navigation
        if self.ctrl_w_pressed && key == "j" && self.mode == Mode::Normal {
            // After Ctrl+W, j moves to response pane
            self.active_pane = ActivePane::Response;
            self.ctrl_w_pressed = false; // Reset flag
            return Ok(());
        }
        if self.ctrl_w_pressed && key == "k" && self.mode == Mode::Normal {
            // After Ctrl+W, k moves to request pane
            self.active_pane = ActivePane::Request;
            self.ctrl_w_pressed = false; // Reset flag
            return Ok(());
        }

        match (self.mode.clone(), self.active_pane.clone(), key) {
            // Normal mode navigation
            (Mode::Normal, ActivePane::Request, "h") => {
                if self.cursor_position.column > 0 {
                    self.cursor_position.column -= 1;
                }
                // Always simulate cursor left movement escape sequence for user feedback
                let cursor_left = "\x1b[1D";
                self.capture_stdout(cursor_left.as_bytes());
            }
            (Mode::Normal, ActivePane::Request, "l") => {
                if let Some(line) = self.request_buffer.get(self.cursor_position.line) {
                    if self.cursor_position.column < line.len() {
                        self.cursor_position.column += 1;
                    }
                }
                // Always simulate cursor right movement escape sequence for user feedback
                let cursor_right = "\x1b[1C";
                self.capture_stdout(cursor_right.as_bytes());
            }
            (Mode::Normal, ActivePane::Request, "j") => {
                if self.cursor_position.line < self.request_buffer.len().saturating_sub(1) {
                    self.cursor_position.line += 1;
                    // Adjust column if new line is shorter
                    if let Some(line) = self.request_buffer.get(self.cursor_position.line) {
                        if self.cursor_position.column > line.len() {
                            self.cursor_position.column = line.len();
                        }
                    }
                }
                // Always simulate cursor down movement escape sequence for user feedback
                let cursor_down = "\x1b[1B";
                self.capture_stdout(cursor_down.as_bytes());
            }
            (Mode::Normal, ActivePane::Request, "k") => {
                if self.cursor_position.line > 0 {
                    self.cursor_position.line -= 1;
                    // Adjust column if new line is shorter
                    if let Some(line) = self.request_buffer.get(self.cursor_position.line) {
                        if self.cursor_position.column > line.len() {
                            self.cursor_position.column = line.len();
                        }
                    }
                }
                // Always simulate cursor up movement escape sequence for user feedback
                let cursor_up = "\x1b[1A";
                self.capture_stdout(cursor_up.as_bytes());
            }
            // Arrow keys work in all modes
            (_, ActivePane::Request, "Left") => {
                if self.cursor_position.column > 0 {
                    self.cursor_position.column -= 1;
                }
                // Always simulate cursor left movement escape sequence for user feedback
                let cursor_left = "\x1b[1D";
                self.capture_stdout(cursor_left.as_bytes());
            }
            (_, ActivePane::Request, "Right") => {
                if let Some(line) = self.request_buffer.get(self.cursor_position.line) {
                    if self.cursor_position.column < line.len() {
                        self.cursor_position.column += 1;
                    }
                }
                // Always simulate cursor right movement escape sequence for user feedback
                let cursor_right = "\x1b[1C";
                self.capture_stdout(cursor_right.as_bytes());
            }
            (_, ActivePane::Request, "Up") => {
                if self.cursor_position.line > 0 {
                    self.cursor_position.line -= 1;
                    if let Some(line) = self.request_buffer.get(self.cursor_position.line) {
                        if self.cursor_position.column > line.len() {
                            self.cursor_position.column = line.len();
                        }
                    }
                }
                // Always simulate cursor up movement escape sequence for user feedback
                let cursor_up = "\x1b[1A";
                self.capture_stdout(cursor_up.as_bytes());
            }
            (_, ActivePane::Request, "Down") => {
                if self.cursor_position.line < self.request_buffer.len().saturating_sub(1) {
                    self.cursor_position.line += 1;
                    if let Some(line) = self.request_buffer.get(self.cursor_position.line) {
                        if self.cursor_position.column > line.len() {
                            self.cursor_position.column = line.len();
                        }
                    }
                }
                // Always simulate cursor down movement escape sequence for user feedback
                let cursor_down = "\x1b[1B";
                self.capture_stdout(cursor_down.as_bytes());
            }
            // Arrow keys work in all modes
            (_, ActivePane::Response, "Left") => {
                if self.cursor_position.column > 0 {
                    self.cursor_position.column -= 1;
                }
                // Always simulate cursor left movement escape sequence for user feedback
                let cursor_left = "\x1b[1D";
                self.capture_stdout(cursor_left.as_bytes());
            }
            (_, ActivePane::Response, "Right") => {
                if let Some(line) = self.response_buffer.get(self.cursor_position.line) {
                    if self.cursor_position.column < line.len() {
                        self.cursor_position.column += 1;
                    }
                }
                // Always simulate cursor right movement escape sequence for user feedback
                let cursor_right = "\x1b[1C";
                self.capture_stdout(cursor_right.as_bytes());
            }
            (_, ActivePane::Response, "Up") => {
                if self.cursor_position.line > 0 {
                    self.cursor_position.line -= 1;
                    if let Some(line) = self.response_buffer.get(self.cursor_position.line) {
                        if self.cursor_position.column > line.len() {
                            self.cursor_position.column = line.len();
                        }
                    }
                }
                // Always simulate cursor up movement escape sequence for user feedback
                let cursor_up = "\x1b[1A";
                self.capture_stdout(cursor_up.as_bytes());
            }
            (_, ActivePane::Response, "Down") => {
                if self.cursor_position.line < self.response_buffer.len().saturating_sub(1) {
                    self.cursor_position.line += 1;
                    if let Some(line) = self.response_buffer.get(self.cursor_position.line) {
                        if self.cursor_position.column > line.len() {
                            self.cursor_position.column = line.len();
                        }
                    }
                }
                // Always simulate cursor down movement escape sequence for user feedback
                let cursor_down = "\x1b[1B";
                self.capture_stdout(cursor_down.as_bytes());
            }

            // Special navigation keys
            (Mode::Normal, ActivePane::Request, "0") => {
                self.cursor_position.column = 0;
                // Simulate cursor to beginning of line
                let cursor_home = "\x1b[1G";
                self.capture_stdout(cursor_home.as_bytes());
            }
            (Mode::Normal, ActivePane::Request, "$") => {
                if let Some(line) = self.request_buffer.get(self.cursor_position.line) {
                    self.cursor_position.column = line.len();
                    // Simulate cursor to end of line
                    let cursor_end = format!("\x1b[{}G", line.len() + 1);
                    self.capture_stdout(cursor_end.as_bytes());
                } else {
                    // If no line content, still emit escape sequence for cursor positioning
                    let cursor_end = "\x1b[1G"; // Move to column 1
                    self.capture_stdout(cursor_end.as_bytes());
                }
            }

            // Mode transitions
            (Mode::Normal, _, "i") => {
                self.mode = Mode::Insert;
                // Simulate cursor style change to blinking bar for insert mode
                let cursor_change = "\x1b[5 q"; // Blinking bar cursor
                self.capture_stdout(cursor_change.as_bytes());
            }
            (Mode::Insert, _, "Escape") => {
                self.mode = Mode::Normal;
                // Simulate cursor style change to steady block for normal mode
                let cursor_change = "\x1b[2 q"; // Steady block cursor
                self.capture_stdout(cursor_change.as_bytes());
            }
            (Mode::Normal, _, ":") => {
                self.mode = Mode::Command;
                self.command_buffer.clear();
            }
            (Mode::Command, _, "Escape") => {
                self.mode = Mode::Normal;
                self.command_buffer.clear();
                // Simulate cursor style change to steady block for normal mode
                let cursor_change = "\x1b[2 q"; // Steady block cursor
                self.capture_stdout(cursor_change.as_bytes());
            }

            // Pane switching with Tab
            (Mode::Normal, ActivePane::Request, "Tab") => {
                self.active_pane = ActivePane::Response;
            }
            (Mode::Normal, ActivePane::Response, "Tab") => {
                self.active_pane = ActivePane::Request;
            }

            // Advanced cursor movements
            (Mode::Normal, ActivePane::Request, "Ctrl+U") => {
                // Scroll up by half page (simulate by moving cursor up multiple lines)
                let half_page = 12;
                for _ in 0..half_page {
                    if self.cursor_position.line > 0 {
                        self.cursor_position.line -= 1;
                    }
                }
                // Simulate half page up movement
                let half_page_up = format!("\x1b[{}A", half_page);
                self.capture_stdout(half_page_up.as_bytes());
            }
            (Mode::Normal, ActivePane::Request, "Ctrl+D") => {
                // Scroll down by half page
                let half_page = 12;
                let max_line = self.request_buffer.len().saturating_sub(1);
                for _ in 0..half_page {
                    if self.cursor_position.line < max_line {
                        self.cursor_position.line += 1;
                    }
                }
                // Simulate half page down movement
                let half_page_down = format!("\x1b[{}B", half_page);
                self.capture_stdout(half_page_down.as_bytes());
            }
            (Mode::Normal, ActivePane::Request, "Ctrl+F")
            | (Mode::Normal, ActivePane::Request, "Page Down") => {
                // Scroll down by full page
                let full_page = 24;
                let max_line = self.request_buffer.len().saturating_sub(1);
                for _ in 0..full_page {
                    if self.cursor_position.line < max_line {
                        self.cursor_position.line += 1;
                    }
                }
                // Simulate full page down movement
                let full_page_down = format!("\x1b[{}B", full_page);
                self.capture_stdout(full_page_down.as_bytes());
            }
            (Mode::Normal, ActivePane::Request, "Ctrl+B")
            | (Mode::Normal, ActivePane::Request, "Page Up") => {
                // Scroll up by full page
                let full_page = 24;
                for _ in 0..full_page {
                    if self.cursor_position.line > 0 {
                        self.cursor_position.line -= 1;
                    }
                }
                // Simulate full page up movement
                let full_page_up = format!("\x1b[{}A", full_page);
                self.capture_stdout(full_page_up.as_bytes());
            }
            (Mode::Normal, ActivePane::Request, "g") => {
                if self.first_g_pressed {
                    // Second 'g' - go to top (gg command)
                    self.cursor_position.line = 0;
                    self.cursor_position.column = 0;
                    let cursor_pos = "\x1b[1;1H"; // Move to top-left
                    self.capture_stdout(cursor_pos.as_bytes());
                    self.first_g_pressed = false;
                } else {
                    // First 'g' - set flag and wait for second
                    self.first_g_pressed = true;
                    // Don't emit cursor movement yet
                }
            }
            (Mode::Normal, ActivePane::Request, "G") => {
                // Go to last line
                let last_line = self.request_buffer.len();
                self.cursor_position.line = last_line.saturating_sub(1);
                self.cursor_position.column = 0;
                // Simulate cursor to last line
                let cursor_last = format!("\x1b[{};1H", last_line);
                self.capture_stdout(cursor_last.as_bytes());
            }

            // Enter key handling
            (Mode::Insert, _, "Enter") => {
                // In insert mode, Enter creates a new line
                if self.request_buffer.is_empty() {
                    self.request_buffer.push(String::new());
                    self.request_buffer.push(String::new());
                } else {
                    // Split current line at cursor position
                    let current_line = self.cursor_position.line;
                    if let Some(line) = self.request_buffer.get_mut(current_line) {
                        let remainder = line.split_off(self.cursor_position.column);
                        self.request_buffer.insert(current_line + 1, remainder);
                    } else {
                        self.request_buffer.push(String::new());
                    }
                }
                // Move cursor to beginning of new line
                self.cursor_position.line += 1;
                self.cursor_position.column = 0;

                // Simulate newline in terminal
                self.capture_stdout(b"\r\n");
            }
            (Mode::Normal, _, "Enter") => {
                // In normal mode, Enter just moves cursor down
                if self.cursor_position.line < self.request_buffer.len().saturating_sub(1) {
                    self.cursor_position.line += 1;
                    self.cursor_position.column = 0;
                    let cursor_down = "\x1b[1B\x1b[1G"; // Down one line, column 1
                    self.capture_stdout(cursor_down.as_bytes());
                }
            }

            // Command execution
            (Mode::Command, _, "Enter") => {
                self.execute_command()?;
                self.mode = Mode::Normal;
            }

            _ => {
                // For unhandled key combinations, do nothing
            }
        }
        Ok(())
    }

    /// Type text into the current buffer
    pub fn type_text(&mut self, text: &str) -> Result<()> {
        match self.mode {
            Mode::Insert => {
                // Handle multiline text input
                let lines: Vec<&str> = text.split('\n').collect();

                if lines.len() == 1 {
                    // Single line - insert at cursor position
                    if self.request_buffer.is_empty() {
                        self.request_buffer.push(String::new());
                    }

                    if let Some(line) = self.request_buffer.get_mut(self.cursor_position.line) {
                        line.insert_str(self.cursor_position.column, text);
                        self.cursor_position.column += text.len();

                        // Simulate text appearing on terminal screen
                        self.capture_stdout(text.as_bytes());
                    }
                } else {
                    // Multiple lines - handle line breaks
                    if self.request_buffer.is_empty() {
                        self.request_buffer = lines.iter().map(|s| s.to_string()).collect();
                        self.cursor_position.line = lines.len().saturating_sub(1);
                        self.cursor_position.column = lines.last().unwrap_or(&"").len();

                        // Simulate multiline text appearing on terminal screen
                        self.capture_stdout(text.as_bytes());
                    } else {
                        // Insert multiline text at current position
                        for (i, line_text) in lines.iter().enumerate() {
                            if i == 0 {
                                // Insert first line at current position
                                if let Some(line) =
                                    self.request_buffer.get_mut(self.cursor_position.line)
                                {
                                    line.insert_str(self.cursor_position.column, line_text);
                                }
                            } else {
                                // Add new lines
                                self.request_buffer
                                    .insert(self.cursor_position.line + i, line_text.to_string());
                            }
                        }
                        self.cursor_position.line += lines.len().saturating_sub(1);
                        self.cursor_position.column = lines.last().unwrap_or(&"").len();

                        // Simulate text appearing on terminal screen
                        self.capture_stdout(text.as_bytes());
                    }
                }
            }
            Mode::Command => {
                self.command_buffer.push_str(text);
            }
            _ => {
                return Err(anyhow::anyhow!("Cannot type text in {:?} mode", self.mode));
            }
        }
        Ok(())
    }

    /// Execute a command from command mode
    fn execute_command(&mut self) -> Result<()> {
        match self.command_buffer.as_str() {
            "x" => {
                // Execute HTTP request
                self.execute_http_request()?;
            }
            "q" => {
                // Quit application
                self.app_exited = true;
            }
            "q!" => {
                // Force quit without saving
                self.app_exited = true;
                self.force_quit = true;
            }
            unknown => {
                // Unknown command
                self.last_error = Some(format!("Unknown command: {}", unknown));
            }
        }
        self.command_buffer.clear();
        Ok(())
    }

    /// Execute HTTP request based on current request buffer
    fn execute_http_request(&mut self) -> Result<()> {
        if self.request_buffer.is_empty() {
            return Err(anyhow::anyhow!("No request to execute"));
        }

        let request_text = self.request_buffer.join("\n");
        self.last_request = Some(request_text.clone());

        // Parse the request (simplified)
        let lines: Vec<&str> = request_text.lines().collect();
        if let Some(first_line) = lines.first() {
            let parts: Vec<&str> = first_line.split_whitespace().collect();
            if parts.len() >= 2 {
                let method = parts[0];
                let path = parts[1];

                // Simulate HTTP response based on mock server
                let response = match (method, path) {
                    ("GET", "/api/users") => serde_json::json!([
                        {"id": 1, "name": "John Doe"},
                        {"id": 2, "name": "Jane Smith"}
                    ])
                    .to_string(),
                    ("POST", "/api/users") => {
                        serde_json::json!({"id": 3, "name": "John Doe"}).to_string()
                    }
                    ("GET", "/api/status") => {
                        serde_json::json!({"status": "ok", "version": "1.0.0"}).to_string()
                    }
                    _ => serde_json::json!({"error": "Not found"}).to_string(),
                };

                self.last_response = Some(response.clone());
                self.response_buffer = response.lines().map(|s| s.to_string()).collect();
            }
        }

        Ok(())
    }

    /// Set request buffer content from a multiline string
    pub fn set_request_buffer(&mut self, content: &str) {
        if content.trim().is_empty() {
            self.request_buffer.clear();
        } else {
            self.request_buffer = content.lines().map(|s| s.to_string()).collect();
        }
        self.cursor_position = CursorPosition { line: 0, column: 0 };
    }

    /// Get request buffer as a single string
    #[allow(dead_code)]
    pub fn get_request_buffer(&self) -> String {
        self.request_buffer.join("\n")
    }

    /// Set up response pane with mock response
    pub fn setup_response_pane(&mut self) {
        let mock_response = serde_json::json!([
            {"id": 1, "name": "John Doe"},
            {"id": 2, "name": "Jane Smith"}
        ])
        .to_string();

        self.response_buffer = mock_response.lines().map(|s| s.to_string()).collect();
        self.last_response = Some(mock_response);
    }

    /// Capture stdout bytes (called by mock renderer or stdout interceptor)
    pub fn capture_stdout(&mut self, bytes: &[u8]) {
        self.stdout_capture.lock().unwrap().extend_from_slice(bytes);
    }

    /// Get the reconstructed terminal state from captured stdout
    pub fn get_terminal_state(&mut self) -> TerminalState {
        let captured_bytes = self.stdout_capture.lock().unwrap().clone();
        let mut terminal_state = TerminalState::new(80, 24);

        // Parse all the captured escape sequences and build terminal state
        for &byte in &captured_bytes {
            self.vte_parser.advance(&mut terminal_state, byte);
        }

        terminal_state
    }

    /// Clear captured stdout data
    pub fn clear_terminal_capture(&mut self) {
        self.stdout_capture.lock().unwrap().clear();
    }

    /// Get terminal rendering statistics
    pub fn get_render_stats(&mut self) -> RenderStats {
        let terminal_state = self.get_terminal_state();
        (
            terminal_state.full_redraws,
            terminal_state.partial_redraws,
            terminal_state.cursor_updates,
            terminal_state.clear_screen_count,
        )
    }
}

impl Default for BluelineWorld {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for BluelineWorld {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BluelineWorld")
            .field("mode", &self.mode)
            .field("active_pane", &self.active_pane)
            .field("request_buffer", &self.request_buffer)
            .field("response_buffer", &self.response_buffer)
            .field("cursor_position", &self.cursor_position)
            .field("command_buffer", &self.command_buffer)
            .field("last_request", &self.last_request)
            .field("last_response", &self.last_response)
            .field("last_error", &self.last_error)
            .field("cli_flags", &self.cli_flags)
            .field("app_exited", &self.app_exited)
            .field("force_quit", &self.force_quit)
            .field("ctrl_w_pressed", &self.ctrl_w_pressed)
            .field("first_g_pressed", &self.first_g_pressed)
            .field("stdout_capture", &"Arc<Mutex<Vec<u8>>>")
            .field("vte_parser", &"Parser")
            .field("terminal_renderer", &"Option<TerminalRenderer<VteWriter>>")
            .finish()
    }
}
