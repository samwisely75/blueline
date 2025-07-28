//! # REPL Application Controller
//!
//! The controller orchestrates the REPL components and manages the event loop.
//! It's responsible for connecting user input to commands and coordinating view updates.

use crate::cmd_args::CommandLineArgs;
use crate::repl::{
    commands::{CommandContext, CommandEvent, CommandRegistry, HttpHeaders, ViewModelSnapshot},
    events::{Pane, SimpleEventBus},
    utils::parse_request_from_text,
    view_models::ViewModel,
    views::{TerminalRenderer, ViewRenderer},
};
use anyhow::Result;
use bluenote::{get_blank_profile, HttpConnectionProfile, IniProfileStore, DEFAULT_INI_FILE_PATH};
use crossterm::{
    event::{self, Event},
    execute, terminal,
};
use std::{
    io::{self, Write},
    time::Duration,
};

/// The main application controller that orchestrates the MVVM pattern
pub struct AppController {
    view_model: ViewModel,
    view_renderer: TerminalRenderer,
    command_registry: CommandRegistry,
    #[allow(dead_code)]
    event_bus: SimpleEventBus,
    should_quit: bool,
    last_render_time: std::time::Instant,
}

impl AppController {
    /// Create new application controller with command line arguments
    pub fn new(cmd_args: CommandLineArgs) -> Result<Self> {
        let mut view_model = ViewModel::new();
        let view_renderer = TerminalRenderer::new()?;
        let command_registry = CommandRegistry::new();
        let event_bus = SimpleEventBus::new();

        // Synchronize view model with actual terminal size
        let (width, height) = view_renderer.terminal_size();
        view_model.update_terminal_size(width, height);

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
            last_render_time: std::time::Instant::now(),
        })
    }

    /// Run the main application loop
    pub async fn run(&mut self) -> Result<()> {
        // Initialize terminal explicitly (matching MVC pattern)
        terminal::enable_raw_mode().map_err(anyhow::Error::from)?;
        execute!(io::stdout(), terminal::EnterAlternateScreen)?;

        // Initialize view renderer (this will clear screen and set initial cursor)
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
                                // Apply events to view model (this will emit appropriate ViewEvents)
                                for event in events {
                                    self.apply_command_event(event).await?;
                                }

                                // Process view events for selective rendering (if not quitting)
                                if !self.should_quit {
                                    // Throttle rapid rendering to prevent ghost cursors
                                    let now = std::time::Instant::now();
                                    let min_render_interval = Duration::from_micros(500);

                                    if now.duration_since(self.last_render_time)
                                        >= min_render_interval
                                    {
                                        let view_events =
                                            self.view_model.collect_pending_view_events();
                                        self.process_view_events(view_events)?;
                                        self.last_render_time = now;
                                    }
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

        // Cleanup (matching MVC pattern)
        self.view_renderer.cleanup()?;
        execute!(io::stdout(), terminal::LeaveAlternateScreen)?;
        terminal::disable_raw_mode().map_err(anyhow::Error::from)?;

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
                        MovementDirection::LineStart => {
                            self.view_model.move_cursor_to_start_of_line()?
                        }
                        MovementDirection::ScrollLeft => {
                            self.view_model.scroll_horizontally(-1, amount)?
                        }
                        MovementDirection::ScrollRight => {
                            self.view_model.scroll_horizontally(1, amount)?
                        }
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
                        MovementDirection::WordForward | MovementDirection::WordBackward => {
                            tracing::warn!("Word movement not yet implemented: {:?}", direction);
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
            CommandEvent::ExCommandCharRequested { ch } => {
                self.view_model.add_ex_command_char(ch)?;
            }
            CommandEvent::ExCommandBackspaceRequested => {
                self.view_model.backspace_ex_command()?;
            }
            CommandEvent::ExCommandExecuteRequested => {
                let events = self.view_model.execute_ex_command()?;
                // Handle events directly to avoid recursion
                for event in events {
                    match event {
                        CommandEvent::QuitRequested => {
                            self.should_quit = true;
                        }
                        _ => {
                            tracing::warn!(
                                "Unhandled event from ex command execution: {:?}",
                                event
                            );
                        }
                    }
                }
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
        // Set executing status to show "Executing..." in status bar
        self.view_model.set_executing_request(true);

        // Immediately refresh the status bar to show executing message
        self.view_renderer.render_status_bar(&self.view_model)?;

        // Get request text and session headers from view model
        let request_text = self.view_model.get_request_text();
        let session_headers = std::collections::HashMap::new(); // TODO: Get from view model

        // Parse request from buffer content
        let (request_args, _url_str) =
            match parse_request_from_text(&request_text, &session_headers) {
                Ok(result) => result,
                Err(error_message) => {
                    self.view_model
                        .set_response(0, format!("Error: {}", error_message));
                    // Clear executing status on error
                    self.view_model.set_executing_request(false);
                    // Refresh status bar to show error
                    self.view_renderer.render_status_bar(&self.view_model)?;
                    return Ok(());
                }
            };

        // Check if HTTP client is available
        if let Some(client) = self.view_model.http_client() {
            // Execute the HTTP request directly using bluenote
            match client.request(&request_args).await {
                Ok(response) => {
                    self.view_model.set_response_from_http(&response);
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

        // Clear executing status when request completes
        self.view_model.set_executing_request(false);

        // Refresh status bar to show response status
        self.view_renderer.render_status_bar(&self.view_model)?;

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

    /// Process view events for selective rendering instead of always doing full redraws
    fn process_view_events(
        &mut self,
        view_events: Vec<crate::repl::events::ViewEvent>,
    ) -> Result<()> {
        use crate::repl::events::ViewEvent;

        // Group events to avoid redundant renders
        let mut needs_full_redraw = false;
        let mut needs_status_bar = false;
        let mut needs_cursor_update = false;
        let mut panes_to_redraw = std::collections::HashSet::new();
        let mut partial_redraws: std::collections::HashMap<Pane, usize> =
            std::collections::HashMap::new();

        for event in view_events {
            match event {
                ViewEvent::FullRedrawRequired => {
                    needs_full_redraw = true;
                    // Full redraw overrides all other events
                    break;
                }
                ViewEvent::PaneRedrawRequired { pane } => {
                    panes_to_redraw.insert(pane);
                    // Full pane redraw overrides partial redraws
                    partial_redraws.remove(&pane);
                }
                ViewEvent::PartialPaneRedrawRequired { pane, start_line } => {
                    // Only add partial redraw if we're not already doing a full pane redraw
                    if !panes_to_redraw.contains(&pane) {
                        partial_redraws
                            .entry(pane)
                            .and_modify(|line| *line = (*line).min(start_line))
                            .or_insert(start_line);
                    }
                }
                ViewEvent::StatusBarUpdateRequired => {
                    needs_status_bar = true;
                }
                ViewEvent::PositionIndicatorUpdateRequired => {
                    // Handle position indicator separately for minimal flickering
                    self.view_renderer
                        .render_position_indicator(&self.view_model)?;
                }
                ViewEvent::CursorUpdateRequired { .. } => {
                    needs_cursor_update = true;
                }
                ViewEvent::ScrollChanged { pane, .. } => {
                    panes_to_redraw.insert(pane);
                    // Scroll requires full pane redraw
                    partial_redraws.remove(&pane);
                    // Ensure cursor is hidden during scroll to prevent ghost cursor
                    needs_cursor_update = true;
                }
            }
        }

        // Process events in order of efficiency
        if needs_full_redraw {
            self.view_renderer.render_full(&self.view_model)?;
        } else {
            // Selective rendering - hide cursor once at the beginning
            let has_content_updates = !panes_to_redraw.is_empty() || !partial_redraws.is_empty();
            if has_content_updates {
                tracing::debug!(
                    "controller: hiding cursor for content updates - panes: {:?}, partial: {:?}",
                    panes_to_redraw,
                    partial_redraws.keys().collect::<Vec<_>>()
                );
                execute!(io::stdout(), crossterm::cursor::Hide)?;
                io::stdout().flush().map_err(anyhow::Error::from)?;

                // Add a tiny delay to ensure cursor hide command is processed by terminal
                // This prevents ghost cursors during rapid key repetition
                std::thread::sleep(std::time::Duration::from_micros(100));
            }

            for pane in &panes_to_redraw {
                self.view_renderer.render_pane(&self.view_model, *pane)?;
            }

            // Handle partial pane redraws
            for (pane, start_line) in &partial_redraws {
                self.view_renderer
                    .render_pane_partial(&self.view_model, *pane, *start_line)?;
            }

            if needs_status_bar {
                self.view_renderer.render_status_bar(&self.view_model)?;
            }

            // Always render cursor after any pane redraw to prevent ghost cursors
            if needs_cursor_update || has_content_updates {
                tracing::debug!("controller: rendering cursor after content updates");
                self.view_renderer.render_cursor(&self.view_model)?;
            }
        }

        Ok(())
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
