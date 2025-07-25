//! # Controller - Main REPL Controller and Command Coordination
//!
//! This module contains the main controller that coordinates the MVC components.
//! It manages the event loop, maintains the command registry, and orchestrates
//! interactions between models, views, and commands.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────┐
//! │   ReplController    │
//! │                     │
//! │  ┌───────────────┐  │    ┌─────────────┐
//! │  │   Commands    │  │────▶│ AppState    │
//! │  │   Registry    │  │    │ (Model)     │
//! │  └───────────────┘  │    └─────────────┘
//! │           │         │           │
//! │           ▼         │           ▼
//! │  ┌───────────────┐  │    ┌─────────────┐
//! │  │  Event Loop   │  │    │ ViewManager │
//! │  └───────────────┘  │    │ (View)      │
//! └─────────────────────┘    └─────────────┘
//! ```

use std::collections::HashMap;
use std::io;

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyEvent},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};

use bluenote::{HttpClient, HttpRequestArgs, IniProfile, Url, UrlPath};

use super::{
    command::{Command, CommandResult},
    commands::{
        CancelCommandModeCommand, CommandModeInputCommand, DeleteCharCommand,
        EnterCommandModeCommand, EnterInsertModeCommand, ExecuteCommandCommand,
        ExitInsertModeCommand, InsertCharCommand, InsertNewLineCommand, MoveCursorDownCommand,
        MoveCursorLeftCommand, MoveCursorLineEndCommand, MoveCursorLineStartCommand,
        MoveCursorRightCommand, MoveCursorUpCommand, ScrollHalfPageUpCommand, SwitchPaneCommand,
    },
    model::{AppState, ResponseBuffer},
    view::{create_default_view_manager, ViewManager},
};

/// HTTP request arguments parsed from the request buffer
#[derive(Debug)]
struct BufferRequestArgs {
    method: Option<String>,
    url_path: Option<UrlPath>,
    body: Option<String>,
    headers: HashMap<String, String>,
}

/// Type alias for the result of parsing HTTP requests from text
/// Returns (request_args, url_string) on success, or error message on failure
type ParseRequestResult = Result<(BufferRequestArgs, String), String>;

impl HttpRequestArgs for BufferRequestArgs {
    fn method(&self) -> Option<&String> {
        self.method.as_ref()
    }

    fn url_path(&self) -> Option<&UrlPath> {
        self.url_path.as_ref()
    }

    fn body(&self) -> Option<&String> {
        self.body.as_ref()
    }

    fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }
}

/// Type alias for command registry to reduce complexity
type CommandRegistry = Vec<Box<dyn Command>>;

/// Main controller that orchestrates the REPL application.
///
/// This is the central coordinator that:
/// - Manages the event loop
/// - Maintains command registry  
/// - Coordinates model updates and view rendering
/// - Handles application lifecycle
pub struct ReplController {
    state: AppState,
    view_manager: ViewManager,
    commands: CommandRegistry,
    client: HttpClient,
    profile: IniProfile,
}

impl ReplController {
    /// Create a new REPL controller
    pub fn new(profile: IniProfile, verbose: bool) -> Result<Self> {
        let client = HttpClient::new(&profile)?;
        let terminal_size = terminal::size()?;

        let state = AppState::new(terminal_size, verbose);
        let view_manager = create_default_view_manager();

        let mut controller = Self {
            state,
            view_manager,
            commands: Vec::new(),
            client,
            profile,
        };

        // Register default commands
        controller.register_default_commands();

        Ok(controller)
    }

    /// Run the REPL event loop
    pub async fn run(&mut self) -> Result<()> {
        // Initialize terminal
        terminal::enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen)?;

        self.view_manager.initialize_terminal(&self.state)?;

        // Initial render
        self.view_manager.render_full(&self.state)?;

        let result = self.event_loop().await;

        // Cleanup
        self.view_manager.cleanup_terminal()?;
        execute!(io::stdout(), LeaveAlternateScreen)?;
        terminal::disable_raw_mode()?;

