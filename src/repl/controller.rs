//! # REPL Application Controller
//!
//! The controller orchestrates the REPL components and manages the event loop.
//! It's responsible for connecting user input to commands and coordinating view updates.

use crate::cmd_args::CommandLineArgs;
use crate::repl::{
    commands::{CommandContext, CommandEvent, CommandRegistry, HttpHeaders, ViewModelSnapshot},
    events::SimpleEventBus,
    http::{execute_http_request, parse_request_from_text},
    view_models::ViewModel,
    views::{TerminalRenderer, ViewRenderer},
};
use anyhow::Result;
use bluenote::{get_blank_profile, HttpConnectionProfile, IniProfileStore, DEFAULT_INI_FILE_PATH};
use crossterm::event::{self, Event};
use std::time::Duration;

/// The main application controller that orchestrates the MVVM pattern
pub struct AppController {
    view_model: ViewModel,
    view_renderer: TerminalRenderer,
    command_registry: CommandRegistry,
    #[allow(dead_code)]
    event_bus: SimpleEventBus,
    should_quit: bool,
}

impl AppController {
    /// Create new application controller with command line arguments
    pub fn new(cmd_args: CommandLineArgs) -> Result<Self> {
        let mut view_model = ViewModel::new();
        let view_renderer = TerminalRenderer::new()?;
        let command_registry = CommandRegistry::new();
        let event_bus = SimpleEventBus::new();

        // Load profile from INI file by name specified in --profile argument
        let profile_name = cmd_args.profile();

        tracing::debug!(
            "Loading profile '{}' from '{}'",
            profile_name,
            DEFAULT_INI_FILE_PATH
        );

        let ini_store = IniProfileStore::new(DEFAULT_INI_FILE_PATH);
        let profile_result = ini_store.get_profile(profile_name)?;

        let profile = match profile_result {
            Some(p) => {
                tracing::debug!("Profile loaded successfully, server: {:?}", p.server());
                p
            }
            None => {
                tracing::debug!("Profile '{}' not found, using blank profile", profile_name);
                get_blank_profile()
            }
        };

        // Set up HTTP client with the loaded profile
        if let Err(e) = view_model.set_http_client(&profile) {
            tracing::warn!("Failed to create HTTP client with profile: {}", e);
            // Continue with default client
        }

        // Set verbose mode from command line args
        view_model.set_verbose(cmd_args.verbose());

        // Set up event bus in view model
        view_model.set_event_bus(Box::new(SimpleEventBus::new()));

        Ok(Self {
            view_model,
            view_renderer,
            command_registry,
            event_bus,
            should_quit: false,
        })
    }

    /// Run the main application loop
    pub async fn run(&mut self) -> Result<()> {
        // Initialize terminal
        self.view_renderer.initialize()?;

        // Initial render
        self.view_renderer.render_full(&self.view_model)?;

        // Main event loop
        while !self.should_quit {
            // Handle terminal events with timeout
            if event::poll(Duration::from_millis(100))? {
                match event::read()? {
                    Event::Key(key_event) => {
                        // Debug log the key event
                        tracing::debug!("Received key event: {:?}", key_event);

                        // Create command context from current state
                        let context = CommandContext::new(ViewModelSnapshot::from_view_model(
                            &self.view_model,
                        ));

                        // Process through command registry
                        if let Ok(events) = self.command_registry.process_event(key_event, &context)
                        {
                            tracing::debug!("Command events generated: {:?}", events);
                            if !events.is_empty() {
                                // Apply events to view model
                                for event in events {
                                    self.apply_command_event(event).await?;
                                }

                                // Render after applying events (if not quitting)
                                if !self.should_quit {
                                    self.view_renderer.render_full(&self.view_model)?;
                                }
                            }
                        }
                    }
                    Event::Resize(width, height) => {
                        self.view_model.update_terminal_size(width, height);
                        self.view_renderer.update_size(width, height);
                        self.view_renderer.render_full(&self.view_model)?;
                    }
                    _ => {
                        // Ignore other events for now
                    }
                }
            }
        }

        // Cleanup
        self.view_renderer.cleanup()?;

        Ok(())
    }

    /// Apply a command event to the view model
    async fn apply_command_event(&mut self, event: CommandEvent) -> Result<()> {
        use crate::repl::commands::MovementDirection;

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
                        _ => {
                            tracing::warn!("Unsupported movement direction: {:?}", direction);
                        }
                    }
                }
            }
            CommandEvent::CursorPositionRequested { position } => {
                self.view_model.set_cursor_position(position)?;
            }
            CommandEvent::TextInsertRequested { text, position: _ } => {
                self.view_model.insert_text(&text)?;
            }
            CommandEvent::TextDeleteRequested {
                position: _,
                amount,
                direction,
            } => {
                for _ in 0..amount {
                    match direction {
                        MovementDirection::Left => self.view_model.delete_char_before_cursor()?,
                        MovementDirection::Right => self.view_model.delete_char_after_cursor()?,
                        _ => {
                            tracing::warn!("Unsupported delete direction: {:?}", direction);
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
            CommandEvent::HttpRequestRequested {
                method,
                url,
                headers,
                body,
            } => {
                self.handle_http_request(method, url, headers, body).await?;
            }
            CommandEvent::TerminalResizeRequested { width, height } => {
                self.view_model.update_terminal_size(width, height);
                self.view_renderer.update_size(width, height);
            }
            CommandEvent::QuitRequested => {
                self.should_quit = true;
            }
            CommandEvent::NoAction => {
                // Do nothing
            }
        }

        Ok(())
    }

    /// Handle HTTP request execution
    async fn handle_http_request(
        &mut self,
        _method: String,
        _url: String,
        _headers: HttpHeaders,
        _body: Option<String>,
    ) -> Result<()> {
        // Get request text and session headers from view model
        let request_text = self.view_model.get_request_text();
        let session_headers = std::collections::HashMap::new(); // TODO: Get from view model
        let verbose = self.view_model.is_verbose();

        // Parse request from buffer content
        let (request_args, url_str) = match parse_request_from_text(&request_text, &session_headers)
        {
            Ok(result) => result,
            Err(error_message) => {
                self.view_model
                    .set_response(0, format!("Error: {}", error_message));
                return Ok(());
            }
        };

        // Check if HTTP client is available
        if let Some(client) = self.view_model.http_client() {
            // Execute the HTTP request
            match execute_http_request(client, &request_args, &url_str, verbose).await {
                Ok((response_text, status_code, _duration)) => {
                    self.view_model.set_response(status_code, response_text);
                }
                Err(error) => {
                    self.view_model
                        .set_response(0, format!("HTTP Error: {}", error));
                }
            }
        } else {
            self.view_model
                .set_response(0, "Error: HTTP client not configured".to_string());
        }

        Ok(())
    }

    /// Get reference to view model (for testing)
    pub fn view_model(&self) -> &ViewModel {
        &self.view_model
    }

    /// Get mutable reference to view model (for testing)
    pub fn view_model_mut(&mut self) -> &mut ViewModel {
        &mut self.view_model
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::events::{EditorMode, Pane};

    #[test]
    fn app_controller_should_create() {
        if crossterm::terminal::size().is_ok() {
            let cmd_args = CommandLineArgs::parse_from(["test"]);
            let controller = AppController::new(cmd_args);
            assert!(controller.is_ok());

            let controller = controller.unwrap();
            assert_eq!(controller.view_model().get_mode(), EditorMode::Normal);
            assert_eq!(controller.view_model().get_current_pane(), Pane::Request);
        }
    }
}
