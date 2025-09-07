//! # REPL Application Controller
//!
//! The controller orchestrates the REPL components and manages the event loop.
//! It's responsible for connecting user input to commands and coordinating view updates.

use crate::config::AppConfig;
use crate::repl::{
    commands::{
        AppStateSnapshot, CommandContext, CommandEvent, CommandRegistry, MovementDirection,
    },
    io::{EventStream, RenderStream},
    models::app_state::AppState,
    models::events::SimpleEventBus,
    models::pane_state::Pane,
    models::LogicalPosition,
    services::{HttpResponseMessage, Services},
    unified_commands::{
        events::YankType as NewYankType, Command, DynamicCommandRegistry, ExecutionContext,
        ModelEvent,
    },
    view_models::post_command_actions::PostCommandAction,
    views::{TerminalRenderer, ViewRenderer},
};
use anyhow::Result;
use bluenote::{get_blank_profile, HttpConnectionProfile, HttpRequestArgs, IniProfileStore};
use crossterm::event::{Event, KeyEvent};
use std::time::Duration;
/// The main application ViewModel that orchestrates the MVVM pattern
pub struct AppViewModel<ES: EventStream, RS: RenderStream> {
    app_state: AppState,
    view_renderer: TerminalRenderer<RS>,
    // Services layer for business logic
    services: Services,
    // Old command system (being phased out)
    command_registry: CommandRegistry,
    // New dynamic command system (checks first, falls back to old system)
    unified_command_registry: DynamicCommandRegistry,
    #[allow(dead_code)]
    event_bus: SimpleEventBus,
    event_stream: ES,
    should_quit: bool,
    last_render_time: std::time::Instant,
}

impl<ES: EventStream, RS: RenderStream> AppViewModel<ES, RS> {
    /// Create new application controller with injected I/O streams (dependency injection)
    pub fn with_io_streams(config: AppConfig, event_stream: ES, render_stream: RS) -> Result<Self> {
        let mut app_state = AppState::new();

        // Pass RenderStream ownership to the View layer (TerminalRenderer)
        let view_renderer = TerminalRenderer::with_render_stream(render_stream)?;

        // Load profile from configuration first (needed for Services)
        let profile_name = config.profile_name();
        let profile_path = config.profile_path();
        let profile = Self::load_profile(profile_name, profile_path)?;

        // Initialize services with the profile
        let mut services = Services::new();
        if let Err(e) = services.configure_http(&profile) {
            tracing::warn!("Failed to configure HTTP service: {}", e);
        }

        let command_registry = CommandRegistry::new();
        let unified_command_registry = DynamicCommandRegistry::new();
        let event_bus = SimpleEventBus::new();

        // Synchronize view model with actual terminal size
        let (width, height) = view_renderer.terminal_size();
        app_state.update_terminal_size(width, height);

        // Configure view model with profile and settings
        Self::configure_app_state(&mut app_state, &profile, profile_name, profile_path);

        // Create the ViewModel
        let mut view_model = Self {
            app_state,
            view_renderer,
            services,
            command_registry,
            unified_command_registry,
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
            view_model.apply_initial_commands(config.initial_commands())?;
        }

        Ok(view_model)
    }
}

impl<ES: EventStream, RS: RenderStream> AppViewModel<ES, RS> {
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
    fn configure_app_state(
        app_state: &mut AppState,
        _profile: &impl HttpConnectionProfile,
        profile_name: &str,
        profile_path: &str,
    ) {
        // HTTP client is now managed by the HttpService, not the ViewModel
        // Just store profile information for display
        app_state.set_profile_info(profile_name.to_string(), profile_path.to_string());

        // Set up event bus in view model
        app_state.set_event_bus(Box::new(SimpleEventBus::new()));
    }

