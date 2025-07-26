//! # REPL Application Controller
//!
//! The controller orchestrates the REPL components and manages the event loop.
//! It's responsible for connecting user input to commands and coordinating view updates.

use crate::repl::{
    commands::CommandRegistry,
    events::SimpleEventBus,
    view_models::ViewModel,
    views::{TerminalRenderer, ViewRenderer},
};
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
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
    /// Create new application controller
    pub fn new() -> Result<Self> {
        let mut view_model = ViewModel::new();
        let view_renderer = TerminalRenderer::new()?;
        let command_registry = CommandRegistry::new();
        let event_bus = SimpleEventBus::new();

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
                        // Check for quit conditions first
                        if self.should_quit_for_key(&key_event) {
                            self.should_quit = true;
                            break;
                        }

                        // Process through command registry
                        if let Ok(handled) = self
                            .command_registry
                            .process_event(key_event, &mut self.view_model)
                        {
                            if handled {
                                // Command was processed, view will be updated via events
                                // For now, we'll do a simple full render
                                // In a more sophisticated implementation, we'd only update what changed
                                self.view_renderer.render_full(&self.view_model)?;
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

    /// Check if we should quit for the given key event
    fn should_quit_for_key(&self, key_event: &crossterm::event::KeyEvent) -> bool {
        // Ctrl+C to quit
        matches!(key_event.code, KeyCode::Char('c'))
            && key_event.modifiers.contains(KeyModifiers::CONTROL)
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

impl Default for AppController {
    fn default() -> Self {
        Self::new().expect("Failed to create AppController")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::events::{EditorMode, Pane};

    #[test]
    fn app_controller_should_create() {
        if crossterm::terminal::size().is_ok() {
            let controller = AppController::new();
            assert!(controller.is_ok());

            let controller = controller.unwrap();
            assert_eq!(controller.view_model().get_mode(), EditorMode::Normal);
            assert_eq!(controller.view_model().get_current_pane(), Pane::Request);
        }
    }

    #[test]
    fn app_controller_should_quit_on_ctrl_c() {
        if let Ok(controller) = AppController::new() {
            let key_event =
                crossterm::event::KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);

            assert!(controller.should_quit_for_key(&key_event));
        }
    }

    #[test]
    fn app_controller_should_not_quit_on_regular_keys() {
        if let Ok(controller) = AppController::new() {
            let key_event = crossterm::event::KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);

            assert!(!controller.should_quit_for_key(&key_event));
        }
    }
}
