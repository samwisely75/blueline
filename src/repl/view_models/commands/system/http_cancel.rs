//! HTTP request cancellation command
//!
//! This command handles Ctrl+C when an HTTP request is executing,
//! cancelling the request instead of quitting the application.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::register_command;
use crate::repl::models::pane_state::EditorMode;
use crate::repl::view_models::commands::{Command, CommandContext, ExecutionContext};
use crate::repl::view_models::post_command_actions::PostCommandAction;

/// Command to cancel HTTP request with Ctrl+C
pub struct HttpCancelCommand;

impl HttpCancelCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Command for HttpCancelCommand {
    fn name(&self) -> &'static str {
        "HttpCancelCommand"
    }

    fn is_relevant(
        &self,
        key_event: KeyEvent,
        _mode: EditorMode,
        _: &CommandContext,
    ) -> bool {
        // Only relevant for Ctrl+C
        if key_event.code != KeyCode::Char('c')
            || !key_event.modifiers.contains(KeyModifiers::CONTROL)
        {
            return false;
        }

        // Only relevant when an HTTP request is executing
        // We need to check this via the context somehow, but CommandContext doesn't have app_state
        // For now, return true and let the execution context handle it
        true
    }

    fn execute(
        &self,
        _key_event: KeyEvent,
        context: &mut ExecutionContext,
    ) -> Result<Vec<PostCommandAction>> {
        tracing::info!("Checking for HTTP request cancellation via Ctrl+C");

        // Only cancel if there's an executing request
        if !context.app_state.is_executing_request() {
            // If no HTTP request is executing, let AppTerminateCommand handle this
            return Ok(vec![]);
        }

        tracing::info!("Cancelling HTTP request via Ctrl+C");

        // Cancel the request through the HTTP service
        if let Some(http_service) = context.services.http.as_mut() {
            http_service.cancel_request();
        } else {
            tracing::warn!("HTTP service not available for cancellation");
            // Still clear the execution state as fallback
            context.app_state.set_executing_request(false);
            context
                .app_state
                .set_status_message("Request cancelled".to_string());
        }

        // Generate post-command actions for status update
        Ok(vec![PostCommandAction::FullRedrawRequired])
    }
}

impl Default for HttpCancelCommand {
    fn default() -> Self {
        Self::new()
    }
}

// Auto-register this command using the inventory system
register_command!(HttpCancelCommand, "HttpCancelCommand");