        result
    }

    /// Main event processing loop
    async fn event_loop(&mut self) -> Result<()> {
        loop {
            match event::read()? {
                Event::Key(key) => {
                    let should_quit = self.handle_key_event(key).await?;
                    if should_quit {
                        break;
                    }
                }
                Event::Resize(width, height) => {
                    self.state.update_terminal_size((width, height));
                    self.view_manager.render_full(&self.state)?;
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Handle a key event by dispatching to appropriate commands
    async fn handle_key_event(&mut self, key: KeyEvent) -> Result<bool> {
        let mut should_quit = false;
        let mut any_handled = false;
        let mut command_results = Vec::new();

        // Track old state for change detection
        let old_mode = self.state.mode.clone();
        let old_pane = self.state.current_pane;
        let old_request_scroll = self.state.request_buffer.scroll_offset;
        let old_response_scroll = self.state.response_buffer.as_ref().map(|r| r.scroll_offset);
        let old_request_pane_height = self.state.request_pane_height;

        // Try each command until one handles the event
        for command in &self.commands {
            // Use the unified Command trait (CommandV2 is auto-implemented via blanket impl)
            if !command.is_relevant(&self.state, &key) {
                continue;
            }

            // Store state before processing to detect changes
            let old_request_content = self.state.request_buffer.get_text();
            let old_request_cursor_line = self.state.request_buffer.cursor_line;
            let old_request_cursor_col = self.state.request_buffer.cursor_col;
            let old_response_cursor = self
                .state
                .response_buffer
                .as_ref()
                .map(|r| (r.cursor_line, r.cursor_col));

            let handled = command.process(key, &mut self.state)?;
            if handled {
                // Detect what actually changed by comparing before/after state
                let new_request_content = self.state.request_buffer.get_text();
                let request_content_changed = old_request_content != new_request_content;
                let request_cursor_moved = old_request_cursor_line
                    != self.state.request_buffer.cursor_line
                    || old_request_cursor_col != self.state.request_buffer.cursor_col;

                let response_cursor_moved =
                    match (&old_response_cursor, &self.state.response_buffer) {
                        (Some((old_line, old_col)), Some(ref buffer)) => {
                            *old_line != buffer.cursor_line || *old_col != buffer.cursor_col
                        }
                        _ => false,
                    };

                let cursor_moved = request_cursor_moved || response_cursor_moved;

                command_results.push(CommandResult {
                    handled: true,
                    content_changed: request_content_changed,
                    cursor_moved,
                    mode_changed: false, // Will be detected by comparing old_mode
                    pane_changed: false, // Will be detected by comparing old_pane
                    scroll_occurred: false, // Will be detected by comparing scroll offsets
                    status_message: None,
                });
                any_handled = true;
                break; // First handler wins
            }
        }

        // Handle special quit commands
        if matches!(key.code, crossterm::event::KeyCode::Char('c'))
            && key
                .modifiers
                .contains(crossterm::event::KeyModifiers::CONTROL)
        {
            should_quit = true;
        }

        // Check if quit was requested via command mode
        if self.state.should_quit {
            should_quit = true;
        }

        // Determine what type of rendering is needed
        if any_handled {
            self.update_view_based_on_changes(
                &command_results,
                old_mode,
                old_pane,
                old_request_scroll,
                old_response_scroll,
                old_request_pane_height,
            )?;
        }

        // Check if HTTP request execution was requested
        if self.state.execute_request_flag {
            self.state.execute_request_flag = false; // Reset flag
            self.execute_http_request().await?;
            // Re-render after HTTP request execution
            self.view_manager.render_full(&self.state)?;
        }

        Ok(should_quit)
    }

    /// Update the view based on detected changes
    fn update_view_based_on_changes(
        &mut self,
        results: &[CommandResult],
        old_mode: super::model::EditorMode,
        old_pane: super::model::Pane,
        old_request_scroll: usize,
        old_response_scroll: Option<usize>,
        old_request_pane_height: usize,
    ) -> Result<()> {
        // Check if scrolling occurred
        let scroll_occurred = self.state.request_buffer.scroll_offset != old_request_scroll
            || self.state.response_buffer.as_ref().map(|r| r.scroll_offset) != old_response_scroll;

        // Check if pane layout changed
        let pane_layout_changed = self.state.request_pane_height != old_request_pane_height;

        // Aggregate results to determine render strategy
        let any_mode_changed =
            results.iter().any(|r| r.mode_changed) || self.state.mode != old_mode;
        let any_pane_changed =
            results.iter().any(|r| r.pane_changed) || self.state.current_pane != old_pane;
        let any_scroll = results.iter().any(|r| r.scroll_occurred) || scroll_occurred;
        let any_content_changed = results.iter().any(|r| r.content_changed);
        let any_cursor_moved = results.iter().any(|r| r.cursor_moved);

        // Apply rendering strategy based on the same logic as the original
        let needs_full_render = any_mode_changed
            || any_pane_changed
            || any_scroll
            || pane_layout_changed
            || matches!(
                self.state.mode,
                super::model::EditorMode::Command
                    | super::model::EditorMode::Visual
                    | super::model::EditorMode::VisualLine
            );

        let needs_content_update = any_content_changed && !needs_full_render;

        if needs_full_render {
            self.view_manager.render_full(&self.state)?;
        } else if needs_content_update {
            self.view_manager.render_content_update(&self.state)?;
        } else if any_cursor_moved {
            self.view_manager.render_cursor_only(&self.state)?;
        }

        Ok(())
    }

    /// Register all default commands
    fn register_default_commands(&mut self) {
        // Movement commands
        self.commands.push(Box::new(MoveCursorLeftCommand::new()));
        self.commands.push(Box::new(MoveCursorRightCommand::new()));
        self.commands.push(Box::new(MoveCursorUpCommand::new()));
        self.commands.push(Box::new(MoveCursorDownCommand::new()));
        self.commands
            .push(Box::new(MoveCursorLineStartCommand::new()));
        self.commands
            .push(Box::new(MoveCursorLineEndCommand::new()));
        self.commands.push(Box::new(SwitchPaneCommand::new()));
        self.commands.push(Box::new(ScrollHalfPageUpCommand::new()));

        // Editing commands
        self.commands.push(Box::new(EnterInsertModeCommand::new()));
        self.commands.push(Box::new(ExitInsertModeCommand::new()));
        self.commands.push(Box::new(InsertCharCommand::new()));
        self.commands.push(Box::new(InsertNewLineCommand::new()));
        self.commands.push(Box::new(DeleteCharCommand::new()));

        // Command mode commands
        self.commands.push(Box::new(EnterCommandModeCommand::new()));
        self.commands.push(Box::new(CommandModeInputCommand::new()));
        self.commands.push(Box::new(ExecuteCommandCommand::new()));
        self.commands
            .push(Box::new(CancelCommandModeCommand::new()));

        // Note: Commands are processed in order, so put more specific commands first
        // and more general commands (like InsertCharCommand) later
    }

    /// Add a custom command to the registry
    pub fn register_command(&mut self, command: Box<dyn Command>) {
        self.commands.push(command);
    }

    /// Get reference to current application state (for testing/debugging)
    pub fn state(&self) -> &AppState {
        &self.state
    }

    /// Get mutable reference to current application state (for testing/debugging)
    pub fn state_mut(&mut self) -> &mut AppState {
        &mut self.state
    }

    /// Parse HTTP request from the request buffer content
    /// Returns (BufferRequestArgs, url_str) or error message
    fn parse_request_from_buffer(&self) -> ParseRequestResult {
        Self::parse_request_from_text(
            &self.state.request_buffer.get_text(),
            &self.state.session_headers,
        )
    }

    /// Parse HTTP request from text content (static method for testing)
    /// Returns (BufferRequestArgs, url_str) or error message
    fn parse_request_from_text(
        text: &str,
        session_headers: &HashMap<String, String>,
    ) -> ParseRequestResult {
        let lines: Vec<&str> = text.lines().collect();

        if lines.is_empty() || lines[0].trim().is_empty() {
            return Err("No request to execute".to_string());
        }

        // Parse first line as method and URL
        let parts: Vec<&str> = lines[0].split_whitespace().collect();
        if parts.len() < 2 {
            return Err("Invalid request format. Use: METHOD URL".to_string());
        }

        let method = parts[0].to_uppercase();
        let url_str = parts[1].to_string();

        // Parse URL
        let url = Url::parse(&url_str);

        // Skip empty line after URL if it exists, then rest becomes the body
        let body_start_idx = if lines.len() > 1 && lines[1].trim().is_empty() {
            2
        } else {
            1
        };

        let body = if lines.len() > body_start_idx {
            Some(lines[body_start_idx..].join("\n"))
        } else {
            None
        };

        // Create request args
        let request_args = BufferRequestArgs {
            method: Some(method),
            url_path: url.to_url_path().cloned(),
            body,
            headers: session_headers.clone(),
        };

        Ok((request_args, url_str))
    }

    /// Execute HTTP request from the request buffer content
    async fn execute_http_request(&mut self) -> Result<()> {
        let (request_args, url_str) = match self.parse_request_from_buffer() {
            Ok(result) => result,
            Err(error_message) => {
                self.state.status_message = error_message;
                return Ok(());
            }
        };

        // Start timing the request
        self.state.request_start_time = Some(std::time::Instant::now());

        // Execute the request
        match self.client.request(&request_args).await {
            Ok(response) => {
                // Calculate request duration
                if let Some(start_time) = self.state.request_start_time.take() {
                    self.state.last_request_duration =
                        Some(start_time.elapsed().as_millis() as u64);
                }

                let status = response.status();
                let body = response.body();

                // Store the response status for display in status bar
                self.state.last_response_status = Some(format!(
                    "HTTP {} {}",
                    status.as_u16(),
                    status.canonical_reason().unwrap_or("")
                ));

                let mut response_text = String::new();

                if self.state.verbose {
                    // Add request information
                    response_text.push_str(&format!(
                        "Request: {} {}\n",
                        request_args.method().unwrap_or(&"GET".to_string()),
                        url_str
                    ));

                    // Add headers if any
                    if !self.state.session_headers.is_empty() {
                        response_text.push_str("Headers:\n");
                        for (key, value) in &self.state.session_headers {
                            response_text.push_str(&format!("  {}: {}\n", key, value));
                        }
                    }

                    response_text.push('\n');

                    // Add response status
                    response_text.push_str(&format!(
                        "Response: {} {}\n\n",
                        status.as_u16(),
                        status.canonical_reason().unwrap_or("")
                    ));
                }

                if let Some(json) = response.json() {
                    response_text.push_str(
                        &serde_json::to_string_pretty(json)
                            .unwrap_or_else(|_| "Invalid JSON".to_string()),
                    );
                } else if !body.is_empty() {
                    // For very long response bodies, add line breaks to prevent display issues
                    let processed_body = if body.lines().any(|line| line.len() > 1000) {
                        // Break very long lines into chunks
                        body.lines()
                            .map(|line| {
                                if line.len() > 1000 {
                                    let mut chunks = Vec::new();
                                    for chunk in line.as_bytes().chunks(1000) {
                                        if let Ok(chunk_str) = std::str::from_utf8(chunk) {
                                            chunks.push(chunk_str.to_string());
                                        }
                                    }
                                    chunks.join("\n")
                                } else {
                                    line.to_string()
                                }
                            })
                            .collect::<Vec<_>>()
                            .join("\n")
                    } else {
                        body.to_string()
                    };
                    response_text.push_str(&processed_body);
                }

                self.state.response_buffer = Some(ResponseBuffer::new(response_text));

                // Update status message
                self.state.status_message = format!(
                    "Request executed: {} {}",
                    status.as_u16(),
                    status.canonical_reason().unwrap_or("")
                );
            }
            Err(err) => {
                // Reset timing on error
                self.state.request_start_time = None;
                self.state.status_message = format!("Request failed: {}", err);
            }
        }

        Ok(())
    }
}

// Trait extension to allow downcasting for CommandV2 check
trait AsAny {
    fn as_any(&self) -> &dyn std::any::Any;
}

impl<T: Command + 'static> AsAny for T {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// TODO: Remove this trait object extension once we have a better solution
// This is a temporary workaround for the downcasting issue
impl dyn Command {
    fn as_any(&self) -> &dyn std::any::Any {
        panic!("as_any not implemented for this Command type")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_parse_request_from_text_valid_get() {
        let text = "GET https://api.example.com/users";
        let headers = HashMap::new();

        let result = ReplController::parse_request_from_text(text, &headers);
        assert!(result.is_ok());

        let (request_args, url_str) = result.unwrap();
        assert_eq!(request_args.method(), Some(&"GET".to_string()));
        assert_eq!(url_str, "https://api.example.com/users");
        assert_eq!(request_args.body(), None);
    }

    #[test]
    fn test_parse_request_from_text_with_body() {
        let text = "POST https://api.example.com/users\n\n{\"name\": \"test\"}";
        let headers = HashMap::new();

        let result = ReplController::parse_request_from_text(text, &headers);
        assert!(result.is_ok());

        let (request_args, url_str) = result.unwrap();
        assert_eq!(request_args.method(), Some(&"POST".to_string()));
        assert_eq!(url_str, "https://api.example.com/users");
        assert_eq!(
            request_args.body(),
            Some(&"{\"name\": \"test\"}".to_string())
        );
    }

    #[test]
    fn test_parse_request_from_text_multiline_body() {
        let text = "PUT https://api.example.com/users/1\n\n{\n  \"name\": \"test\",\n  \"email\": \"test@example.com\"\n}";
        let headers = HashMap::new();

        let result = ReplController::parse_request_from_text(text, &headers);
        assert!(result.is_ok());

        let (request_args, _) = result.unwrap();
        let expected_body = "{\n  \"name\": \"test\",\n  \"email\": \"test@example.com\"\n}";
        assert_eq!(request_args.body(), Some(&expected_body.to_string()));
    }

    #[test]
    fn test_parse_request_from_text_empty() {
        let text = "";
        let headers = HashMap::new();

        let result = ReplController::parse_request_from_text(text, &headers);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "No request to execute");
    }

    #[test]
    fn test_parse_request_from_text_invalid_format() {
        let text = "GET"; // Missing URL
        let headers = HashMap::new();

        let result = ReplController::parse_request_from_text(text, &headers);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Invalid request format. Use: METHOD URL"
        );
    }

    #[test]
    fn test_parse_request_from_text_method_case_insensitive() {
        let text = "post https://api.example.com/users";
        let headers = HashMap::new();

        let result = ReplController::parse_request_from_text(text, &headers);
        assert!(result.is_ok());

        let (request_args, _) = result.unwrap();
        assert_eq!(request_args.method(), Some(&"POST".to_string()));
    }

    #[test]
    fn test_parse_request_from_text_with_session_headers() {
        let text = "GET https://api.example.com/users";
        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), "Bearer token123".to_string());
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        let result = ReplController::parse_request_from_text(text, &headers);
        assert!(result.is_ok());

        let (request_args, _) = result.unwrap();
        assert_eq!(
            request_args.headers().get("Authorization"),
            Some(&"Bearer token123".to_string())
        );
        assert_eq!(
            request_args.headers().get("Content-Type"),
            Some(&"application/json".to_string())
        );
    }

    #[test]
    fn test_parse_request_from_text_body_without_empty_line() {
        let text = "POST https://api.example.com/users\n{\"name\": \"test\"}";
        let headers = HashMap::new();

        let result = ReplController::parse_request_from_text(text, &headers);
        assert!(result.is_ok());

        let (request_args, _) = result.unwrap();
        assert_eq!(
            request_args.body(),
            Some(&"{\"name\": \"test\"}".to_string())
        );
    }
}
