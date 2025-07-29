use anyhow::Result;
use cucumber::World;
use std::collections::HashMap;
use wiremock::{Mock, MockServer, ResponseTemplate};

// Import real application components
use blueline::repl::{
    commands::{CommandContext, CommandEvent, CommandRegistry, ViewModelSnapshot},
    events::{EditorMode, Pane, SimpleEventBus},
    view_models::ViewModel,
};

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
    /// Real ViewModel instance
    pub view_model: ViewModel,

    /// Real CommandRegistry instance
    pub command_registry: CommandRegistry,

    /// Current mode (for compatibility with existing steps)
    pub mode: Mode,

    /// Currently active pane (for compatibility)
    pub active_pane: ActivePane,

    /// Request buffer content (for compatibility)
    pub request_buffer: Vec<String>,

    /// Response buffer content (for compatibility)
    pub response_buffer: Vec<String>,

    /// Current cursor position (for compatibility)
    pub cursor_position: CursorPosition,

    /// Command buffer for command mode (for compatibility)
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

impl std::fmt::Debug for BluelineWorld {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BluelineWorld")
            .field("mode", &self.mode)
            .field("active_pane", &self.active_pane)
            .field("cursor_position", &self.cursor_position)
            .field("app_exited", &self.app_exited)
            .finish()
    }
}

impl BluelineWorld {
    pub fn new() -> Self {
        let mut view_model = ViewModel::new();
        // Set up event bus
        view_model.set_event_bus(Box::new(SimpleEventBus::new()));

        let command_registry = CommandRegistry::new();

        Self {
            view_model,
            command_registry,
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

    /// Simulate key press in the REPL using real command processing
    pub fn press_key(&mut self, key: &str) -> Result<()> {
        // Convert key string to crossterm KeyEvent
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let key_event = match key {
            // Special keys
            "Escape" => KeyEvent::new(KeyCode::Esc, KeyModifiers::empty()),
            "Enter" => KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()),
            "Tab" => KeyEvent::new(KeyCode::Tab, KeyModifiers::empty()),
            "Backspace" => KeyEvent::new(KeyCode::Backspace, KeyModifiers::empty()),
            "Delete" => KeyEvent::new(KeyCode::Delete, KeyModifiers::empty()),

            // Arrow keys
            "Up" => KeyEvent::new(KeyCode::Up, KeyModifiers::empty()),
            "Down" => KeyEvent::new(KeyCode::Down, KeyModifiers::empty()),
            "Left" => KeyEvent::new(KeyCode::Left, KeyModifiers::empty()),
            "Right" => KeyEvent::new(KeyCode::Right, KeyModifiers::empty()),

            // Page keys
            "Page Up" => KeyEvent::new(KeyCode::PageUp, KeyModifiers::empty()),
            "Page Down" => KeyEvent::new(KeyCode::PageDown, KeyModifiers::empty()),

            // Control combinations
            "Ctrl+C" => KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
            "Ctrl+U" => KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL),
            "Ctrl+D" => KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL),
            "Ctrl+F" => KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL),
            "Ctrl+B" => KeyEvent::new(KeyCode::Char('b'), KeyModifiers::CONTROL),
            "Ctrl+J" => KeyEvent::new(KeyCode::Char('j'), KeyModifiers::CONTROL),
            "Ctrl+K" => KeyEvent::new(KeyCode::Char('k'), KeyModifiers::CONTROL),
            "Ctrl+W" => KeyEvent::new(KeyCode::Char('w'), KeyModifiers::CONTROL),

            // Shift combinations
            s if s.starts_with("Shift+") => {
                let ch = s.trim_start_matches("Shift+").chars().next().unwrap_or(' ');
                KeyEvent::new(KeyCode::Char(ch.to_ascii_uppercase()), KeyModifiers::SHIFT)
            }

            // Single characters
            s if s.len() == 1 => {
                let ch = s.chars().next().unwrap();
                KeyEvent::new(KeyCode::Char(ch), KeyModifiers::empty())
            }

