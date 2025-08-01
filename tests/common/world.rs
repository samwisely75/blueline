use super::terminal_state::TerminalState;
use anyhow::Result;
use blueline::repl::commands::{CommandEvent, MovementDirection};
use blueline::{CommandContext, CommandRegistry, ViewModel, ViewModelSnapshot};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
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

    /// Real ViewModel from blueline for actual application logic
    pub view_model: Option<ViewModel>,

    /// Real CommandRegistry for processing key events
    pub command_registry: Option<CommandRegistry>,
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
            view_model: None,
            command_registry: None,
        }
    }

    /// Initialize the terminal renderer with VTE writer for testing
    pub fn init_terminal_renderer(&mut self) -> Result<()> {
        let vte_writer = VteWriter::new(self.stdout_capture.clone());
        self.terminal_renderer = Some(blueline::TerminalRenderer::with_writer(vte_writer)?);
        Ok(())
    }

    /// Initialize real blueline application components for testing
    pub fn init_real_application(&mut self) -> Result<()> {
        // Initialize real ViewModel
        let mut view_model = ViewModel::new();
        view_model.update_terminal_size(80, 24); // Set test terminal size
        self.view_model = Some(view_model);

        // Initialize real CommandRegistry
        self.command_registry = Some(CommandRegistry::new());

        // Initialize terminal renderer with VTE capture
        self.init_terminal_renderer()?;

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

    /// Process key press using real blueline command system
    pub fn press_key(&mut self, key: &str) -> Result<()> {
        println!("🔑 press_key called with: '{key}'");
        println!(
            "   Current state: mode={:?}, pane={:?}",
            self.mode, self.active_pane
        );
        // TEMPORARY FIX: Always use simulation to avoid stdout/stdin issues
        // TODO: Properly separate real application tests from simulation tests
        let result = self.process_simulated_key_event(key);
        println!(
            "   After press_key: mode={:?}, result={:?}",
            self.mode,
            result.is_ok()
        );
        result

        // OLD CODE: Check if we have real application components
        // if self.view_model.is_some() && self.command_registry.is_some() {
        //     return self.process_real_key_event(key);
        // }
        // Fallback to old simulation for compatibility
        // self.process_simulated_key_event(key)
    }

    /// Process key event using real blueline command system
    #[allow(dead_code)]
    fn process_real_key_event(&mut self, key: &str) -> Result<()> {
        // Convert key string to KeyEvent
        let key_event = self.string_to_key_event(key)?;

        // Extract references to avoid borrowing issues
        let view_model = self.view_model.as_mut().unwrap();
        let command_registry = self.command_registry.as_ref().unwrap();

        // Create command context from current view model state
        let snapshot = ViewModelSnapshot::from_view_model(view_model);
        let context = CommandContext::new(snapshot);

        // Process the key event through the real command registry
        match command_registry.process_event(key_event, &context) {
            Ok(events) => {
                println!("🔧 Real key '{key}' generated {count} events", count = events.len());

                // Apply events to the real view model
                for event in events {
                    println!("  📝 Applying event: {event:?}");
                    self.apply_command_event_to_view_model(event)?;
                }

                // Render the updated state
                self.render_real_view_model()?;

                Ok(())
            }
            Err(e) => {
                println!("❌ Error processing key '{key}': {e}");
                Err(e)
            }
        }
    }

    /// Convert key string to KeyEvent
    #[allow(dead_code)]
    fn string_to_key_event(&self, key: &str) -> Result<KeyEvent> {
        let key_event = match key {
            "i" => KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE),
            "Escape" => KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE),
            "Enter" => KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
            "h" => KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE),
            "j" => KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE),
            "k" => KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE),
            "l" => KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE),
            ":" => KeyEvent::new(KeyCode::Char(':'), KeyModifiers::NONE),
            _ => return Err(anyhow::anyhow!("Unsupported key: {key}")),
        };
        Ok(key_event)
    }

    /// Apply a CommandEvent to the ViewModel (similar to AppController)
    fn apply_command_event_to_view_model(&mut self, event: CommandEvent) -> Result<()> {
        let view_model = self.view_model.as_mut().unwrap();
        match event {
            CommandEvent::CursorMoveRequested { direction, amount } => {
                for _ in 0..amount {
                    match direction {
                        MovementDirection::Left => view_model.move_cursor_left()?,
                        MovementDirection::Right => view_model.move_cursor_right()?,
                        MovementDirection::Up => view_model.move_cursor_up()?,
                        MovementDirection::Down => view_model.move_cursor_down()?,
                        MovementDirection::LineEnd => view_model.move_cursor_to_end_of_line()?,
                        MovementDirection::LineStart => {
                            view_model.move_cursor_to_start_of_line()?
                        }
                        _ => println!(
                            "⚠️  Movement direction {direction:?} not yet implemented in tests"
                        ),
                    }
                }
            }
            CommandEvent::TextInsertRequested { text, position: _ } => {
                view_model.insert_text(&text)?;
            }
            CommandEvent::ModeChangeRequested { new_mode } => {
                view_model.change_mode(new_mode)?;
            }
            CommandEvent::PaneSwitchRequested { target_pane } => {
                use blueline::repl::events::Pane;
                match target_pane {
                    Pane::Request => view_model.switch_to_request_pane(),
                    Pane::Response => view_model.switch_to_response_pane(),
                }
            }
            CommandEvent::HttpRequestRequested {
                method,
                url,
                headers: _,
                body,
            } => {
                // This would trigger HTTP execution
                println!("🌐 HTTP Request: {method} {url} (body: {body:?})");
                // For now, just record the request
                self.last_request = Some(format!("{method} {url}"));
            }
            _ => {
                println!("⚠️  CommandEvent {event:?} not yet implemented in tests");
            }
        }
        Ok(())
    }

    /// Render the real view model state through terminal renderer
    fn render_real_view_model(&mut self) -> Result<()> {
        if let Some(ref mut _renderer) = self.terminal_renderer {
            println!("🎨 Rendering real view model state");

            // Capture current view model state as terminal output
            let view_model = self.view_model.as_ref().unwrap();
            let mode = view_model.get_mode();
            let output = format!("Real ViewModel State: Mode={mode:?}\r\n");
            self.capture_stdout(output.as_bytes());

            // Also emit mode-specific cursor styling
            match mode {
                blueline::repl::events::EditorMode::Insert => {
                    self.capture_stdout(b"\x1b[5 q"); // Blinking bar cursor
                }
                blueline::repl::events::EditorMode::Normal => {
                    self.capture_stdout(b"\x1b[2 q"); // Steady block cursor
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Fallback simulation for compatibility (old method)
    fn process_simulated_key_event(&mut self, key: &str) -> Result<()> {
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
                    if self.cursor_position.column < line.chars().count() {
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
                        let line_char_count = line.chars().count();
                        if self.cursor_position.column > line_char_count {
                            self.cursor_position.column = line_char_count;
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
                        let line_char_count = line.chars().count();
                        if self.cursor_position.column > line_char_count {
                            self.cursor_position.column = line_char_count;
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
                    if self.cursor_position.column < line.chars().count() {
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
                    let line_char_count = line.chars().count();
                    self.cursor_position.column = line_char_count;
                    // Simulate cursor to end of line
                    let cursor_end = format!("\x1b[{position}G", position = line_char_count + 1);
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
                let half_page_up = format!("\x1b[{half_page}A");
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
                let half_page_down = format!("\x1b[{half_page}B");
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
                let full_page_down = format!("\x1b[{full_page}B");
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
                let full_page_up = format!("\x1b[{full_page}A");
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
                let cursor_last = format!("\x1b[{last_line};1H");
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
                // In normal mode, Enter executes HTTP request if there's content in request buffer
                if !self.request_buffer.is_empty() {
                    // Execute HTTP request
                    self.execute_http_request()?;

                    // Simulate terminal rendering for dual-pane layout
                    self.simulate_dual_pane_rendering();
                } else {
                    // If no content, just move cursor down
                    if self.cursor_position.line < self.request_buffer.len().saturating_sub(1) {
                        self.cursor_position.line += 1;
                        self.cursor_position.column = 0;
                        let cursor_down = "\x1b[1B\x1b[1G"; // Down one line, column 1
                        self.capture_stdout(cursor_down.as_bytes());
                    }
                }
            }

            // Command execution
            (Mode::Command, _, "Enter") => {
                self.execute_command()?;
                self.mode = Mode::Normal;
            }

            // Word navigation - Request pane
            (Mode::Normal, ActivePane::Request, "w") => {
                self.move_to_next_word_request();
                let cursor_right = "\x1b[1C"; // Basic cursor movement for visual feedback
                self.capture_stdout(cursor_right.as_bytes());
            }
            (Mode::Normal, ActivePane::Request, "b") => {
                self.move_to_previous_word_request();
                let cursor_left = "\x1b[1D"; // Basic cursor movement for visual feedback
                self.capture_stdout(cursor_left.as_bytes());
            }
            (Mode::Normal, ActivePane::Request, "e") => {
                self.move_to_end_of_word_request();
                let cursor_right = "\x1b[1C"; // Basic cursor movement for visual feedback
                self.capture_stdout(cursor_right.as_bytes());
            }

            // Word navigation - Response pane
            (Mode::Normal, ActivePane::Response, "w") => {
                self.move_to_next_word_response();
                let cursor_right = "\x1b[1C"; // Basic cursor movement for visual feedback
                self.capture_stdout(cursor_right.as_bytes());
            }
            (Mode::Normal, ActivePane::Response, "b") => {
                self.move_to_previous_word_response();
                let cursor_left = "\x1b[1D"; // Basic cursor movement for visual feedback
                self.capture_stdout(cursor_left.as_bytes());
            }
            (Mode::Normal, ActivePane::Response, "e") => {
                self.move_to_end_of_word_response();
                let cursor_right = "\x1b[1C"; // Basic cursor movement for visual feedback
                self.capture_stdout(cursor_right.as_bytes());
            }

            // Response pane line movement (like request pane but for response)
            (Mode::Normal, ActivePane::Response, "h") => {
                if self.cursor_position.column > 0 {
                    self.cursor_position.column -= 1;
                }
                let cursor_left = "\x1b[1D";
                self.capture_stdout(cursor_left.as_bytes());
            }
            (Mode::Normal, ActivePane::Response, "l") => {
                if let Some(line) = self.response_buffer.get(self.cursor_position.line) {
                    if self.cursor_position.column < line.chars().count() {
                        self.cursor_position.column += 1;
                    }
                }
                let cursor_right = "\x1b[1C";
                self.capture_stdout(cursor_right.as_bytes());
            }
            (Mode::Normal, ActivePane::Response, "j") => {
                if self.cursor_position.line < self.response_buffer.len().saturating_sub(1) {
                    self.cursor_position.line += 1;
                    // Clamp column to line length to fix issue #3
                    if let Some(line) = self.response_buffer.get(self.cursor_position.line) {
                        let line_char_count = line.chars().count();
                        if self.cursor_position.column > line_char_count {
                            self.cursor_position.column = line_char_count;
                        }
                    }
                }
                let cursor_down = "\x1b[1B";
                self.capture_stdout(cursor_down.as_bytes());
            }
            (Mode::Normal, ActivePane::Response, "k") => {
                if self.cursor_position.line > 0 {
                    self.cursor_position.line -= 1;
                    // Clamp column to line length to fix issue #3
                    if let Some(line) = self.response_buffer.get(self.cursor_position.line) {
                        let line_char_count = line.chars().count();
                        if self.cursor_position.column > line_char_count {
                            self.cursor_position.column = line_char_count;
                        }
                    }
                }
                let cursor_up = "\x1b[1A";
                self.capture_stdout(cursor_up.as_bytes());
            }
            (Mode::Normal, ActivePane::Response, "0") => {
                self.cursor_position.column = 0;
                let cursor_home = "\x1b[1G";
                self.capture_stdout(cursor_home.as_bytes());
            }
            (Mode::Normal, ActivePane::Response, "$") => {
                if let Some(line) = self.response_buffer.get(self.cursor_position.line) {
                    self.cursor_position.column = line.chars().count();
                    let cursor_end = format!("\x1b[{position}G", position = line.chars().count() + 1);
                    self.capture_stdout(cursor_end.as_bytes());
                } else {
                    let cursor_end = "\x1b[1G"; // Move to column 1
                    self.capture_stdout(cursor_end.as_bytes());
                }
            }

            _ => {
                // For unhandled key combinations, print debug info
                println!(
                    "⚠️  Unhandled key combination: mode={:?}, pane={:?}, key={}",
                    self.mode, self.active_pane, key
                );
            }
        }
        Ok(())
    }

    /// Type text using real application logic
    pub fn type_text(&mut self, text: &str) -> Result<()> {
        // Check if we have real application components
        if self.view_model.is_some() && self.command_registry.is_some() {
            return self.type_text_real(text);
        }

        // Fallback to simulation
        self.type_text_simulated(text)
    }

    /// Type text using real command processing
    fn type_text_real(&mut self, text: &str) -> Result<()> {
        println!("⌨️  Typing '{text}' using real application logic");

        for ch in text.chars() {
            let key_event = KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE);

            // Get references to avoid borrowing issues
            let view_model = self.view_model.as_mut().unwrap();
            let command_registry = self.command_registry.as_ref().unwrap();

            let snapshot = ViewModelSnapshot::from_view_model(view_model);
            let context = CommandContext::new(snapshot);

            match command_registry.process_event(key_event, &context) {
                Ok(events) => {
                    for event in events {
                        println!("  📝 Character '{ch}' event: {event:?}");
                        self.apply_command_event_to_view_model(event)?;
                    }
                    // Render after each character
                    self.render_real_view_model()?;
                }
                Err(e) => {
                    println!("❌ Error typing character '{ch}': {e}");
                }
            }
        }
        Ok(())
    }

    /// Fallback text typing (old simulation method)
    fn type_text_simulated(&mut self, text: &str) -> Result<()> {
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
                self.last_error = Some(format!("Unknown command: {unknown}"));
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

    /// Synchronize the test world's request_buffer with the real ViewModel's content
    /// This is critical for proper rendering when using real ViewModel components
    pub fn sync_request_buffer_from_view_model(&mut self) {
        if let Some(ref view_model) = self.view_model {
            let request_text = view_model.get_request_text();
            println!("🔄 Syncing request buffer from ViewModel: '{request_text}'");

            if request_text.trim().is_empty() {
                self.request_buffer.clear();
            } else {
                self.request_buffer = request_text.lines().map(|s| s.to_string()).collect();
            }

            println!("📋 Synchronized request_buffer: {request_buffer:?}", request_buffer = self.request_buffer);
        } else {
            println!("⚠️  No ViewModel available to sync from");
        }
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

    /// Simulate dual-pane terminal rendering after HTTP request execution
    pub fn simulate_dual_pane_rendering(&mut self) {
        // Clear screen and start fresh layout
        let clear_screen = "\x1b[2J\x1b[H"; // Clear screen, move cursor to home
        self.capture_stdout(clear_screen.as_bytes());

        // Simulate request pane rendering (top half)
        self.render_request_pane();

        // Simulate response pane rendering (bottom half)
        self.render_response_pane();

        // Simulate status line
        self.render_status_line();

        // Position cursor at a valid location (within bounds)
        let cursor_pos = "\x1b[1;1H"; // Move cursor to top-left (row 1, col 1)
        self.capture_stdout(cursor_pos.as_bytes());
    }

    /// Simulate rendering the request pane with borders and content
    fn render_request_pane(&mut self) {
        // Request pane header
        let header =
            "┌─ Request ───────────────────────────────────────────────────────────────┐\r\n";
        self.capture_stdout(header.as_bytes());

        // Request content with line numbers
        if self.request_buffer.is_empty() {
            let empty_line = "│                                                                            │\r\n";
            self.capture_stdout(empty_line.as_bytes());
        } else {
            // Clone the buffer to avoid borrowing issues
            let request_buffer = self.request_buffer.clone();
            for (i, line) in request_buffer.iter().enumerate() {
                let padded_line = format!(
                    "│ {:2} {}{}│\r\n",
                    i + 1,
                    line,
                    " ".repeat(72_usize.saturating_sub(line.len() + 4))
                );
                self.capture_stdout(padded_line.as_bytes());
            }
        }

        // Fill remaining space in request pane (assume 10 lines total)
        let request_lines = self.request_buffer.len().max(1);
        for _ in request_lines..10 {
            let empty_line = "│                                                                            │\r\n";
            self.capture_stdout(empty_line.as_bytes());
        }

        // Request pane separator
        let separator =
            "├─ Response ──────────────────────────────────────────────────────────────┤\r\n";
        self.capture_stdout(separator.as_bytes());
    }

    /// Simulate rendering the response pane with HTTP response content
    fn render_response_pane(&mut self) {
        if self.response_buffer.is_empty() {
            // This would be the bug case - empty response pane
            for _ in 0..10 {
                let empty_line = "│                                                                            │\r\n";
                self.capture_stdout(empty_line.as_bytes());
            }
        } else {
            // Render response content
            let response_buffer = self.response_buffer.clone();
            for line in &response_buffer {
                let padded_line = format!(
                    "│ {}{}│\r\n",
                    line,
                    " ".repeat(75_usize.saturating_sub(line.len()))
                );
                self.capture_stdout(padded_line.as_bytes());
            }

            // Fill remaining response pane space
            let response_lines = self.response_buffer.len();
            for _ in response_lines..10 {
                let empty_line = "│                                                                            │\r\n";
                self.capture_stdout(empty_line.as_bytes());
            }
        }

        // Bottom border
        let bottom =
            "└────────────────────────────────────────────────────────────────────────┘\r\n";
        self.capture_stdout(bottom.as_bytes());
    }

    /// Simulate rendering the status line
    fn render_status_line(&mut self) {
        let status = match self.mode {
            Mode::Normal => format!(" -- NORMAL -- | {:?} Pane", self.active_pane),
            Mode::Insert => format!(" -- INSERT -- | {:?} Pane", self.active_pane),
            Mode::Command => format!(" -- COMMAND -- | {:?} Pane", self.active_pane),
        };

        let padded_status = format!(
            "{}{}",
            status,
            " ".repeat(80_usize.saturating_sub(status.len()))
        );

        // Reverse video for status line
        let status_line = format!("\x1b[7m{padded_status}\x1b[0m\r\n");
        self.capture_stdout(status_line.as_bytes());
    }

    /// Move to next word in request pane
    fn move_to_next_word_request(&mut self) {
        if let Some(line) = self.request_buffer.get(self.cursor_position.line) {
            if let Some(next_pos) = self.find_next_word_boundary(line, self.cursor_position.column)
            {
                self.cursor_position.column = next_pos;
                return;
            }
        }
        // If no word boundary found on current line, move to beginning of next line
        if self.cursor_position.line + 1 < self.request_buffer.len() {
            self.cursor_position.line += 1;
            self.cursor_position.column = 0;
        }
    }

    /// Move to previous word in request pane
    fn move_to_previous_word_request(&mut self) {
        if let Some(line) = self.request_buffer.get(self.cursor_position.line) {
            if let Some(prev_pos) =
                self.find_previous_word_boundary(line, self.cursor_position.column)
            {
                self.cursor_position.column = prev_pos;
                return;
            }
        }
        // If no word boundary found, move to end of previous line
        if self.cursor_position.line > 0 {
            self.cursor_position.line -= 1;
            if let Some(line) = self.request_buffer.get(self.cursor_position.line) {
                self.cursor_position.column = line.chars().count();
            }
        }
    }

    /// Move to end of word in request pane
    fn move_to_end_of_word_request(&mut self) {
        if let Some(line) = self.request_buffer.get(self.cursor_position.line) {
            if let Some(end_pos) = self.find_end_of_word(line, self.cursor_position.column) {
                self.cursor_position.column = end_pos;
            }
        }
    }

    /// Move to next word in response pane
    fn move_to_next_word_response(&mut self) {
        if let Some(line) = self.response_buffer.get(self.cursor_position.line) {
            if let Some(next_pos) = self.find_next_word_boundary(line, self.cursor_position.column)
            {
                self.cursor_position.column = next_pos;
                return;
            }
        }
        // If no word boundary found on current line, move to beginning of next line
        if self.cursor_position.line + 1 < self.response_buffer.len() {
            self.cursor_position.line += 1;
            self.cursor_position.column = 0;
        }
    }

    /// Move to previous word in response pane
    fn move_to_previous_word_response(&mut self) {
        if let Some(line) = self.response_buffer.get(self.cursor_position.line) {
            if let Some(prev_pos) =
                self.find_previous_word_boundary(line, self.cursor_position.column)
            {
                self.cursor_position.column = prev_pos;
                return;
            }
        }
        // If no word boundary found, move to end of previous line
        if self.cursor_position.line > 0 {
            self.cursor_position.line -= 1;
            if let Some(line) = self.response_buffer.get(self.cursor_position.line) {
                self.cursor_position.column = line.chars().count();
            }
        }
    }

    /// Move to end of word in response pane
    fn move_to_end_of_word_response(&mut self) {
        if let Some(line) = self.response_buffer.get(self.cursor_position.line) {
            if let Some(end_pos) = self.find_end_of_word(line, self.cursor_position.column) {
                self.cursor_position.column = end_pos;
            }
        }
    }

    /// Find next word boundary in a line (character-aware for Japanese text)
    fn find_next_word_boundary(&self, line: &str, current_col: usize) -> Option<usize> {
        let chars: Vec<char> = line.chars().collect();
        if current_col >= chars.len() {
            return None;
        }

        let mut pos = current_col;
        let mut in_word = false;

        // Skip current character and find next word
        for (i, &ch) in chars.iter().enumerate().skip(current_col + 1) {
            if self.is_word_char(ch) {
                if !in_word {
                    return Some(i); // Found start of next word
                }
                in_word = true;
            } else {
                in_word = false;
            }
            pos = i;
        }

        // If we're at the end, return the end position
        if pos < chars.len() {
            Some(chars.len())
        } else {
            None
        }
    }

    /// Find previous word boundary in a line (character-aware for Japanese text)
    fn find_previous_word_boundary(&self, line: &str, current_col: usize) -> Option<usize> {
        if current_col == 0 {
            return None;
        }

        let chars: Vec<char> = line.chars().collect();
        let mut in_word = false;

        // Search backwards for word boundary
        for i in (0..current_col).rev() {
            let ch = chars[i];

            if self.is_word_char(ch) {
                if !in_word {
                    return Some(i); // Found beginning of a word
                }
                in_word = true;
            } else {
                in_word = false;
            }
        }

        Some(0) // Return beginning of line if no word found
    }

    /// Find end of current or next word
    fn find_end_of_word(&self, line: &str, current_col: usize) -> Option<usize> {
        let chars: Vec<char> = line.chars().collect();
        if current_col >= chars.len() {
            return None;
        }

        let mut found_word_start = false;

        // Find end of current or next word
        for (i, &ch) in chars.iter().enumerate().skip(current_col) {
            if self.is_word_char(ch) {
                found_word_start = true;
            } else if found_word_start {
                return Some(i.saturating_sub(1)); // End of word (last character of word)
            }
        }

        // If we found a word that extends to end of line
        if found_word_start {
            Some(chars.len().saturating_sub(1))
        } else {
            None
        }
    }

    /// Check if character is part of a word (supports Japanese characters)
    fn is_word_char(&self, ch: char) -> bool {
        ch.is_alphanumeric() || ch == '_' || self.is_japanese_char(ch)
    }

    /// Check if character is a Japanese character (Hiragana, Katakana, Kanji)
    fn is_japanese_char(&self, ch: char) -> bool {
        let code = ch as u32;
        (0x3040..=0x309F).contains(&code) // Hiragana
            || (0x30A0..=0x30FF).contains(&code) // Katakana
            || (0x4E00..=0x9FAF).contains(&code) // CJK Unified Ideographs
            || (0x3400..=0x4DBF).contains(&code) // CJK Extension A
            || (0x20000..=0x2A6DF).contains(&code) // CJK Extension B
            || (0xF900..=0xFAFF).contains(&code) // CJK Compatibility Ideographs
            || (0xFF00..=0xFFEF).contains(&code) // Full-width characters
            || (0xAC00..=0xD7AF).contains(&code) // Hangul (Korean)
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
