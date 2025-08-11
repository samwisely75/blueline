//! # REPL Application Controller
//!
//! The controller orchestrates the REPL components and manages the event loop.
//! It's responsible for connecting user input to commands and coordinating view updates.

use crate::repl::{
    commands::{CommandContext, CommandEvent, CommandRegistry, HttpHeaders, ViewModelSnapshot},
    events::{Pane, SimpleEventBus},
    io::{EventStream, RenderStream},
    utils::parse_request_from_text,
    view_models::ViewModel,
    views::{TerminalRenderer, ViewRenderer},
};
use crate::{cmd_args::CommandLineArgs, config};
use anyhow::Result;
use bluenote::{get_blank_profile, HttpConnectionProfile, IniProfileStore};
use crossterm::event::{Event, KeyEvent};
use std::time::Duration;

/// The main application controller that orchestrates the MVVM pattern
pub struct AppController<ES: EventStream, RS: RenderStream> {
    view_model: ViewModel,
    view_renderer: TerminalRenderer<RS>,
    command_registry: CommandRegistry,
    #[allow(dead_code)]
    event_bus: SimpleEventBus,
    event_stream: ES,
    should_quit: bool,
    last_render_time: std::time::Instant,
}

impl<ES: EventStream, RS: RenderStream> AppController<ES, RS> {
    /// Create new application controller with injected I/O streams (dependency injection)
    pub fn with_io_streams(
        cmd_args: CommandLineArgs,
        event_stream: ES,
        render_stream: RS,
    ) -> Result<Self> {
        let mut view_model = ViewModel::new();

        // Pass RenderStream ownership to the View layer (TerminalRenderer)
        let view_renderer = TerminalRenderer::with_render_stream(render_stream)?;
        let command_registry = CommandRegistry::new();
        let event_bus = SimpleEventBus::new();

        // Synchronize view model with actual terminal size
        let (width, height) = view_renderer.terminal_size();
        view_model.update_terminal_size(width, height);

        // Load profile from configuration
        let profile_name = cmd_args.profile();
        let profile_path = config::get_profile_path();
        let profile = Self::load_profile(profile_name, &profile_path)?;

        // Configure view model with profile and settings
        Self::configure_view_model(
            &mut view_model,
            &profile,
            profile_name,
            &profile_path,
            &cmd_args,
        );

        Ok(Self {
            view_model,
            view_renderer,
            command_registry,
            event_bus,
            event_stream,
            should_quit: false,
            last_render_time: std::time::Instant::now(),
        })
    }
}

impl<ES: EventStream, RS: RenderStream> AppController<ES, RS> {
    /// Load profile from INI file or return blank profile if not found
    fn load_profile(profile_name: &str, profile_path: &str) -> Result<impl HttpConnectionProfile> {
        tracing::debug!("Loading profile '{}' from '{}'", profile_name, profile_path);

        let ini_store = IniProfileStore::new(profile_path);
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

        Ok(profile)
    }

    /// Configure view model with profile settings and command line arguments
    fn configure_view_model(
        view_model: &mut ViewModel,
        profile: &impl HttpConnectionProfile,
        profile_name: &str,
        profile_path: &str,
        cmd_args: &CommandLineArgs,
    ) {
        // Set up HTTP client with the loaded profile
        if let Err(e) = view_model.set_http_client(profile) {
            tracing::warn!("Failed to create HTTP client with profile: {}", e);
            // Continue with default client
        }

        // Store profile information for display
        view_model.set_profile_info(profile_name.to_string(), profile_path.to_string());

        // Set verbose mode from command line args
        view_model.set_verbose(cmd_args.verbose());

        // Set up event bus in view model
        view_model.set_event_bus(Box::new(SimpleEventBus::new()));
    }

    /// Run the main application loop
    pub async fn run(&mut self) -> Result<()> {
        // Initialize view renderer (handles all terminal setup)
        self.view_renderer.initialize()?;

        // Initial render
        self.view_renderer.render_full(&self.view_model)?;

        // Main event loop
        while !self.should_quit {
            // Handle terminal events with timeout
            if self.event_stream.poll(Duration::from_millis(100))? {
                match self.event_stream.read()? {
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
                        // Render on terminal resize
                        self.view_renderer.render_full(&self.view_model)?;
                    }
                    _ => {
                        // Ignore other events for now
                    }
                }
            }
        }