            _ => {
                return Err(anyhow::anyhow!("Unknown key: {}", key));
            }
        };

        // Create command context from current view model state
        let context = self.create_command_context();

        // Process the key event through the real command registry
        if let Ok(events) = self.command_registry.process_event(key_event, &context) {
            // Apply each command event to the view model
            for event in events {
                self.apply_command_event(event)?;
            }
        }

        // Update our compatibility fields from the real view model
        self.sync_from_view_model();

        Ok(())
    }

    /// Create command context from current view model state
    fn create_command_context(&self) -> CommandContext {
        let snapshot = ViewModelSnapshot {
            current_mode: self.view_model.get_mode(),
            current_pane: self.view_model.get_current_pane(),
            cursor_position: self.view_model.get_cursor_position(),
            request_text: self.view_model.get_request_text(),
            response_text: self.view_model.get_response_text(),
            terminal_width: 80,  // Default for tests
            terminal_height: 24, // Default for tests
            verbose: false,
        };
        CommandContext::new(snapshot)
    }

    /// Apply command event to the view model
    fn apply_command_event(&mut self, event: CommandEvent) -> Result<()> {
        use blueline::repl::commands::MovementDirection;

        match event {
            CommandEvent::CursorMoveRequested { direction, amount } => {
                for _ in 0..amount {
                    match direction {
                        MovementDirection::Left => self.view_model.move_cursor_left()?,
                        MovementDirection::Right => self.view_model.move_cursor_right()?,
                        MovementDirection::Up => self.view_model.move_cursor_up()?,
                        MovementDirection::Down => self.view_model.move_cursor_down()?,
                        MovementDirection::LineEnd => {
                            self.view_model.move_cursor_to_end_of_line()?
                        }
                        MovementDirection::LineStart => {
                            self.view_model.move_cursor_to_start_of_line()?
                        }
                        MovementDirection::ScrollLeft => {
                            self.view_model.scroll_horizontally(-1, amount)?
                        }
                        MovementDirection::ScrollRight => {
                            self.view_model.scroll_horizontally(1, amount)?
                        }
                        // BUGFIX: Add missing DocumentStart/DocumentEnd cases to test framework
                        // Without these cases, gg and G command integration tests would fail
                        // because test framework couldn't apply the movement events they generate
                        MovementDirection::DocumentStart => {
                            self.view_model.move_cursor_to_document_start()?
                        }
                        MovementDirection::DocumentEnd => {
                            self.view_model.move_cursor_to_document_end()?
                        }
                        MovementDirection::PageDown => {
                            self.view_model.scroll_vertically_by_page(1)?
                        }
                        MovementDirection::PageUp => {
                            self.view_model.scroll_vertically_by_page(-1)?
                        }
                        MovementDirection::HalfPageDown => {
                            self.view_model.scroll_vertically_by_half_page(1)?
                        }
                        MovementDirection::HalfPageUp => {
                            self.view_model.scroll_vertically_by_half_page(-1)?
                        }
                        MovementDirection::WordForward
                        | MovementDirection::WordBackward
                        | MovementDirection::WordEnd => {
                            // Word movements not implemented yet
                        }
                        MovementDirection::LineNumber(line_number) => {
                            self.view_model.move_cursor_to_line(line_number)?
                        }
                    }
                }
            }
            CommandEvent::ModeChangeRequested { new_mode } => {
                self.view_model.change_mode(new_mode)?;
            }
            CommandEvent::PaneSwitchRequested { target_pane } => {
                self.view_model.switch_pane(target_pane)?;
            }
            CommandEvent::TextInsertRequested { text, .. } => {
                self.view_model.insert_text(&text)?;
            }
            CommandEvent::ExCommandCharRequested { ch } => {
                self.view_model.add_ex_command_char(ch)?;
            }
            CommandEvent::ExCommandBackspaceRequested => {
                self.view_model.backspace_ex_command()?;
            }
            CommandEvent::ExCommandExecuteRequested => {
                let events = self.view_model.execute_ex_command()?;
                for event in events {
                    self.apply_command_event(event)?;
                }
            }
            CommandEvent::QuitRequested => {
                self.app_exited = true;
            }
            _ => {
                // Other events not handled in tests
            }
        }
        Ok(())
    }

    /// Sync compatibility fields from view model
    fn sync_from_view_model(&mut self) {
        // Update mode
        self.mode = match self.view_model.get_mode() {
            EditorMode::Normal => Mode::Normal,
            EditorMode::Insert => Mode::Insert,
            EditorMode::Command => Mode::Command,
            EditorMode::GPrefix => Mode::Normal, // Treat G prefix mode as normal for testing
            EditorMode::Visual => Mode::Normal, // Treat visual mode as normal for testing compatibility
        };

        // Update pane
        self.active_pane = match self.view_model.get_current_pane() {
            Pane::Request => ActivePane::Request,
            Pane::Response => ActivePane::Response,
        };

        // Update cursor position
        let pos = self.view_model.get_cursor_position();
        self.cursor_position = CursorPosition {
            line: pos.line,
            column: pos.column,
        };

        // Update buffers
        self.request_buffer = self
            .view_model
            .get_request_text()
            .lines()
            .map(|s| s.to_string())
            .collect();

        let response_text = self.view_model.get_response_text();
        if !response_text.is_empty() {
            self.response_buffer = response_text.lines().map(|s| s.to_string()).collect();
        }
    }

    /// Type text into the current buffer
    pub fn type_text(&mut self, text: &str) -> Result<()> {
        // Check mode and sync first
        self.sync_from_view_model();

        // In insert mode, each character generates a TextInsertRequested event
        if self.view_model.get_mode() == EditorMode::Insert {
            // For each character, process it through the command system
            for ch in text.chars() {
                self.press_key(&ch.to_string())?;
            }
        } else if self.view_model.get_mode() == EditorMode::Command {
            // In command mode, add to ex command buffer
            for ch in text.chars() {
                self.view_model.add_ex_command_char(ch)?;
            }
        } else {
            // If not in insert or command mode, this might be a test setup issue
            // Let's first switch to insert mode and then type
            self.view_model.change_mode(EditorMode::Insert)?;
            for ch in text.chars() {
                self.press_key(&ch.to_string())?;
            }
        }

        // Sync state
        self.sync_from_view_model();
        Ok(())
    }

    /// Set request buffer content from a multiline string
    pub fn set_request_buffer(&mut self, content: &str) {
        // For now, just simulate typing the content
        // TODO: Find a better way to set buffer content directly
        if !content.trim().is_empty() {
            self.view_model
                .change_mode(EditorMode::Insert)
                .unwrap_or(());
            self.view_model.insert_text(content).unwrap_or(());
            self.view_model
                .change_mode(EditorMode::Normal)
                .unwrap_or(());
        }

        // Sync compatibility fields
        self.sync_from_view_model();
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
