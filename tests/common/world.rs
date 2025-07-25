use anyhow::Result;
use cucumber::World;
use std::collections::HashMap;
use wiremock::{Mock, MockServer, ResponseTemplate};

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
#[derive(Debug, World)]
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
        }
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
            return Ok(());
        }

        // Handle post-Ctrl+W navigation
        if key == "j" && self.mode == Mode::Normal {
            // After Ctrl+W, j moves to response pane
            self.active_pane = ActivePane::Response;
            return Ok(());
        }
        if key == "k" && self.mode == Mode::Normal {
            // After Ctrl+W, k moves to request pane
            self.active_pane = ActivePane::Request;
            return Ok(());
        }

        match (self.mode.clone(), self.active_pane.clone(), key) {
            // Normal mode navigation
            (Mode::Normal, ActivePane::Request, "h") => {
                if self.cursor_position.column > 0 {
                    self.cursor_position.column -= 1;
                }
            }
            (Mode::Normal, ActivePane::Request, "l") => {
                if let Some(line) = self.request_buffer.get(self.cursor_position.line) {
                    if self.cursor_position.column < line.len() {
                        self.cursor_position.column += 1;
                    }
                }
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
            }
            (Mode::Normal, ActivePane::Request, "0") => {
                self.cursor_position.column = 0;
            }
            (Mode::Normal, ActivePane::Request, "$") => {
                if let Some(line) = self.request_buffer.get(self.cursor_position.line) {
                    self.cursor_position.column = line.len();
                }
            }

            // Mode transitions
            (Mode::Normal, _, "i") => {
                self.mode = Mode::Insert;
            }
            (Mode::Insert, _, "Escape") => {
                self.mode = Mode::Normal;
            }
            (Mode::Normal, _, ":") => {
                self.mode = Mode::Command;
                self.command_buffer.clear();
            }
            (Mode::Command, _, "Escape") => {
                self.mode = Mode::Normal;
                self.command_buffer.clear();
            }

            // Pane switching with Tab
            (Mode::Normal, ActivePane::Request, "Tab") => {
                self.active_pane = ActivePane::Response;
            }
            (Mode::Normal, ActivePane::Response, "Tab") => {
                self.active_pane = ActivePane::Request;
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
                    }
                } else {
                    // Multiple lines - handle line breaks
                    if self.request_buffer.is_empty() {
                        self.request_buffer = lines.iter().map(|s| s.to_string()).collect();
                        self.cursor_position.line = lines.len().saturating_sub(1);
                        self.cursor_position.column = lines.last().unwrap_or(&"").len();
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
}

impl Default for BluelineWorld {
    fn default() -> Self {
        Self::new()
    }
}
