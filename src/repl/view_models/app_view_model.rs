//! # REPL Application Controller
//!
//! The controller orchestrates the REPL components and manages the event loop.
//! It's responsible for connecting user input to commands and coordinating view updates.

use crate::config::AppConfig;
use crate::repl::{
    io::{EventStream, RenderStream},
    models::app_state::AppState,
    models::pane_state::Pane,
    models::LogicalPosition,
    services::{HttpResponseMessage, Services},
    view_models::commands::{DynamicCommandRegistry, ExecutionContext},
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
    // New dynamic command system (checks first, falls back to old system)
    unified_command_registry: DynamicCommandRegistry,
    event_stream: ES,
    should_quit: bool,
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

        let unified_command_registry = DynamicCommandRegistry::new();

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
            unified_command_registry,
            event_stream,
            should_quit: false,
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
                crate::repl::view_models::commands::CommandContext::from_app_state(&self.app_state);

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
            Event::Key(key_event) => self.handle_key_event(key_event).await?,
            Event::Resize(width, height) => self.handle_resize_event(width, height)?,
            _ => {} // Ignore other events for now
        }

        Ok(())
    }

    /// Handle key events with unified command system
    async fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        tracing::debug!("Processing key event with unified system: {:?}", key_event);

        // Create command context from current state
        let context =
            crate::repl::view_models::commands::CommandContext::from_app_state(&self.app_state);
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
        }

        Ok(())
    }

    /// Handle HTTP response received from the service
    fn handle_http_response(&mut self, response_msg: HttpResponseMessage) -> Result<()> {
        match response_msg {
            HttpResponseMessage::Success {
                request,
                response,
                url,
            } => {
                // Get the raw body content
                let raw_body = response.body();
                
                // Debug logging for auto-format
                let autoformat_enabled = self.app_state.is_autoformat_enabled();
                let is_json_content = self.app_state.is_json_content_type(&response);
                tracing::info!("=== AUTO-FORMAT DEBUG ===");
                tracing::info!("Auto-format enabled: {}", autoformat_enabled);
                tracing::info!("Is JSON content: {}", is_json_content);
                tracing::info!("Response content-type: {:?}", response.headers().get("content-type"));
                tracing::info!("Raw body (first 200 chars): {}", &raw_body[..std::cmp::min(200, raw_body.len())]);

                // Apply JSON formatting if auto-format is enabled and content-type indicates JSON
                let formatted_body = if autoformat_enabled && is_json_content {
                    tracing::info!("✓ APPLYING JSON formatting...");
                    let formatted = self.services.json_formatter.format_json(raw_body);
                    tracing::info!("✓ Formatted body (first 200 chars): {}", &formatted[..std::cmp::min(200, formatted.len())]);
                    formatted
                } else {
                    tracing::info!("✗ NOT applying JSON formatting - autoformat: {}, is_json: {}", autoformat_enabled, is_json_content);
                    raw_body.to_string()
                };

                // Update response pane with the formatted response
                self.app_state
                    .set_response_from_http(&response, formatted_body);
                self.app_state.set_executing_request(false);

                let status = response.status().as_u16();
                let duration_ms = response.duration_ms();

                // Log the completion
                tracing::info!(
                    "HTTP {} {} completed with status {} in {}ms",
                    request.method().unwrap_or(&"GET".to_string()),
                    url,
                    status,
                    duration_ms
                );

                // Set status message
                let status_msg = if (200..300).contains(&status) {
                    format!("Request completed: {status}")
                } else {
                    format!("Request failed: {status}")
                };
                self.app_state.set_status_message(status_msg);
            }
            HttpResponseMessage::Error { message } => {
                // Update response with error message
                self.app_state.set_response(0, message.clone());
                self.app_state.set_executing_request(false);

                tracing::error!("HTTP request failed: {}", message);

                // Set status message
                self.app_state
                    .set_status_message(format!("Request failed: {message}"));
            }
            HttpResponseMessage::Cancelled { message } => {
                // Clear response by setting empty content
                self.app_state.set_response(0, "".to_string());
                self.app_state.set_executing_request(false);

                tracing::info!("HTTP request cancelled: {}", message);

                // Set status message
                self.app_state
                    .set_status_message("Request cancelled".to_string());
            }
        }

        // Switch to response pane to show results
        // self.app_state.switch_to_response_pane();

        // Generate PostCommandActions for the response update
        let post_actions = vec![
            PostCommandAction::SecondaryAreaRedrawRequired,
            PostCommandAction::StatusBarUpdateRequired,
            PostCommandAction::FullRedrawRequired,
        ];

        // Process the actions to trigger rendering
        self.process_view_events(post_actions)?;

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
    fn process_view_events(&mut self, view_events: Vec<PostCommandAction>) -> Result<()> {
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

    /// Check if the application should quit (for testing)
    pub fn should_quit(&self) -> bool {
        self.should_quit
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
        assert_eq!(view_model.app_state.get_mode(), EditorMode::Normal);
        assert_eq!(view_model.app_state.get_current_pane(), Pane::Request);
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
            view_model.app_state.pane_manager.is_wrap_enabled(),
            "Wrap should be enabled from config command"
        );
    }
}