    /// Apply initial ex commands from config file
    fn apply_initial_commands(&mut self, commands: &[String]) -> Result<()> {
        // Process ex commands from config through the unified command system
        for command in commands {
            tracing::info!("Applying config command: {}", command);

            // Set the ex command buffer with the config command
            self.app_state.set_ex_command_buffer(command.clone());

            // Create command context after setting the ex command buffer
            let context =
                crate::repl::unified_commands::CommandContext::from_app_state(&self.app_state);

            // Try to find a unified command that matches this ex command
            if let Some(unified_command) = self.unified_command_registry.process_key_event(
                crossterm::event::KeyEvent::new(
                    crossterm::event::KeyCode::Enter,
                    crossterm::event::KeyModifiers::NONE,
                ),
                crate::repl::models::pane_state::EditorMode::Command,
                &context,
            ) {
                tracing::info!("Executing unified command: {}", unified_command.name());

                // Execute the command with ExecutionContext
                let mut exec_context = ExecutionContext {
                    app_state: &mut self.app_state,
                    services: &mut self.services,
                };

                // Create a dummy KeyEvent for config commands
                let dummy_key_event = crossterm::event::KeyEvent::new(
                    crossterm::event::KeyCode::Null,
                    crossterm::event::KeyModifiers::empty(),
                );
                match unified_command.execute(dummy_key_event, &mut exec_context) {
                    Ok(view_events) => {
                        tracing::info!(
                            "Config command '{}' executed successfully with {} view events",
                            command,
                            view_events.len()
                        );
                        // Process view events but don't render (we're in init phase)
                        if !view_events.is_empty() {
                            // Just collect the events for now - rendering happens after init
                            tracing::debug!(
                                "Config command produced view events: {:?}",
                                view_events
                            );
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to execute config command '{}': {}", command, e);
                    }
                }
            } else {
                tracing::warn!(
                    "Config command '{}' not recognized by unified command system",
                    command
                );
            }

            // Clear the ex command buffer for the next command
            self.app_state.clear_ex_command_buffer();
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
    ///    d. Collect PostCommandActions from ViewModel changes
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
        self.view_renderer.render_full(&self.app_state)?;

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
        // Check for HTTP responses from the service (non-blocking)
        if let Some(ref mut http_service) = self.services.http {
            if let Some(response_msg) = http_service.poll_response() {
                self.handle_http_response(response_msg)?;
                return Ok(());
            }
        }

        // Poll for terminal events with 100ms timeout
        if !self.event_stream.poll(Duration::from_millis(100))? {
            return Ok(());
        }

        match self.event_stream.read()? {
            Event::Key(key_event) => self.handle_key_event_with_unified_first(key_event).await?,
            Event::Resize(width, height) => self.handle_resize_event(width, height)?,
            _ => {} // Ignore other events for now
        }

        Ok(())
    }

    /// Handle keyboard input events
    async fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        tracing::debug!("Received key event: {:?}", key_event);

        // Create command context snapshot for command processing
        let context = CommandContext::new(AppStateSnapshot::from_app_state(&self.app_state));

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

    /// Handle key events with unified command system first, then fall back to old system
    ///
    /// This allows gradual migration by checking unified commands first, then
    /// falling back to the existing command system if no unified command matches.
    async fn handle_key_event_with_unified_first(&mut self, key_event: KeyEvent) -> Result<()> {
        tracing::debug!("Processing key event with unified system: {:?}", key_event);

        // Create command context from current state
        let context =
            crate::repl::unified_commands::CommandContext::from_app_state(&self.app_state);
        let current_mode = self.app_state.get_mode();

        // Find the first relevant command
        if let Some(command) =
            self.unified_command_registry
                .process_key_event(key_event, current_mode, &context)
        {
            tracing::debug!("Executing unified command: {}", command.name());

            // Execute the command with ExecutionContext
            let mut exec_context = ExecutionContext {
                app_state: &mut self.app_state,
                services: &mut self.services,
            };
            let view_events = command.execute(key_event, &mut exec_context)?;

            tracing::debug!(
                "Command {} produced {} view events",
                command.name(),
                view_events.len()
            );

            // Process PostCommandActions directly without storing in AppState
            if !view_events.is_empty() {
                self.process_view_events(view_events)?;
            }
        } else {
            tracing::debug!(
                "No unified command found for key {:?} in mode {:?}",
                key_event,
                current_mode
            );
            // Fall back to old system for now
            self.handle_key_event(key_event).await?;
        }

        Ok(())
    }

    /// Handle HTTP response received from the service
    fn handle_http_response(&mut self, response_msg: HttpResponseMessage) -> Result<()> {
        let event = match response_msg {
            HttpResponseMessage::Success {
                request,
                response,
                url,
            } => {
                // Update response pane with the response
                self.app_state.set_response_from_http(&response);
                self.app_state.set_executing_request(false);

                let status = response.status().as_u16();
                let body = response.body().to_string();

                // Log the completion
                let duration_ms = response.duration_ms();
                tracing::info!(
                    "HTTP {} {} completed with status {} in {}ms",
                    request.method().unwrap_or(&"GET".to_string()),
                    url,
                    status,
                    duration_ms
                );

                ModelEvent::HttpResponseReceived { status, body }
            }
            HttpResponseMessage::Error { message } => {
                // Update response with error message
                self.app_state.set_response(0, message.clone());
                self.app_state.set_executing_request(false);

                tracing::error!("HTTP request failed: {}", message);

                ModelEvent::StatusMessageSet { message }
            }
        };

        // Process the event through the normal flow
        self.process_model_event_internal(event)?;

        // Switch to response pane to show results
        self.app_state.switch_to_response_pane();

        // Trigger re-render to show the response
        self.render_if_needed()?;

        Ok(())
    }

    /// Handle terminal resize events
    fn handle_resize_event(&mut self, width: u16, height: u16) -> Result<()> {
        // Synchronize both model and view with new terminal dimensions
        self.app_state.update_terminal_size(width, height);
        self.view_renderer.update_size(width, height);
        // Full redraw required after resize to handle layout changes
        self.view_renderer.render_full(&self.app_state)?;
        Ok(())
    }

    /// Perform rendering with throttling to prevent ghost cursors
    fn render_if_needed(&mut self) -> Result<()> {
        let now = std::time::Instant::now();
        let min_render_interval = Duration::from_micros(500);

        if now.duration_since(self.last_render_time) < min_render_interval {
            return Ok(());
        }

        let view_events = self.app_state.collect_pending_view_events();
        self.process_view_events(view_events)?;
        self.last_render_time = now;

        Ok(())
    }

    /// Apply a command event to the view model
    ///
    /// HIGH-LEVEL LOGIC FLOW:
    /// This method serves as the command processor that translates semantic commands
    /// into specific ViewModel operations. Each CommandEvent type maps to one or more
    /// ViewModel method calls that modify application state and emit PostCommandActions.
    ///
    /// ARCHITECTURAL PATTERN:
    /// - Commands are processed atomically (all-or-nothing)
    /// - State changes emit PostCommandActions for selective rendering
    /// - Complex commands (like ex commands) can generate nested events
    /// - HTTP requests are handled asynchronously with status updates
    async fn apply_command_event(&mut self, event: CommandEvent) -> Result<()> {
        match event {
            CommandEvent::CursorMoveRequested { direction, amount } => {
                for _ in 0..amount {
                    match direction {
                        MovementDirection::Left => self.app_state.move_cursor_left()?,
                        MovementDirection::Right => self.app_state.move_cursor_right()?,
                        MovementDirection::Up => self.app_state.move_cursor_up()?,
                        MovementDirection::Down => self.app_state.move_cursor_down()?,
                        MovementDirection::LineEnd => {
                            self.app_state.move_cursor_to_end_of_line()?
                        }
                        MovementDirection::LineEndForAppend => {
                            self.app_state.move_cursor_to_line_end_for_append()?
                        }
                        MovementDirection::LineStart => {
                            self.app_state.move_cursor_to_start_of_line()?
                        }
                        MovementDirection::ScrollLeft => {
                            self.app_state.scroll_horizontally(-1, amount)?
                        }
                        MovementDirection::ScrollRight => {
                            self.app_state.scroll_horizontally(1, amount)?
                        }
                        MovementDirection::DocumentStart => {
                            self.app_state.move_cursor_to_document_start()?
                        }
                        MovementDirection::DocumentEnd => {
                            self.app_state.move_cursor_to_document_end()?
                        }
                        MovementDirection::WordForward => {
                            self.app_state.move_cursor_to_next_word()?
                        }
                        MovementDirection::WordBackward => {
                            self.app_state.move_cursor_to_previous_word()?
                        }
                        MovementDirection::WordEnd => {
                            self.app_state.move_cursor_to_end_of_word()?
                        }
                        MovementDirection::LineNumber(line_number) => {
                            self.app_state.move_cursor_to_line(line_number)?
                        }
                        MovementDirection::PageDown => self.app_state.move_cursor_page_down()?,
                        MovementDirection::PageUp => self.app_state.move_cursor_page_up()?,
                        MovementDirection::HalfPageDown => {
                            self.app_state.move_cursor_half_page_down()?
                        }
                        MovementDirection::HalfPageUp => {
                            self.app_state.move_cursor_half_page_up()?
                        }
                    }
                }
            }
            CommandEvent::CursorPositionRequested { position } => {
                self.app_state.set_cursor_position(position)?;
            }
            CommandEvent::TextInsertRequested { text, position: _ } => {
                // Check if we're in Visual Block Insert mode with multiple cursors
                // NOTE: Multi-cursor text insertion is now handled by MultiCursorTextInsertCommand
                // in the unified command system, which intercepts character input before
                // it reaches this legacy TextInsertRequested event.
                if self.app_state.is_in_visual_block_insert_mode() {
                    // Legacy multi-cursor insertion temporarily disabled during migration
                    tracing::debug!("VisualBlockInsert mode detected - should be handled by MultiCursorTextInsertCommand");
                    // For now, fall back to regular insertion to prevent breaking functionality
                    self.app_state.insert_text(&text)?;
                } else {
                    self.app_state.insert_text(&text)?;
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
                // NOTE: Multi-cursor text deletion is now handled by MultiCursorTextDeleteCommand
                // in the unified command system, which intercepts delete keys in VisualBlockInsert mode
                if self.app_state.is_in_visual_block_insert_mode() {
                    // Multi-cursor deletion now handled by unified command system
                    tracing::debug!("TextDeleteRequested in VisualBlockInsert mode - should be handled by MultiCursorTextDeleteCommand");
                } else {
                    for i in 0..amount {
                        match direction {
                            MovementDirection::Left => {
                                tracing::debug!(
                                    "🗑️  Attempting delete_char_before_cursor (iteration {})",
                                    i + 1
                                );
                                match self.app_state.delete_char_before_cursor() {
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
                                match self.app_state.delete_char_after_cursor() {
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
                match self.app_state.change_mode(new_mode) {
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
                let previous_mode = self.app_state.get_previous_mode();
                tracing::debug!("Restoring previous mode: {:?}", previous_mode);
                match self.app_state.change_mode(previous_mode) {
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
                Pane::Request => self.app_state.switch_to_request_pane(),
                Pane::Response => self.app_state.switch_to_response_pane(),
            },
            CommandEvent::HttpRequestRequested { .. } => {
                // This is now handled by HttpExecuteCommand
                tracing::debug!("HTTP request received via old command path - ignoring");
            }
            CommandEvent::TerminalResizeRequested { width, height } => {
                self.app_state.update_terminal_size(width, height);
                self.view_renderer.update_size(width, height);
            }
            CommandEvent::QuitRequested => {
                self.should_quit = true;
            }
            CommandEvent::ExCommandCharRequested { ch } => {
                self.app_state.add_ex_command_char(ch)?;
            }
            CommandEvent::ExCommandBackspaceRequested => {
                self.app_state.backspace_ex_command()?;
            }
            CommandEvent::ExCommandExecuteRequested => {
                // Ex commands are now handled by the unified command system
                // This legacy handler just clears the buffer and exits command mode
                tracing::debug!(
                    "Legacy ex command execute request - clearing buffer and exiting command mode"
                );
                self.app_state.clear_ex_command_buffer();
                let previous_mode = self.app_state.get_previous_mode();
                self.app_state.change_mode(previous_mode)?;
            }
            CommandEvent::ShowProfileRequested => {
                // Show profile information directly (old ShowProfileCommand logic)
                let profile_name = self.app_state.get_profile_name();
                let profile_path = self.app_state.get_profile_path();

                tracing::info!("Showing profile: {} at {}", profile_name, profile_path);

                let message = format!("[{profile_name}] in {profile_path}");
                self.app_state.set_status_message(message);

                self.process_view_events(vec![PostCommandAction::StatusBarUpdateRequired])?;
            }
            CommandEvent::SettingChangeRequested { setting, value } => {
                // Now handled by SettingChangeCommand
                use crate::repl::unified_commands::system::setting_change::SettingChangeCommand;
                let command = SettingChangeCommand::new(setting, value);
                let mut exec_context = ExecutionContext {
                    app_state: &mut self.app_state,
                    services: &mut self.services,
                };
                // Create a dummy KeyEvent for SettingChange (no key event in this context)
                let dummy_key_event = crossterm::event::KeyEvent::new(
                    crossterm::event::KeyCode::Null,
                    crossterm::event::KeyModifiers::empty(),
                );
                if let Ok(view_events) = command.execute(dummy_key_event, &mut exec_context) {
                    self.process_view_events(view_events)?;
                }
            }
            CommandEvent::YankSelectionRequested => {
                // Now handled by YankSelectionCommand in unified_commands
                // self.handle_yank_selection()?;
            }
            CommandEvent::DeleteSelectionRequested => {
                // Now handled by DeleteSelectionCommand in unified_commands
                tracing::debug!(
                    "DeleteSelectionRequested received via old command path - ignoring"
                );
            }
            CommandEvent::CutSelectionRequested => {
                // Now handled by CutSelectionCommand in unified_commands
                tracing::debug!("CutSelectionRequested received via old command path - ignoring");
            }
            CommandEvent::CutCharacterRequested => {
                // Now handled by CutCharacterCommand in unified_commands
                tracing::debug!("CutCharacterRequested received via old command path - ignoring");
            }
            CommandEvent::CutToEndOfLineRequested => {
                // Now handled by CutToEndOfLineCommand in unified_commands
                tracing::debug!("CutToEndOfLineRequested received via old command path - ignoring");
            }
            CommandEvent::CutCurrentLineRequested => {
                // Now handled by CutCurrentLineCommand in unified_commands
                tracing::debug!("CutCurrentLineRequested received via old command path - ignoring");
            }
            CommandEvent::YankCurrentLineRequested => {
                tracing::debug!(
                    "YankCurrentLineRequested received via old command path - ignoring"
                );
            }
            CommandEvent::VisualBlockInsertRequested => {
                // Now handled by VisualBlockInsertCommand in unified_commands
                tracing::debug!(
                    "VisualBlockInsertRequested received via old command path - ignoring"
                );
            }
            CommandEvent::VisualBlockAppendRequested => {
                // Now handled by VisualBlockAppendCommand in unified_commands
                tracing::debug!(
                    "VisualBlockAppendRequested received via old command path - ignoring"
                );
            }
            CommandEvent::ExitVisualBlockInsertRequested => {
                // Now handled by ExitVisualBlockInsertCommand in unified commands
                tracing::debug!(
                    "ExitVisualBlockInsertRequested received via old command path - ignoring"
                );
            }
            CommandEvent::RepeatVisualSelectionRequested => {
                // Now handled by RepeatVisualSelectionCommand in unified_commands
                tracing::debug!(
                    "RepeatVisualSelectionRequested received via old command path - ignoring"
                );
            }
            CommandEvent::PasteAfterRequested => {
                // Now handled by PasteAfterCommand in unified_commands
                tracing::debug!(
                    "PasteAfterRequested received via old command path - ignoring (handled by unified system)"
                );
            }
            CommandEvent::PasteAtCursorRequested => {
                // Now handled by PasteAtCursorCommand in unified_commands
                tracing::debug!(
                    "PasteAtCursorRequested received via old command path - ignoring (handled by unified system)"
                );
            }
            CommandEvent::ChangeSelectionRequested => {
                // Now handled by ChangeSelectionCommand in unified commands
                tracing::debug!(
                    "ChangeSelectionRequested received via old command path - ignoring"
                );
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
    ///
    /// Get reference to view model (for testing)
    pub fn app_state(&self) -> &AppState {
        &self.app_state
    }

    /// Get mutable reference to view model (for testing)
    pub fn app_state_mut(&mut self) -> &mut AppState {
        &mut self.app_state
    }

    /// Process view events for selective rendering instead of always doing full redraws
    ///
    /// HIGH-LEVEL LOGIC FLOW:
    /// 1. Collect and group PostCommandActions to minimize redundant renders
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
        view_events: Vec<crate::repl::view_models::PostCommandAction>,
    ) -> Result<()> {
        use crate::repl::view_models::PostCommandAction;

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
                PostCommandAction::FullRedrawRequired => {
                    needs_full_redraw = true;
                    // Full redraw overrides all other events
                    break;
                }
                PostCommandAction::CurrentAreaRedrawRequired => {
                    needs_current_area_redraw = true;
                }
                PostCommandAction::SecondaryAreaRedrawRequired => {
                    needs_secondary_area_redraw = true;
                }
                PostCommandAction::CurrentAreaPartialRedrawRequired { start_line } => {
                    // Only add partial redraw if we're not already doing a full current area redraw
                    if !needs_current_area_redraw {
                        let current_pane = self.app_state.get_current_pane();
                        partial_redraws
                            .entry(current_pane)
                            .and_modify(|line| *line = (*line).min(start_line))
                            .or_insert(start_line);
                    }
                }
                PostCommandAction::SecondaryAreaPartialRedrawRequired { start_line } => {
                    // Only add partial redraw if we're not already doing a full secondary area redraw
                    if !needs_secondary_area_redraw {
                        let current_pane = self.app_state.get_current_pane();
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
                PostCommandAction::StatusBarUpdateRequired => {
                    needs_status_bar = true;
                }
                PostCommandAction::PositionIndicatorUpdateRequired => {
                    // Handle position indicator separately for minimal flickering
                    self.view_renderer
                        .render_position_indicator(&self.app_state)?;
                }
                PostCommandAction::ActiveCursorUpdateRequired => {
                    needs_cursor_update = true;
                }
                PostCommandAction::CurrentAreaScrollChanged { .. } => {
                    needs_current_area_redraw = true;
                    // Ensure cursor is updated after scroll to prevent ghost cursor
                    needs_cursor_update = true;
                }
                PostCommandAction::SecondaryAreaScrollChanged { .. } => {
                    needs_secondary_area_redraw = true;
                }
                PostCommandAction::FocusSwitched => {
                    // Focus switch requires cursor update and status bar update
                    needs_cursor_update = true;
                    needs_status_bar = true;
                }
                PostCommandAction::RequestContentChanged => {
                    // Request content changed - redraw current area if we're in request pane
                    if self.app_state.is_in_request_pane() {
                        needs_current_area_redraw = true;
                    } else {
                        needs_secondary_area_redraw = true;
                    }
                }
                PostCommandAction::ResponseContentChanged => {
                    // Response content changed - redraw current area if we're in response pane
                    if self.app_state.is_in_response_pane() {
                        needs_current_area_redraw = true;
                    } else {
                        needs_secondary_area_redraw = true;
                    }
                }
                PostCommandAction::AllContentAreasRedrawRequired => {
                    needs_current_area_redraw = true;
                    needs_secondary_area_redraw = true;
                }
                PostCommandAction::QuitRequested => {
                    self.should_quit = true;
                }
            }
        }

        // Process events in order of efficiency
        if needs_full_redraw {
            self.view_renderer.render_full(&self.app_state)?;
        } else {
            // Selective rendering - renderer handles cursor visibility
            let has_content_updates = needs_current_area_redraw
                || needs_secondary_area_redraw
                || !partial_redraws.is_empty();
            if has_content_updates {
                tracing::debug!(
                    "view_model: content updates - current: {}, secondary: {}, partial: {:?}",
                    needs_current_area_redraw,
                    needs_secondary_area_redraw,
                    partial_redraws.keys().collect::<Vec<_>>()
                );
                // Cursor hiding is now handled by each render method in the renderer
                // to ensure consistent behavior and prevent ghost cursors
            }

            // Render current area if needed
            if needs_current_area_redraw {
                let current_pane = self.app_state.get_current_pane();
                self.view_renderer
                    .render_pane(&self.app_state, current_pane)?;
            }

            // Render secondary area if needed
            if needs_secondary_area_redraw {
                let current_pane = self.app_state.get_current_pane();
                let secondary_pane = match current_pane {
                    Pane::Request => Pane::Response,
                    Pane::Response => Pane::Request,
                };
                self.view_renderer
                    .render_pane(&self.app_state, secondary_pane)?;
            }

            // Handle partial pane redraws
            for (pane, start_line) in &partial_redraws {
                self.view_renderer
                    .render_pane_partial(&self.app_state, *pane, *start_line)?;
            }

            if needs_status_bar {
                self.view_renderer.render_status_bar(&self.app_state)?;
            }

            // Always render cursor after any pane redraw to prevent ghost cursors
            if needs_cursor_update || has_content_updates {
                tracing::debug!("view_model: rendering cursor after content updates");
                self.view_renderer.render_cursor(&self.app_state)?;
            }
        }

        Ok(())
    }

    /// Handle text insertion for multi-cursor Visual Block Insert mode
    ///
    /// Inserts the same text at all cursor positions simultaneously,
    /// providing live feedback across all selected lines.
    ///
    /// NOTE: This function has been migrated to MultiCursorTextInsertCommand
    /// but is temporarily commented out due to architectural limitations.
    /// The unified command system cannot access KeyEvent characters in execute(),
    /// which prevents full migration of character input handling.
    #[allow(unused)]
    fn handle_multi_cursor_text_insert(&mut self, text: &str) -> Result<()> {
        let cursor_positions = self.app_state.get_visual_block_insert_cursors().to_vec();

        if cursor_positions.is_empty() {
            // Fallback to regular insert if no cursors are set
            return self.app_state.insert_text(text);
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
            self.app_state.set_cursor_position(*position)?;
            self.app_state.insert_text(text)?;
        }

        // Update all cursor positions to reflect the inserted text
        let text_len = text.chars().count(); // Handle multi-byte characters correctly
        let updated_positions: Vec<LogicalPosition> = cursor_positions
            .iter()
            .map(|pos| LogicalPosition::new(pos.line, pos.column + text_len))
            .collect();

        // Set the primary cursor to the first position before updating positions
        if let Some(first_pos) = updated_positions.first() {
            self.app_state.set_cursor_position(*first_pos)?;
        }

        self.app_state
            .update_visual_block_insert_cursors(updated_positions);

        tracing::debug!("Multi-cursor text insert completed, updated cursor positions");
        Ok(())
    }

    // MIGRATED: handle_multi_cursor_text_delete function has been migrated to MultiCursorTextDeleteCommand
    // The unified command system now handles multi-cursor text deletion directly
    // keeping this commented for reference during transition
    /*
    /// Handle text deletion for multi-cursor Visual Block Insert mode
    fn handle_multi_cursor_text_delete(
        &mut self,
        amount: usize,
        direction: MovementDirection,
    ) -> Result<()> {
        let cursor_positions = self.app_state.get_visual_block_insert_cursors().to_vec();
        let start_columns = self
            .app_state
            .get_visual_block_insert_start_columns()
            .to_vec();

        if cursor_positions.is_empty() {
            // Fallback to regular delete if no cursors are set
            for _ in 0..amount {
                match direction {
                    MovementDirection::Left => {
                        self.app_state.delete_char_before_cursor()?;
                    }
                    MovementDirection::Right => {
                        self.app_state.delete_char_after_cursor()?;
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
            self.app_state.set_cursor_position(*position)?;

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
                        self.app_state.delete_char_before_cursor()?;
                    }
                    MovementDirection::Right => {
                        self.app_state.delete_char_after_cursor()?;
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
            self.app_state.set_cursor_position(*first_pos)?;
        }

        self.app_state
            .update_visual_block_insert_cursors(updated_positions);

        tracing::debug!("Multi-cursor text delete completed, updated cursor positions");
        Ok(())
    }
    */

    /// Process a single key event without running the full event loop (for testing)
    pub async fn process_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        tracing::debug!("Processing key event: {:?}", key_event);
        tracing::debug!("AppViewModel: process_key_event called with {key_event:?}");

        // Create command context from current state
        tracing::debug!("AppViewModel: Creating command context");
        let context = CommandContext::new(AppStateSnapshot::from_app_state(&self.app_state));
        tracing::debug!("AppViewModel: Command context created");

        // Process through command registry
        tracing::debug!("AppViewModel: About to call command_registry.process_event");
        if let Ok(events) = self.command_registry.process_event(key_event, &context) {
            tracing::debug!(
                "AppViewModel: Command events generated: {} events",
                events.len()
            );
            tracing::debug!("Command events generated: {:?}", events);
            if !events.is_empty() {
                // Apply events to view model (this will emit appropriate PostCommandActions)
                tracing::debug!(
                    "AppViewModel: About to apply {} command events",
                    events.len()
                );
                for (i, event) in events.iter().enumerate() {
                    tracing::debug!(
                        "AppViewModel: Applying event {}/{}: {:?}",
                        i + 1,
                        events.len(),
                        event
                    );
                    self.apply_command_event(event.clone()).await?;
                    tracing::debug!(
                        "AppViewModel: Applied event {}/{} successfully",
                        i + 1,
                        events.len()
                    );
                }
                tracing::debug!("AppViewModel: All command events applied successfully");

                // Render after processing key events
                self.view_renderer.render_full(&self.app_state)?;
            } else {
                tracing::debug!("AppViewModel: No command events generated");
            }
        } else {
            tracing::warn!("AppViewModel: Failed to process key event: {key_event:?}");
        }

        tracing::debug!("AppViewModel: process_key_event completed successfully");
        Ok(())
    }

    /// Check if the application should quit (for testing)
    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    /// Execute a Command using the new Command Pattern
    ///
    /// This method allows execution of Commands that emit PostCommandActions
    /// alongside the existing command system. This enables gradual migration.
    pub fn execute_command(&mut self, command: Box<dyn Command>) -> Result<()> {
        tracing::debug!("Executing command: {}", command.name());

        let mut exec_context = ExecutionContext {
            app_state: &mut self.app_state,
            services: &mut self.services,
        };
        // Create a dummy KeyEvent for execute_command (no key event in this context)
        let dummy_key_event = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Null,
            crossterm::event::KeyModifiers::empty(),
        );
        let view_events = command.execute(dummy_key_event, &mut exec_context)?;

        tracing::debug!(
            "Command {} produced {} view events",
            command.name(),
            view_events.len()
        );

        // Process PostCommandActions directly without storing in AppState
        if !view_events.is_empty() {
            self.process_view_events(view_events)?;
        }

        Ok(())
    }

    /// Process a ModelEvent and convert it to actual state changes
    ///
    /// This is the bridge between semantic ModelEvents and the actual
    /// application state changes. It handles status messages, logging,
    /// and any necessary side effects.
    #[cfg(test)]
    pub fn process_model_event(&mut self, event: ModelEvent) -> Result<()> {
        self.process_model_event_internal(event)
    }

    /// Internal implementation of process_model_event
    fn process_model_event_internal(&mut self, event: ModelEvent) -> Result<()> {
        match event {
            ModelEvent::TextYanked {
                pane,
                text,
                yank_type,
            } => {
                // Store in yank buffer using YankService
                // (No need to convert types anymore - yank_type is already NewYankType)
                self.services.yank.yank(text.clone(), yank_type)?;

                // Create appropriate status message
                let char_count = text.chars().count();
                let line_count = text.lines().count();
                let message = match yank_type {
                    NewYankType::Character => {
                        if line_count > 1 {
                            format!("{line_count} lines yanked (character-wise)")
                        } else {
                            format!("{char_count} characters yanked")
                        }
                    }
                    NewYankType::Line => {
                        format!("{line_count} lines yanked")
                    }
                    NewYankType::Block => {
                        format!("Block yanked ({line_count} lines, {char_count} chars)")
                    }
                };

                self.app_state.set_status_message(message);

                tracing::info!(
                    "Yanked {} characters ({} lines) to buffer as {:?} from {:?}",
                    char_count,
                    line_count,
                    yank_type,
                    pane
                );
            }

            ModelEvent::ModeChanged { old_mode, new_mode } => {
                self.app_state.change_mode(new_mode)?;
                tracing::debug!("Mode changed from {:?} to {:?}", old_mode, new_mode);
            }

            ModelEvent::SelectionCleared { pane } => {
                // Clear the visual selection in the ViewModel
                self.app_state.clear_visual_selection()?;
                tracing::debug!("Selection cleared for {:?}", pane);
            }

            ModelEvent::StatusMessageSet { message } => {
                self.app_state.set_status_message(message);
            }

            ModelEvent::StatusMessageCleared => {
                self.app_state.set_status_message(String::new());
            }

            ModelEvent::HttpRequestStarted { method, url } => {
                // Execute the HTTP request through the service
                if let Some(http_service) = self.services.http.as_mut() {
                    // Get the full request text and execute it
                    let request_text = self.app_state.get_request_text();
                    self.app_state.set_executing_request(true);
                    http_service.execute_async(request_text);
                    tracing::info!("HTTP request initiated: {method} {url}");
                } else {
                    tracing::error!("HTTP service not available");
                    self.app_state
                        .set_status_message("HTTP service not configured".to_string());
                }
            }

            ModelEvent::HttpResponseReceived { status, body } => {
                // Update response pane with received data
                self.app_state.set_response(status, body);
                self.app_state.set_executing_request(false);
                self.app_state.switch_to_response_pane();

                let status_msg = if (200..300).contains(&status) {
                    format!("Request completed: {status}")
                } else {
                    format!("Request failed: {status}")
                };
                self.app_state.set_status_message(status_msg);
            }

            // Handle other events as we implement them
            _ => {
                tracing::debug!("ModelEvent not yet implemented: {:?}", event);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd_args::CommandLineArgs;
    use crate::repl::models::pane_state::{EditorMode, Pane};

    #[test]
    fn app_view_model_should_create() {
        use crate::repl::io::mock::{MockEventStream, MockRenderStream};

        let cmd_args = CommandLineArgs::parse_from(["test"]);
        let config = AppConfig::from_args(cmd_args);
        let view_model = AppViewModel::with_io_streams(
            config,
            MockEventStream::empty(),
            MockRenderStream::new(),
        );
        assert!(view_model.is_ok());

        let view_model = view_model.unwrap();
        assert_eq!(view_model.app_state().get_mode(), EditorMode::Normal);
        assert_eq!(view_model.app_state().get_current_pane(), Pane::Request);
    }

    #[test]
    fn app_controller_should_execute_yank_selection_command() {
        use crate::repl::io::mock::{MockEventStream, MockRenderStream};
        use crate::repl::unified_commands::yank::YankSelectionCommand;

        let cmd_args = CommandLineArgs::parse_from(["test"]);
        let config = AppConfig::from_args(cmd_args);
        let mut view_model = AppViewModel::with_io_streams(
            config,
            MockEventStream::empty(),
            MockRenderStream::new(),
        )
        .unwrap();

        // Test YankSelectionCommand in Normal mode (should succeed but with no selection message)
        let command = Box::new(YankSelectionCommand::new());
        let result = view_model.execute_command(command);

        // YankSelectionCommand should succeed (returns status bar update for "no selection")
        assert!(
            result.is_ok(),
            "Command should succeed even without selection"
        );

        // Verify we're still in Normal mode
        assert_eq!(view_model.app_state().get_mode(), EditorMode::Normal);
    }

    #[tokio::test]
    async fn app_controller_should_use_unified_command_system() {
        use crate::repl::io::mock::{MockEventStream, MockRenderStream};
        use crossterm::event::{KeyCode, KeyModifiers};

        let cmd_args = CommandLineArgs::parse_from(["test"]);
        let config = AppConfig::from_args(cmd_args);
        let mut view_model = AppViewModel::with_io_streams(
            config,
            MockEventStream::empty(),
            MockRenderStream::new(),
        )
        .unwrap();

        // Verify unified command registry is initialized
        assert!(view_model.unified_command_registry.command_count() > 0);

        // Test 'y' key in Normal mode - should fall back to old system (no unified command)
        let y_key = crossterm::event::KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE);
        let result = view_model.handle_key_event_with_unified_first(y_key).await;
        assert!(
            result.is_ok(),
            "Unified command system should handle key events gracefully"
        );

        // Verify old system handled it (y in Normal mode goes to YPrefix mode)
        assert_eq!(view_model.app_state().get_mode(), EditorMode::YPrefix);
    }

    #[test]
    fn app_view_model_should_apply_config_commands() {
        use crate::repl::io::mock::{MockEventStream, MockRenderStream};

        // Create a config with initial commands
        let test_commands = vec!["set wrap on".to_string(), "set number on".to_string()];
        let config = AppConfig::new(
            "test".to_string(),
            "/nonexistent/profile/path".to_string(),
            test_commands,
        );

        let view_model = AppViewModel::with_io_streams(
            config,
            MockEventStream::empty(),
            MockRenderStream::new(),
        );

        assert!(
            view_model.is_ok(),
            "ViewModel should be created successfully"
        );
        let view_model = view_model.unwrap();

        // Verify that wrap is enabled (this would be set by the "set wrap on" command)
        assert!(
            view_model.app_state().pane_manager.is_wrap_enabled(),
            "Wrap should be enabled from config command"
        );
    }
}
