//! Debug module for visual mode issues

#[cfg(test)]
mod tests {
    use crate::repl::{
        commands::{
            Command, CommandContext, CommandRegistry, EnterVisualModeCommand, ViewModelSnapshot,
        },
        events::EditorMode,
        view_models::ViewModel,
    };
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn debug_visual_mode_runtime_issue() {
        tracing::info!("=== Debugging Visual Mode Issue ===");

        // Create a view model in the same state as runtime
        let mut view_model = ViewModel::new();

        // Set up a realistic terminal size
        view_model.update_terminal_size(80, 24);

        tracing::debug!("Initial mode: {:?}", view_model.get_mode());
        tracing::debug!("Initial pane: {:?}", view_model.get_current_pane());

        // Create command registry (same as runtime)
        let registry = CommandRegistry::new();

        // Create the exact same context that would be created at runtime
        let context = CommandContext::new(ViewModelSnapshot::from_view_model(&view_model));

        tracing::debug!(
            "Context state: mode={:?}, pane={:?}",
            context.state.current_mode,
            context.state.current_pane
        );

        // Create the 'v' key event (same as runtime)
        let v_event = KeyEvent::new(KeyCode::Char('v'), KeyModifiers::empty());

        tracing::debug!("Processing 'v' key event: {:?}", v_event);

        // Test the visual mode command directly first
        let visual_cmd = EnterVisualModeCommand;
        let is_relevant = visual_cmd.is_relevant(&context, &v_event);

        tracing::debug!("EnterVisualModeCommand.is_relevant(): {}", is_relevant);

        if !is_relevant {
            tracing::error!("Visual mode command not relevant!");
            tracing::debug!("Required conditions:");
            tracing::debug!(
                "  - event.code == 'v': {}",
                matches!(v_event.code, KeyCode::Char('v'))
            );
            tracing::debug!(
                "  - current_mode == Normal: {}",
                context.state.current_mode == EditorMode::Normal
            );
            tracing::debug!("  - no modifiers: {}", v_event.modifiers.is_empty());
            tracing::debug!("  - current_mode = {:?}", context.state.current_mode);
            tracing::debug!("  - modifiers = {:?}", v_event.modifiers);
        } else {
            tracing::info!("Visual mode command is relevant");

            // Test executing the command directly
            match visual_cmd.execute(v_event, &context) {
                Ok(events) => {
                    tracing::info!("Direct command execution generated: {:?}", events);
                }
                Err(e) => {
                    tracing::error!("Direct command execution failed: {}", e);
                }
            }
        }

        // Process the event through the registry (same as runtime)
        match registry.process_event(v_event, &context) {
            Ok(events) => {
                tracing::debug!("Registry events generated: {:?}", events);

                if events.is_empty() {
                    tracing::error!("NO EVENTS GENERATED FROM REGISTRY - This is the problem!");

                    // Debug the command registry - check if visual command is registered
                    let commands = registry.get_commands();
                    tracing::debug!("Registry has {} commands", commands.len());

                    // Find visual mode command in registry
                    let mut found_visual_cmd = false;
                    for (i, cmd) in commands.iter().enumerate() {
                        if cmd.name() == "EnterVisualMode" {
                            found_visual_cmd = true;
                            tracing::info!("Found EnterVisualMode command at index {}", i);

                            // Test this specific command
                            let cmd_relevant = cmd.is_relevant(&context, &v_event);
                            tracing::debug!("  - is_relevant: {}", cmd_relevant);
                            break;
                        }
                    }

                    if !found_visual_cmd {
                        tracing::error!("EnterVisualMode command NOT FOUND in registry!");
                    }
                } else {
                    tracing::info!("Events generated successfully!");

                    // Apply the events to see if mode change works
                    for event in events {
                        tracing::debug!("Applying event: {:?}", event);

                        // Simulate what the controller does
                        match event {
                            crate::repl::commands::CommandEvent::ModeChangeRequested {
                                new_mode,
                            } => match view_model.change_mode(new_mode) {
                                Ok(_) => {
                                    tracing::info!(
                                        "Mode changed successfully to: {:?}",
                                        view_model.get_mode()
                                    );
                                }
                                Err(e) => {
                                    tracing::error!("Failed to change mode: {}", e);
                                }
                            },
                            _ => {
                                tracing::warn!("Unexpected event type");
                            }
                        }
                    }
                }
            }
            Err(e) => {
                tracing::error!("Error processing event: {}", e);
            }
        }

        tracing::debug!("Final mode: {:?}", view_model.get_mode());

        // Test should pass if we can get to visual mode
        // This test may fail initially to help us debug
        // assert_eq!(view_model.get_mode(), EditorMode::Visual);
    }
}
