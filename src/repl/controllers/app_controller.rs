//! # REPL Application Controller
//!
//! The controller orchestrates the REPL components and manages the event loop.
//! It's responsible for connecting user input to commands and coordinating view updates.

use crate::config::AppConfig;
use crate::repl::{
    commands::{
        CommandContext, CommandEvent, CommandRegistry, ExCommandRegistry, HttpHeaders,
        MovementDirection, Setting, SettingValue, ViewModelSnapshot,
    },
    events::{EditorMode, LogicalPosition, Pane, SimpleEventBus},
    io::{EventStream, RenderStream},
    utils::parse_request_from_text,
    view_models::{ViewModel, YankType},
    views::{TerminalRenderer, ViewRenderer},
};
use anyhow::Result;
use bluenote::{get_blank_profile, HttpConnectionProfile, IniProfileStore};
use crossterm::event::{Event, KeyEvent};
use std::time::Duration;

/// The main application controller that orchestrates the MVVM pattern
pub struct AppController<ES: EventStream, RS: RenderStream> {
    view_model: ViewModel,
    view_renderer: TerminalRenderer<RS>,
    command_registry: CommandRegistry,
    ex_command_registry: ExCommandRegistry,
    #[allow(dead_code)]
    event_bus: SimpleEventBus,
    event_stream: ES,
    should_quit: bool,
    last_render_time: std::time::Instant,
}