        // Cleanup (all handled by view renderer)
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
                        MovementDirection::LineEndForAppend => {
                            self.view_model.move_cursor_to_line_end_for_append()?
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
                        MovementDirection::WordForward => {
                            self.view_model.move_cursor_to_next_word()?
                        }
                        MovementDirection::WordBackward => {
                            self.view_model.move_cursor_to_previous_word()?
                        }
                        MovementDirection::WordEnd => {
                            self.view_model.move_cursor_to_end_of_word()?
                        }
                        MovementDirection::LineNumber(line_number) => {
                            self.view_model.move_cursor_to_line(line_number)?
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
                tracing::debug!(
                    "🗑️  Processing TextDeleteRequested: amount={}, direction={:?}",
                    amount,
                    direction
                );
                for i in 0..amount {
                    match direction {
                        MovementDirection::Left => {
                            tracing::debug!(
                                "🗑️  Attempting delete_char_before_cursor (iteration {})",
                                i + 1
                            );
                            match self.view_model.delete_char_before_cursor() {
                                Ok(_) => tracing::debug!("✅ delete_char_before_cursor succeeded"),
                                Err(e) => {
                                    tracing::error!("❌ delete_char_before_cursor failed: {}", e)
                                }
                            }
                        }
                        MovementDirection::Right => {
                            tracing::debug!(
                                "🗑️  Attempting delete_char_after_cursor (iteration {})",
                                i + 1
                            );
                            match self.view_model.delete_char_after_cursor() {
                                Ok(_) => tracing::debug!("✅ delete_char_after_cursor succeeded"),
                                Err(e) => {
                                    tracing::error!("❌ delete_char_after_cursor failed: {}", e)
                                }
                            }
                        }
                        _ => {
                            tracing::warn!("Unsupported delete direction: {:?}", direction);
                        }
                    }
                }
                tracing::debug!("🗑️  TextDeleteRequested processing completed");
            }
            CommandEvent::ModeChangeRequested { new_mode } => {
                tracing::debug!("Applying mode change request: {:?}", new_mode);
                match self.view_model.change_mode(new_mode) {
                    Ok(_) => {
                        tracing::info!("Mode successfully changed to: {:?}", new_mode);
                    }
                    Err(e) => {
                        tracing::error!("Failed to change mode to {:?}: {}", new_mode, e);
                        return Err(e);
                    }
                }
            }
            CommandEvent::PaneSwitchRequested { target_pane } => match target_pane {
                Pane::Request => self.view_model.switch_to_request_pane(),
                Pane::Response => self.view_model.switch_to_response_pane(),
            },
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
                        CommandEvent::ShowProfileRequested => {
                            self.handle_show_profile();
                        }
                        CommandEvent::CursorMoveRequested { direction, amount } => {
                            // BUGFIX: Handle line navigation from ex commands like `:58`
                            // Previously these events were unhandled, causing `:number` to not work
                            for _ in 0..amount {
                                match direction {
                                    MovementDirection::LineNumber(line_number) => {
                                        self.view_model.move_cursor_to_line(line_number)?
                                    }
                                    _ => {
                                        tracing::warn!(
                                            "Unsupported movement direction from ex command: {:?}",
                                            direction
                                        );
                                    }
                                }
                            }
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
            CommandEvent::ShowProfileRequested => {
                self.handle_show_profile();
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
                        .set_response(0, format!("Error: {error_message}"));
                    // Clear executing status on error
                    self.view_model.set_executing_request(false);
                    // Refresh status bar to show error (skip in CI mode)
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
                        .set_response(0, format!("HTTP Error: {error}"));
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
        let mut needs_current_area_redraw = false;
        let mut needs_secondary_area_redraw = false;
        let mut partial_redraws: std::collections::HashMap<Pane, usize> =
            std::collections::HashMap::new();

        for event in view_events {
            match event {
                ViewEvent::FullRedrawRequired => {
                    needs_full_redraw = true;
                    // Full redraw overrides all other events
                    break;
                }
                ViewEvent::CurrentAreaRedrawRequired => {
                    needs_current_area_redraw = true;
                }
                ViewEvent::SecondaryAreaRedrawRequired => {
                    needs_secondary_area_redraw = true;
                }
                ViewEvent::CurrentAreaPartialRedrawRequired { start_line } => {
                    // Only add partial redraw if we're not already doing a full current area redraw
                    if !needs_current_area_redraw {
                        let current_pane = self.view_model.get_current_pane();
                        partial_redraws
                            .entry(current_pane)
                            .and_modify(|line| *line = (*line).min(start_line))
                            .or_insert(start_line);
                    }
                }
                ViewEvent::SecondaryAreaPartialRedrawRequired { start_line } => {
                    // Only add partial redraw if we're not already doing a full secondary area redraw
                    if !needs_secondary_area_redraw {
                        let current_pane = self.view_model.get_current_pane();
                        let secondary_pane = match current_pane {
                            Pane::Request => Pane::Response,
                            Pane::Response => Pane::Request,
                        };
                        partial_redraws
                            .entry(secondary_pane)
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
                ViewEvent::ActiveCursorUpdateRequired => {
                    needs_cursor_update = true;
                }
                ViewEvent::CurrentAreaScrollChanged { .. } => {
                    needs_current_area_redraw = true;
                    // Ensure cursor is updated after scroll to prevent ghost cursor
                    needs_cursor_update = true;
                }
                ViewEvent::SecondaryAreaScrollChanged { .. } => {
                    needs_secondary_area_redraw = true;
                }
                ViewEvent::FocusSwitched => {
                    // Focus switch requires cursor update and status bar update
                    needs_cursor_update = true;
                    needs_status_bar = true;
                }
                ViewEvent::RequestContentChanged => {
                    // Request content changed - redraw current area if we're in request pane
                    if self.view_model.is_in_request_pane() {
                        needs_current_area_redraw = true;
                    } else {
                        needs_secondary_area_redraw = true;
                    }
                }
                ViewEvent::ResponseContentChanged => {
                    // Response content changed - redraw current area if we're in response pane
                    if self.view_model.is_in_response_pane() {
                        needs_current_area_redraw = true;
                    } else {
                        needs_secondary_area_redraw = true;
                    }
                }
                ViewEvent::AllContentAreasRedrawRequired => {
                    needs_current_area_redraw = true;
                    needs_secondary_area_redraw = true;
                }
            }
        }

        // Process events in order of efficiency
        if needs_full_redraw {
            self.view_renderer.render_full(&self.view_model)?;
        } else {
            // Selective rendering - renderer handles cursor visibility
            let has_content_updates = needs_current_area_redraw
                || needs_secondary_area_redraw
                || !partial_redraws.is_empty();
            if has_content_updates {
                tracing::debug!(
                    "controller: content updates - current: {}, secondary: {}, partial: {:?}",
                    needs_current_area_redraw,
                    needs_secondary_area_redraw,
                    partial_redraws.keys().collect::<Vec<_>>()
                );
                // Cursor hiding is now handled by each render method in the renderer
                // to ensure consistent behavior and prevent ghost cursors
            }

            // Render current area if needed
            if needs_current_area_redraw {
                let current_pane = self.view_model.get_current_pane();
                self.view_renderer
                    .render_pane(&self.view_model, current_pane)?;
            }

            // Render secondary area if needed
            if needs_secondary_area_redraw {
                let current_pane = self.view_model.get_current_pane();
                let secondary_pane = match current_pane {
                    Pane::Request => Pane::Response,
                    Pane::Response => Pane::Request,
                };
                self.view_renderer
                    .render_pane(&self.view_model, secondary_pane)?;
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
    /// Handle showing profile information in status bar
    fn handle_show_profile(&mut self) {
        let profile_name = self.view_model.get_profile_name();
        let profile_path = self.view_model.get_profile_path();
        let message = format!("[{profile_name}] in {profile_path}");
        self.view_model.set_status_message(message);
    }

    /// Process a single key event without running the full event loop (for testing)
    pub async fn process_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        tracing::debug!("Processing key event: {:?}", key_event);
        tracing::debug!("AppController: process_key_event called with {key_event:?}");

        // Create command context from current state
        tracing::debug!("AppController: Creating command context");
        let context = CommandContext::new(ViewModelSnapshot::from_view_model(&self.view_model));
        tracing::debug!("AppController: Command context created");

        // Process through command registry
        tracing::debug!("AppController: About to call command_registry.process_event");
        if let Ok(events) = self.command_registry.process_event(key_event, &context) {
            tracing::debug!(
                "AppController: Command events generated: {} events",
                events.len()
            );
            tracing::debug!("Command events generated: {:?}", events);
            if !events.is_empty() {
                // Apply events to view model (this will emit appropriate ViewEvents)
                tracing::debug!(
                    "AppController: About to apply {} command events",
                    events.len()
                );
                for (i, event) in events.iter().enumerate() {
                    tracing::debug!(
                        "AppController: Applying event {}/{}: {:?}",
                        i + 1,
                        events.len(),
                        event
                    );
                    self.apply_command_event(event.clone()).await?;
                    tracing::debug!(
                        "AppController: Applied event {}/{} successfully",
                        i + 1,
                        events.len()
                    );
                }
                tracing::debug!("AppController: All command events applied successfully");

                // Render after processing key events
                self.view_renderer.render_full(&self.view_model)?;
            } else {
                tracing::debug!("AppController: No command events generated");
            }
        } else {
            tracing::warn!("AppController: Failed to process key event: {key_event:?}");
        }

        tracing::debug!("AppController: process_key_event completed successfully");
        Ok(())
    }

    /// Check if the application should quit (for testing)
    pub fn should_quit(&self) -> bool {
        self.should_quit
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
            let controller = AppController::with_io_streams(
                cmd_args,
                crate::repl::io::TerminalEventStream::new(),
                crate::repl::io::TerminalRenderStream::new(),
            );
            assert!(controller.is_ok());

            let controller = controller.unwrap();
            assert_eq!(controller.view_model().get_mode(), EditorMode::Normal);
            assert_eq!(controller.view_model().get_current_pane(), Pane::Request);
        }
    }
}