impl<ES: EventStream, RS: RenderStream> AppController<ES, RS> {
    /// Create new application controller with injected I/O streams (dependency injection)
    pub fn with_io_streams(config: AppConfig, event_stream: ES, render_stream: RS) -> Result<Self> {
        let mut view_model = ViewModel::new();

        // Pass RenderStream ownership to the View layer (TerminalRenderer)
        let view_renderer = TerminalRenderer::with_render_stream(render_stream)?;
        let command_registry = CommandRegistry::new();
        let ex_command_registry = ExCommandRegistry::new();
        let event_bus = SimpleEventBus::new();

        // Synchronize view model with actual terminal size
        let (width, height) = view_renderer.terminal_size();
        view_model.update_terminal_size(width, height);

        // Load profile from configuration
        let profile_name = config.profile_name();
        let profile_path = config.profile_path();
        let profile = Self::load_profile(profile_name, profile_path)?;

        // Configure view model with profile and settings
        Self::configure_view_model(&mut view_model, &profile, profile_name, profile_path);

        // Create the controller
        let mut controller = Self {
            view_model,
            view_renderer,
            command_registry,
            ex_command_registry,
            event_bus,
            event_stream,
            should_quit: false,
            last_render_time: std::time::Instant::now(),
        };

        // Apply initial commands from config file
        if !config.initial_commands().is_empty() {
            tracing::info!(
                "Applying {} config commands",
                config.initial_commands().len()
            );
            controller.apply_initial_commands(config.initial_commands())?;
        }

        Ok(controller)
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

    /// Configure view model with profile settings
    fn configure_view_model(
        view_model: &mut ViewModel,
        profile: &impl HttpConnectionProfile,
        profile_name: &str,
        profile_path: &str,
    ) {
        // Set up HTTP client with the loaded profile
        if let Err(e) = view_model.set_http_client(profile) {
            tracing::warn!("Failed to create HTTP client with profile: {}", e);
            // Continue with default client
        }

        // Store profile information for display
        view_model.set_profile_info(profile_name.to_string(), profile_path.to_string());

        // Set up event bus in view model
        view_model.set_event_bus(Box::new(SimpleEventBus::new()));
    }

    /// Apply initial ex commands from config file
    fn apply_initial_commands(&mut self, commands: &[String]) -> Result<()> {
        for command in commands {
            tracing::debug!("Applying config command: {}", command);

            // Create command context
            let context = CommandContext::new(ViewModelSnapshot::from_view_model(&self.view_model));

            // Execute the ex command
            match self.ex_command_registry.execute_command(command, &context) {
                Ok(events) => {
                    // Apply each event
                    for event in events {
                        match event {
                            CommandEvent::SettingChangeRequested { setting, value } => {
                                if let Err(e) = self.handle_setting_change(setting, value) {
                                    tracing::warn!("Failed to apply setting from config: {}", e);
                                }
                            }
                            _ => {
                                tracing::debug!(
                                    "Ignoring non-setting command event from config: {:?}",
                                    event
                                );
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to execute config command '{}': {}", command, e);
                }
            }
        }
        Ok(())
    }

    /// Run the main application loop
    ///
    /// HIGH-LEVEL LOGIC FLOW:
    /// 1. Initialize terminal and perform initial render
    /// 2. Main event loop with 100ms timeout polling:
    ///    a. Read terminal events (keyboard, resize)
    ///    b. Convert events to commands via CommandRegistry
    ///    c. Apply commands to ViewModel (business logic)
    ///    d. Collect ViewEvents from ViewModel changes
    ///    e. Render only what changed (selective rendering)
    /// 3. Handle terminal cleanup on exit
    ///
    /// CRITICAL PERFORMANCE OPTIMIZATIONS:
    /// - Throttled rendering (500μs minimum interval) prevents ghost cursors
    /// - Selective rendering only updates changed screen regions
    /// - Event-driven architecture minimizes unnecessary redraws
    pub async fn run(&mut self) -> Result<()> {
        // INITIALIZATION PHASE: Setup terminal and initial display
        self.view_renderer.initialize()?;
        self.view_renderer.render_full(&self.view_model)?;

        // MAIN EVENT LOOP: Handle user input and update display
        while !self.should_quit {
            self.process_next_event().await?;
        }

        // Cleanup (all handled by view renderer)
        self.view_renderer.cleanup()?;

        Ok(())
    }

    /// Process the next terminal event if available
    async fn process_next_event(&mut self) -> Result<()> {
        // Poll for terminal events with 100ms timeout
        if !self.event_stream.poll(Duration::from_millis(100))? {
            return Ok(());
        }

        match self.event_stream.read()? {
            Event::Key(key_event) => self.handle_key_event(key_event).await?,
            Event::Resize(width, height) => self.handle_resize_event(width, height)?,
            _ => {} // Ignore other events for now
        }

        Ok(())
    }

    /// Handle keyboard input events
    async fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        tracing::debug!("Received key event: {:?}", key_event);

        // Create command context snapshot for command processing
        let context = CommandContext::new(ViewModelSnapshot::from_view_model(&self.view_model));

        // Convert key event to command events via registry
        let Ok(events) = self.command_registry.process_event(key_event, &context) else {
            return Ok(());
        };

        tracing::debug!("Command events generated: {:?}", events);

        if events.is_empty() {
            return Ok(());
        }

        // Apply command events to ViewModel
        for event in events {
            self.apply_command_event(event).await?;
        }

        // Perform throttled rendering if needed
        if !self.should_quit {
            self.render_if_needed()?;
        }

        Ok(())
    }

    /// Handle terminal resize events
    fn handle_resize_event(&mut self, width: u16, height: u16) -> Result<()> {
        // Synchronize both model and view with new terminal dimensions
        self.view_model.update_terminal_size(width, height);
        self.view_renderer.update_size(width, height);
        // Full redraw required after resize to handle layout changes
        self.view_renderer.render_full(&self.view_model)?;
        Ok(())
    }

    /// Perform rendering with throttling to prevent ghost cursors
    fn render_if_needed(&mut self) -> Result<()> {
        let now = std::time::Instant::now();
        let min_render_interval = Duration::from_micros(500);

        if now.duration_since(self.last_render_time) < min_render_interval {
            return Ok(());
        }

        let view_events = self.view_model.collect_pending_view_events();
        self.process_view_events(view_events)?;
        self.last_render_time = now;

        Ok(())
    }

    /// Apply a command event to the view model
    ///
    /// HIGH-LEVEL LOGIC FLOW:
    /// This method serves as the command processor that translates semantic commands
    /// into specific ViewModel operations. Each CommandEvent type maps to one or more
    /// ViewModel method calls that modify application state and emit ViewEvents.
    ///
    /// ARCHITECTURAL PATTERN:
    /// - Commands are processed atomically (all-or-nothing)
    /// - State changes emit ViewEvents for selective rendering
    /// - Complex commands (like ex commands) can generate nested events
    /// - HTTP requests are handled asynchronously with status updates
    async fn apply_command_event(&mut self, event: CommandEvent) -> Result<()> {
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
                        MovementDirection::PageDown => self.view_model.move_cursor_page_down()?,
                        MovementDirection::PageUp => self.view_model.move_cursor_page_up()?,
                        MovementDirection::HalfPageDown => {
                            self.view_model.move_cursor_half_page_down()?
                        }
                        MovementDirection::HalfPageUp => {
                            self.view_model.move_cursor_half_page_up()?
                        }
                    }
                }
            }
            CommandEvent::CursorPositionRequested { position } => {
                self.view_model.set_cursor_position(position)?;
            }
            CommandEvent::TextInsertRequested { text, position: _ } => {
                // Check if we're in Visual Block Insert mode with multiple cursors
                if self.view_model.is_in_visual_block_insert_mode() {
                    self.handle_multi_cursor_text_insert(&text)?;
                } else {
                    self.view_model.insert_text(&text)?;
                }
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

                // Check if we're in Visual Block Insert mode with multiple cursors
                if self.view_model.is_in_visual_block_insert_mode() {
                    self.handle_multi_cursor_text_delete(amount, direction)?;
                } else {
                    for i in 0..amount {
                        match direction {
                            MovementDirection::Left => {
                                tracing::debug!(
                                    "🗑️  Attempting delete_char_before_cursor (iteration {})",
                                    i + 1
                                );
                                match self.view_model.delete_char_before_cursor() {
                                    Ok(_) => {
                                        tracing::debug!("✅ delete_char_before_cursor succeeded")
                                    }
                                    Err(e) => {
                                        tracing::error!(
                                            "❌ delete_char_before_cursor failed: {}",
                                            e
                                        )
                                    }
                                }
                            }
                            MovementDirection::Right => {
                                tracing::debug!(
                                    "🗑️  Attempting delete_char_after_cursor (iteration {})",
                                    i + 1
                                );
                                match self.view_model.delete_char_after_cursor() {
                                    Ok(_) => {
                                        tracing::debug!("✅ delete_char_after_cursor succeeded")
                                    }
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
            CommandEvent::RestorePreviousModeRequested => {
                let previous_mode = self.view_model.get_previous_mode();
                tracing::debug!("Restoring previous mode: {:?}", previous_mode);
                match self.view_model.change_mode(previous_mode) {
                    Ok(_) => {
                        tracing::info!("Successfully restored previous mode: {:?}", previous_mode);
                    }
                    Err(e) => {
                        tracing::error!(
                            "Failed to restore previous mode {:?}: {}",
                            previous_mode,
                            e
                        );
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
                // Get the ex command string from the view model
                let command_str = self.view_model.get_ex_command_buffer().to_string();

                // Create command context for ex command execution
                let context =
                    CommandContext::new(ViewModelSnapshot::from_view_model(&self.view_model));

                // Execute through the ex command registry
                let events = self
                    .ex_command_registry
                    .execute_command(&command_str, &context)?;

                // Clear the command buffer and return to previous mode after successful execution
                self.view_model.clear_ex_command_buffer();
                let previous_mode = self.view_model.get_previous_mode();
                self.view_model.change_mode(previous_mode)?;

                // Handle events directly to avoid recursion
                for event in events {
                    match event {
                        CommandEvent::QuitRequested => {
                            self.should_quit = true;
                        }
                        CommandEvent::ShowProfileRequested => {
                            self.handle_show_profile();
                        }
                        CommandEvent::SettingChangeRequested { setting, value } => {
                            // Handle setting changes from ex commands
                            self.handle_setting_change(setting, value)?;
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
            CommandEvent::SettingChangeRequested { setting, value } => {
                self.handle_setting_change(setting, value)?;
            }
            CommandEvent::YankSelectionRequested => {
                self.handle_yank_selection()?;
            }
            CommandEvent::DeleteSelectionRequested => {
                self.handle_delete_selection()?;
            }
            CommandEvent::CutSelectionRequested => {
                self.handle_cut_selection()?;
            }
            CommandEvent::CutCharacterRequested => {
                self.handle_cut_character()?;
            }
            CommandEvent::CutToEndOfLineRequested => {
                self.handle_cut_to_end_of_line()?;
            }
            CommandEvent::CutCurrentLineRequested => {
                self.handle_cut_current_line()?;
            }
            CommandEvent::ChangeSelectionRequested => {
                self.handle_change_selection()?;
            }
            CommandEvent::VisualBlockInsertRequested => {
                self.handle_visual_block_insert()?;
            }
            CommandEvent::VisualBlockAppendRequested => {
                self.handle_visual_block_append()?;
            }
            CommandEvent::ExitVisualBlockInsertRequested => {
                self.handle_exit_visual_block_insert()?;
            }
            CommandEvent::RepeatVisualSelectionRequested => {
                self.handle_repeat_visual_selection()?;
            }
            CommandEvent::PasteAfterRequested => {
                self.handle_paste_after()?;
            }
            CommandEvent::PasteAtCursorRequested => {
                self.handle_paste_at_cursor()?;
            }
            CommandEvent::NoAction => {
                // Do nothing
            }
        }

        Ok(())
    }

    /// Handle HTTP request execution
    ///
    /// HIGH-LEVEL LOGIC FLOW:
    /// 1. Set executing status for immediate UI feedback
    /// 2. Parse request content from current buffer text
    /// 3. Execute HTTP request asynchronously via bluenote client
    /// 4. Update response pane with results or error messages  
    /// 5. Clear executing status and refresh status bar
    ///
    /// CRITICAL TIMING:
    /// - Status bar updates happen immediately (before/after request)
    /// - Request execution is fully asynchronous
    /// - UI remains responsive during network operations
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
    ///
    /// HIGH-LEVEL LOGIC FLOW:
    /// 1. Collect and group ViewEvents to minimize redundant renders
    /// 2. Determine optimal rendering strategy based on event types
    /// 3. Execute renders in order of efficiency (full > area > partial > status)
    /// 4. Always render cursor last to prevent ghost cursor artifacts
    ///
    /// PERFORMANCE OPTIMIZATIONS:
    /// - Event grouping prevents duplicate renders of same areas
    /// - Selective rendering only updates changed screen regions
    /// - Cursor management prevents flickering and ghost cursors
    /// - Full redraw overrides all other events for simplicity
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

    /// Handle setting changes from ex commands
    fn handle_setting_change(&mut self, setting: Setting, value: SettingValue) -> Result<()> {
        self.view_model.apply_setting(setting, value)
    }

    /// Handle yanking selected text to yank buffer
    fn handle_yank_selection(&mut self) -> Result<()> {
        // Get selected text from current pane
        if let Some(text) = self.view_model.get_selected_text() {
            // Determine yank type based on current visual mode
            let current_mode = self.view_model.get_mode();
            let yank_type = match current_mode {
                EditorMode::Visual => YankType::Character,
                EditorMode::VisualLine => YankType::Line,
                EditorMode::VisualBlock => YankType::Block,
                _ => YankType::Character, // Fallback for any other mode
            };

            // Store in yank buffer with appropriate type
            self.view_model
                .yank_to_buffer_with_type(text.clone(), yank_type)?;

            // Switch to Normal mode (automatically clears visual selection)
            self.view_model.change_mode(EditorMode::Normal)?;

            // Show feedback in status bar
            let char_count = text.chars().count();
            let line_count = text.lines().count();
            let message = match yank_type {
                YankType::Character => {
                    if line_count > 1 {
                        format!("{line_count} lines yanked (character-wise)")
                    } else {
                        format!("{char_count} characters yanked")
                    }
                }
                YankType::Line => format!("{line_count} lines yanked (line-wise)"),
                YankType::Block => format!("Block yanked ({line_count} lines, {char_count} chars)"),
            };
            self.view_model.set_status_message(message);

            tracing::info!(
                "Yanked {} characters ({} lines) to buffer as {:?}",
                char_count,
                line_count,
                yank_type
            );
        } else {
            tracing::warn!("No text selected for yanking");
            self.view_model
                .set_status_message("No text selected".to_string());
        }

        Ok(())
    }

    /// Handle deleting selected text
    fn handle_delete_selection(&mut self) -> Result<()> {
        // Delete the selected text - the method now returns the deleted text directly
        if let Some(deleted_text) = self.view_model.delete_selected_text()? {
            // Switch to Normal mode (automatically clears visual selection)
            self.view_model.change_mode(EditorMode::Normal)?;

            // Show feedback in status bar
            let char_count = deleted_text.chars().count();
            let line_count = deleted_text.lines().count();
            let message = if line_count > 1 {
                format!("{line_count} lines deleted")
            } else {
                format!("{char_count} characters deleted")
            };
            self.view_model.set_status_message(message);

            tracing::info!("Deleted {} characters ({} lines)", char_count, line_count);
        } else {
            tracing::warn!("No text selected for deletion");
            self.view_model
                .set_status_message("No text selected".to_string());
        }

        Ok(())
    }

    /// Handle cutting (delete + yank) selected text
    fn handle_cut_selection(&mut self) -> Result<()> {
        // Cut combines yank + delete, but we need to yank first before deleting
        if let Some(text) = self.view_model.get_selected_text() {
            // Determine yank type based on current visual mode BEFORE any mode changes
            let current_mode = self.view_model.get_mode();
            let yank_type = match current_mode {
                EditorMode::Visual => YankType::Character,
                EditorMode::VisualLine => YankType::Line,
                EditorMode::VisualBlock => YankType::Block,
                _ => YankType::Character, // Fallback for any other mode
            };

            // First yank to buffer with appropriate type
            self.view_model
                .yank_to_buffer_with_type(text.clone(), yank_type)?;

            // Then delete the selected text (this also returns the deleted text for verification)
            if let Some(deleted_text) = self.view_model.delete_selected_text()? {
                // Switch to Normal mode (automatically clears visual selection)
                self.view_model.change_mode(EditorMode::Normal)?;

                // Show feedback in status bar
                let char_count = deleted_text.chars().count();
                let line_count = deleted_text.lines().count();
                let message = match yank_type {
                    YankType::Character => {
                        if line_count > 1 {
                            format!("{line_count} lines cut (character-wise)")
                        } else {
                            format!("{char_count} characters cut")
                        }
                    }
                    YankType::Line => format!("{line_count} lines cut (line-wise)"),
                    YankType::Block => {
                        format!("Block cut ({line_count} lines, {char_count} chars)")
                    }
                };
                self.view_model.set_status_message(message);

                tracing::info!(
                    "Cut {} characters ({} lines) to buffer as {:?}",
                    char_count,
                    line_count,
                    yank_type
                );
            } else {
                tracing::warn!("Failed to delete selected text during cut operation");
                self.view_model
                    .set_status_message("Cut operation failed".to_string());
            }
        } else {
            tracing::warn!("No text selected for cutting");
            self.view_model
                .set_status_message("No text selected".to_string());
        }

        Ok(())
    }

    /// Handle cutting (delete + yank) character at cursor
    fn handle_cut_character(&mut self) -> Result<()> {
        // Cut character at cursor position - the method already handles yanking
        self.view_model.cut_char_at_cursor()?;

        tracing::info!("Cut 1 character at cursor to yank buffer");

        Ok(())
    }

    /// Handle cutting (delete + yank) from cursor to end of line
    fn handle_cut_to_end_of_line(&mut self) -> Result<()> {
        // Cut from cursor to end of line - the method already handles yanking
        self.view_model.cut_to_end_of_line()?;

        tracing::info!("Cut from cursor to end of line to yank buffer");

        Ok(())
    }

    /// Handle cutting (delete + yank) entire current line
    fn handle_cut_current_line(&mut self) -> Result<()> {
        // Cut entire current line - the method already handles yanking
        self.view_model.cut_current_line()?;

        tracing::info!("Cut entire current line to yank buffer");

        Ok(())
    }

    /// Handle change selection operation (Visual Block mode 'c' command)
    ///
    /// This implements vim's Visual Block change command:
    /// 1. Delete the selected rectangular block
    /// 2. Enter Visual Block Insert mode for multi-cursor text replacement
    /// 3. Shows multi-cursor feedback on all affected lines in real-time
    /// 4. When Esc is pressed, exits Visual Block Insert mode
    fn handle_change_selection(&mut self) -> Result<()> {
        // Change operation is currently only supported in Visual Block mode
        let current_mode = self.view_model.get_mode();
        if current_mode != EditorMode::VisualBlock {
            tracing::warn!("Change selection only supported in Visual Block mode, current mode: {current_mode:?}");
            self.view_model.set_status_message(
                "Change command only supported in Visual Block mode".to_string(),
            );
            return Ok(());
        }

        // Get the visual selection before deleting it
        let (selection_start, selection_end, _pane) = self.view_model.get_visual_selection();
        if selection_start.is_none() || selection_end.is_none() {
            tracing::warn!("No visual selection for change operation");
            self.view_model
                .set_status_message("No text selected".to_string());
            return Ok(());
        }

        let start = selection_start.unwrap();
        let end = selection_end.unwrap();

        // Calculate the cursor positions for Visual Block Insert mode
        // This is similar to Visual Block Insert, but we start from the deleted block
        let top_line = start.line.min(end.line);
        let bottom_line = start.line.max(end.line);
        let left_col = start.column.min(end.column);

        // Delete the selected block text first
        if let Some(deleted_text) = self.view_model.delete_selected_text()? {
            // Create cursor positions for all lines in the deleted block range
            let mut cursor_positions = Vec::new();
            for line_num in top_line..=bottom_line {
                cursor_positions.push(LogicalPosition::new(line_num, left_col));
            }

            // Set up Visual Block Insert mode with multi-cursor state
            self.view_model
                .set_visual_block_insert_cursors(cursor_positions.clone());

            // Switch to VisualBlockInsert mode (not regular Insert)
            self.view_model.change_mode(EditorMode::VisualBlockInsert)?;

            // Position the main cursor at the first line of the block
            self.view_model.set_cursor_position(cursor_positions[0])?;

            // Show feedback in status bar
            let char_count = deleted_text.chars().count();
            let line_count = deleted_text.lines().count();
            let message = if line_count > 1 {
                format!("Changed {line_count} lines, Visual Block Insert mode")
            } else {
                format!("Changed {char_count} characters, Visual Block Insert mode")
            };
            self.view_model.set_status_message(message);

            tracing::info!(
                "Changed {} characters ({} lines), entered Visual Block Insert mode with {} cursors",
                char_count,
                line_count,
                cursor_positions.len()
            );
        } else {
            tracing::warn!("No text selected for changing");
            self.view_model
                .set_status_message("No text selected".to_string());
        }

        Ok(())
    }

    /// Handle Visual Block Insert operation ('I' in Visual Block mode)
    ///
    /// This implements vim's Visual Block Insert command:
    /// 1. Remember the selected block coordinates
    /// 2. Move cursor to the start of the first selected line in the block  
    /// 3. Enter special VisualBlockInsert mode
    /// 4. Text typed appears on first line, replicated to all lines on Esc
    fn handle_visual_block_insert(&mut self) -> Result<()> {
        // Only supported in Visual Block mode
        let current_mode = self.view_model.get_mode();
        if current_mode != EditorMode::VisualBlock {
            tracing::warn!("Visual Block Insert only supported in Visual Block mode, current mode: {current_mode:?}");
            self.view_model.set_status_message(
                "Visual Block Insert only supported in Visual Block mode".to_string(),
            );
            return Ok(());
        }

        // Get the visual selection coordinates
        let (start_pos, end_pos, pane) = self.view_model.get_visual_selection();
        if let (Some(start), Some(end), Some(selected_pane)) = (start_pos, end_pos, pane) {
            if selected_pane != self.view_model.get_current_pane() {
                tracing::warn!("Visual selection is not in current pane");
                return Ok(());
            }

            // Calculate the block boundaries
            let start_line = start.line.min(end.line);
            let end_line = start.line.max(end.line);
            let start_col = start.column.min(end.column);

            // Create cursor positions for all lines in the block
            let mut cursor_positions = Vec::new();
            for line in start_line..=end_line {
                cursor_positions.push(LogicalPosition::new(line, start_col));
            }

            // Set multi-cursor state for Visual Block Insert
            self.view_model
                .set_visual_block_insert_cursors(cursor_positions);

            // Move primary cursor to start of block (beginning of leftmost column on first line)
            self.view_model
                .set_cursor_position(LogicalPosition::new(start_line, start_col))?;

            // Enter Visual Block Insert mode
            self.view_model.change_mode(EditorMode::VisualBlockInsert)?;

            // Show feedback
            let line_count = (start.line.max(end.line) - start_line) + 1;
            self.view_model
                .set_status_message(format!("Visual Block Insert: {line_count} lines"));

            tracing::info!(
                "Entered Visual Block Insert mode at position ({}, {}), affecting {} lines",
                start_line,
                start_col,
                line_count
            );
        } else {
            tracing::warn!("No visual block selection found");
            self.view_model
                .set_status_message("No visual block selection".to_string());
        }

        Ok(())
    }

    /// Handle Visual Block Append operation ('A' in Visual Block mode)
    ///
    /// This implements vim's Visual Block Append command:
    /// 1. Remember the selected block coordinates
    /// 2. Move cursor to the end of the first selected line in the block
    /// 3. Enter special VisualBlockInsert mode
    /// 4. Text typed appears on first line, replicated to all lines on Esc
    fn handle_visual_block_append(&mut self) -> Result<()> {
        // Only supported in Visual Block mode
        let current_mode = self.view_model.get_mode();
        if current_mode != EditorMode::VisualBlock {
            tracing::warn!("Visual Block Append only supported in Visual Block mode, current mode: {current_mode:?}");
            self.view_model.set_status_message(
                "Visual Block Append only supported in Visual Block mode".to_string(),
            );
            return Ok(());
        }

        // Get the visual selection coordinates
        let (start_pos, end_pos, pane) = self.view_model.get_visual_selection();
        if let (Some(start), Some(end), Some(selected_pane)) = (start_pos, end_pos, pane) {
            if selected_pane != self.view_model.get_current_pane() {
                tracing::warn!("Visual selection is not in current pane");
                return Ok(());
            }

            // Calculate the block boundaries
            let start_line = start.line.min(end.line);
            let end_line = start.line.max(end.line);
            let end_col = start.column.max(end.column);

            // Create cursor positions for all lines in the block (AFTER the end position for append)
            // Visual Block 'A' should position cursor after the rightmost selected character
            let mut cursor_positions = Vec::new();
            for line in start_line..=end_line {
                cursor_positions.push(LogicalPosition::new(line, end_col + 1));
            }

            // Set multi-cursor state for Visual Block Insert
            self.view_model
                .set_visual_block_insert_cursors(cursor_positions);

            // Move primary cursor to after the end of block (one position after rightmost column)
            self.view_model
                .set_cursor_position(LogicalPosition::new(start_line, end_col + 1))?;

            // Enter Visual Block Insert mode
            self.view_model.change_mode(EditorMode::VisualBlockInsert)?;

            // Show feedback
            let line_count = (start.line.max(end.line) - start_line) + 1;
            self.view_model
                .set_status_message(format!("Visual Block Append: {line_count} lines"));

            tracing::info!(
                "Entered Visual Block Append mode at position ({}, {}), affecting {} lines",
                start_line,
                end_col,
                line_count
            );
        } else {
            tracing::warn!("No visual block selection found");
            self.view_model
                .set_status_message("No visual block selection".to_string());
        }

        Ok(())
    }

    /// Handle exit from Visual Block Insert mode with text replication
    ///
    /// This implements the complex vim behavior where:
    /// 1. Text typed on the first line during Visual Block Insert is captured
    /// 2. That text is replicated to all lines that were in the original block selection  
    /// 3. Cursor is positioned at the end of the inserted text on the first line
    fn handle_exit_visual_block_insert(&mut self) -> Result<()> {
        tracing::info!("Exiting Visual Block Insert mode");

        // Preserve cursor position at the first multi-cursor position
        let cursor_to_preserve = self
            .view_model
            .get_visual_block_insert_cursors()
            .first()
            .copied(); // Get first cursor position before clearing

        // Clear multi-cursor state
        self.view_model.clear_visual_block_insert_cursors();

        // Clear visual selection that was active when we entered Visual Block Insert
        self.view_model.clear_visual_selection()?;

        // Restore cursor position to where typing was happening (first cursor)
        if let Some(preserved_cursor) = cursor_to_preserve {
            self.view_model.set_cursor_position(preserved_cursor)?;
            tracing::debug!("Preserved cursor position at {:?}", preserved_cursor);
        }

        self.view_model.change_mode(EditorMode::Normal)?;

        // Clear any previous status messages when exiting Visual Block Insert
        self.view_model.clear_status_message();

        Ok(())
    }

    /// Handle repeat visual selection (gv command)
    ///
    /// Restores the last visual selection including:
    /// 1. The selection range (start and end positions)
    /// 2. The visual mode type (character/line/block)
    /// 3. Cursor position at end of selection
    fn handle_repeat_visual_selection(&mut self) -> Result<()> {
        tracing::info!("Handling repeat visual selection (gv command)");

        // First, return to Normal mode to exit GPrefix mode
        self.view_model.change_mode(EditorMode::Normal)?;

        // Try to restore the last visual selection
        match self.view_model.restore_last_visual_selection()? {
            Some(mode) => {
                tracing::info!("Restored last visual selection with mode {:?}", mode);
                // Change to the restored visual mode
                self.view_model.change_mode(mode)?;
            }
            None => {
                tracing::info!("No previous visual selection to restore");
                // Stay in Normal mode if there's no selection to restore
            }
        }

        Ok(())
    }

    /// Handle text insertion for multi-cursor Visual Block Insert mode
    ///
    /// Inserts the same text at all cursor positions simultaneously,
    /// providing live feedback across all selected lines.
    fn handle_multi_cursor_text_insert(&mut self, text: &str) -> Result<()> {
        let cursor_positions = self.view_model.get_visual_block_insert_cursors().to_vec();

        if cursor_positions.is_empty() {
            // Fallback to regular insert if no cursors are set
            return self.view_model.insert_text(text);
        }

        tracing::debug!(
            "Multi-cursor text insert: '{}' at {} positions",
            text,
            cursor_positions.len()
        );

        // Insert text at each cursor position
        // We need to process in reverse order to maintain position validity
        for position in cursor_positions.iter().rev() {
            // Temporarily set cursor to this position and insert text
            self.view_model.set_cursor_position(*position)?;
            self.view_model.insert_text(text)?;
        }

        // Update all cursor positions to reflect the inserted text
        let text_len = text.chars().count(); // Handle multi-byte characters correctly
        let updated_positions: Vec<LogicalPosition> = cursor_positions
            .iter()
            .map(|pos| LogicalPosition::new(pos.line, pos.column + text_len))
            .collect();

        // Set the primary cursor to the first position before updating positions
        if let Some(first_pos) = updated_positions.first() {
            self.view_model.set_cursor_position(*first_pos)?;
        }

        self.view_model
            .update_visual_block_insert_cursors(updated_positions);

        tracing::debug!("Multi-cursor text insert completed, updated cursor positions");
        Ok(())
    }

    /// Handle text deletion for multi-cursor Visual Block Insert mode
    fn handle_multi_cursor_text_delete(
        &mut self,
        amount: usize,
        direction: MovementDirection,
    ) -> Result<()> {
        let cursor_positions = self.view_model.get_visual_block_insert_cursors().to_vec();
        let start_columns = self
            .view_model
            .get_visual_block_insert_start_columns()
            .to_vec();

        if cursor_positions.is_empty() {
            // Fallback to regular delete if no cursors are set
            for _ in 0..amount {
                match direction {
                    MovementDirection::Left => {
                        self.view_model.delete_char_before_cursor()?;
                    }
                    MovementDirection::Right => {
                        self.view_model.delete_char_after_cursor()?;
                    }
                    _ => {
                        tracing::warn!("Unsupported delete direction: {:?}", direction);
                    }
                }
            }
            return Ok(());
        }

        tracing::debug!(
            "Multi-cursor text delete: {} chars in direction {:?} at {} positions, start columns: {:?}",
            amount,
            direction,
            cursor_positions.len(),
            start_columns
        );

        // Perform deletion at each cursor position, respecting boundaries
        // We need to process in reverse order to maintain position validity
        for (i, position) in cursor_positions.iter().enumerate().rev() {
            let start_column = start_columns.get(i).copied().unwrap_or(0);

            // Temporarily set cursor to this position
            self.view_model.set_cursor_position(*position)?;

            // For left deletion (backspace), respect the Visual Block start boundary
            let effective_amount = if direction == MovementDirection::Left {
                // Calculate how many characters we can actually delete without going beyond start
                let current_col = position.column;
                let max_deletable = current_col.saturating_sub(start_column);
                let effective = amount.min(max_deletable);
                tracing::debug!(
                    "Backspace calculation: line={}, current_col={}, start_col={}, max_deletable={}, requested={}, effective={}",
                    position.line, current_col, start_column, max_deletable, amount, effective
                );
                effective
            } else {
                amount
            };

            for _ in 0..effective_amount {
                match direction {
                    MovementDirection::Left => {
                        self.view_model.delete_char_before_cursor()?;
                    }
                    MovementDirection::Right => {
                        self.view_model.delete_char_after_cursor()?;
                    }
                    _ => {
                        tracing::warn!("Unsupported delete direction: {:?}", direction);
                        break;
                    }
                }
            }

            tracing::debug!(
                "Line {}: deleted {} chars (requested: {}, start_column: {}, current: {})",
                position.line,
                effective_amount,
                amount,
                start_column,
                position.column
            );
        }

        // Update all cursor positions to reflect the deleted text
        let updated_positions: Vec<LogicalPosition> = match direction {
            MovementDirection::Left => {
                // For backspace, cursor positions move left by amount actually deleted (respecting boundaries)
                cursor_positions
                    .iter()
                    .enumerate()
                    .map(|(i, pos)| {
                        let start_column = start_columns.get(i).copied().unwrap_or(0);
                        let current_col = pos.column;
                        let max_deletable = current_col.saturating_sub(start_column);
                        let effective_amount = amount.min(max_deletable);
                        LogicalPosition::new(pos.line, pos.column.saturating_sub(effective_amount))
                    })
                    .collect()
            }
            MovementDirection::Right => {
                // For forward delete, cursor positions stay the same
                cursor_positions
            }
            _ => cursor_positions,
        };

        // Set the primary cursor to the first position before updating positions
        if let Some(first_pos) = updated_positions.first() {
            self.view_model.set_cursor_position(*first_pos)?;
        }

        self.view_model
            .update_visual_block_insert_cursors(updated_positions);

        tracing::debug!("Multi-cursor text delete completed, updated cursor positions");
        Ok(())
    }

    /// Handle pasting yanked text after cursor
    fn handle_paste_after(&mut self) -> Result<()> {
        if let Some(yank_entry) = self.view_model.get_yanked_entry() {
            // Paste the text after the current cursor position using type-aware paste
            self.view_model.paste_after_with_type(&yank_entry)?;

            // Show feedback
            let char_count = yank_entry.text.chars().count();
            let line_count = yank_entry.text.lines().count();
            let message = match yank_entry.yank_type {
                YankType::Character => {
                    if line_count > 1 {
                        format!("{line_count} lines pasted (character-wise)")
                    } else {
                        format!("{char_count} characters pasted")
                    }
                }
                YankType::Line => format!("{line_count} lines pasted (line-wise)"),
                YankType::Block => format!("Block pasted ({line_count} lines, {char_count} chars)"),
            };
            self.view_model.set_status_message(message);

            tracing::info!(
                "Pasted {} characters ({} lines) after cursor as {:?}",
                char_count,
                line_count,
                yank_entry.yank_type
            );
        } else {
            self.view_model
                .set_status_message("Nothing to paste".to_string());
            tracing::warn!("No text in yank buffer to paste");
        }

        Ok(())
    }

    /// Handle pasting yanked text at current cursor position
    fn handle_paste_at_cursor(&mut self) -> Result<()> {
        if let Some(yank_entry) = self.view_model.get_yanked_entry() {
            tracing::debug!(
                "Retrieved yank entry with type: {:?}, text length: {}",
                yank_entry.yank_type,
                yank_entry.text.len()
            );

            // Paste the text at current position (before cursor) using type-aware paste
            self.view_model.paste_with_type(&yank_entry)?;

            // Show feedback
            let char_count = yank_entry.text.chars().count();
            let line_count = yank_entry.text.lines().count();
            let message = match yank_entry.yank_type {
                YankType::Character => {
                    if line_count > 1 {
                        format!("{line_count} lines pasted (character-wise)")
                    } else {
                        format!("{char_count} characters pasted")
                    }
                }
                YankType::Line => format!("{line_count} lines pasted (line-wise)"),
                YankType::Block => format!("Block pasted ({line_count} lines, {char_count} chars)"),
            };
            self.view_model.set_status_message(message);

            tracing::info!(
                "Pasted {} characters ({} lines) at cursor as {:?}",
                char_count,
                line_count,
                yank_entry.yank_type
            );
        } else {
            self.view_model
                .set_status_message("Nothing to paste".to_string());
            tracing::warn!("No text in yank buffer to paste");
        }

        Ok(())
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
    use crate::cmd_args::CommandLineArgs;
    use crate::repl::events::{EditorMode, Pane};

    #[test]
    fn app_controller_should_create() {
        if crossterm::terminal::size().is_ok() {
            let cmd_args = CommandLineArgs::parse_from(["test"]);
            let config = AppConfig::from_args(cmd_args);
            let controller = AppController::with_io_streams(
                config,
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
