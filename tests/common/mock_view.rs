//! # Mock View Renderer for Testing
//!
//! This module provides a mock implementation of ViewRenderer that tracks
//! method calls for integration testing. It allows tests to verify that
//! the correct rendering methods are called with the expected parameters.

use anyhow::Result;
use std::sync::{Arc, Mutex};

// Import from the main crate using the crate name
use blueline::repl::{model::AppState, view_trait::ViewRenderer};

/// Type alias for render call storage to reduce complexity
type RenderCallStorage = Arc<Mutex<Vec<RenderCallRecord>>>;

/// Types of render calls that can be tracked
#[derive(Debug, Clone, PartialEq)]
pub enum RenderCall {
    CursorOnly,
    ContentUpdate,
    Full,
    InitializeTerminal,
    CleanupTerminal,
}

/// Snapshot of AppState for testing verification
#[derive(Debug, Clone)]
pub struct StateSnapshot {
    pub mode: String,
    pub current_pane: String,
    #[allow(dead_code)]
    pub request_buffer_lines: usize,
    #[allow(dead_code)]
    pub response_buffer_exists: bool,
    #[allow(dead_code)]
    pub cursor_line: usize,
    #[allow(dead_code)]
    pub cursor_col: usize,
}

impl StateSnapshot {
    fn from_app_state(state: &AppState) -> Self {
        Self {
            mode: format!("{:?}", state.mode),
            current_pane: format!("{:?}", state.current_pane),
            request_buffer_lines: state.request_buffer.line_count(),
            response_buffer_exists: state.response_buffer.is_some(),
            cursor_line: state.current_buffer().cursor_line(),
            cursor_col: state.current_buffer().cursor_col(),
        }
    }
}

/// A record of a render call with its parameters
#[derive(Debug, Clone)]
pub struct RenderCallRecord {
    pub call_type: RenderCall,
    pub state_snapshot: Option<StateSnapshot>,
}

/// Mock implementation of ViewRenderer that tracks calls for testing
#[derive(Debug)]
pub struct MockViewRenderer {
    /// Thread-safe storage for tracking render calls
    calls: RenderCallStorage,
}

impl MockViewRenderer {
    /// Create a new mock view renderer
    pub fn new() -> Self {
        Self {
            calls: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Get the number of times a specific render method was called
    pub fn get_call_count(&self, call_type: &RenderCall) -> usize {
        let calls = self.calls.lock().unwrap();
        calls
            .iter()
            .filter(|record| &record.call_type == call_type)
            .count()
    }

    /// Get the total number of render calls
    pub fn get_total_call_count(&self) -> usize {
        let calls = self.calls.lock().unwrap();
        calls.len()
    }

    /// Get all render calls in order
    pub fn get_all_calls(&self) -> Vec<RenderCallRecord> {
        let calls = self.calls.lock().unwrap();
        calls.clone()
    }

    /// Get the last call of a specific type
    pub fn get_last_call(&self, call_type: &RenderCall) -> Option<RenderCallRecord> {
        let calls = self.calls.lock().unwrap();
        calls
            .iter()
            .rev()
            .find(|record| &record.call_type == call_type)
            .cloned()
    }

    /// Clear all recorded calls
    pub fn clear_calls(&self) {
        let mut calls = self.calls.lock().unwrap();
        calls.clear();
    }

    /// Record a render call
    fn record_call(&self, call_type: RenderCall, state: Option<&AppState>) {
        let mut calls = self.calls.lock().unwrap();
        let state_snapshot = state.map(StateSnapshot::from_app_state);
        calls.push(RenderCallRecord {
            call_type,
            state_snapshot,
        });
    }

    /// Verify that render calls occurred in the expected order
    pub fn verify_call_sequence(&self, expected: &[RenderCall]) -> bool {
        let calls = self.calls.lock().unwrap();
        if calls.len() != expected.len() {
            return false;
        }

        calls
            .iter()
            .zip(expected.iter())
            .all(|(actual, expected)| &actual.call_type == expected)
    }
}

impl Default for MockViewRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl ViewRenderer for MockViewRenderer {
    fn render_cursor_only(&mut self, state: &AppState) -> Result<()> {
        self.record_call(RenderCall::CursorOnly, Some(state));
        Ok(())
    }

    fn render_content_update(&mut self, state: &AppState) -> Result<()> {
        self.record_call(RenderCall::ContentUpdate, Some(state));
        Ok(())
    }

    fn render_full(&mut self, state: &AppState) -> Result<()> {
        self.record_call(RenderCall::Full, Some(state));
        Ok(())
    }

    fn initialize_terminal(&self, state: &AppState) -> Result<()> {
        self.record_call(RenderCall::InitializeTerminal, Some(state));
        Ok(())
    }

    fn cleanup_terminal(&self) -> Result<()> {
        self.record_call(RenderCall::CleanupTerminal, None);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use blueline::repl::model::{AppState, EditorMode, Pane};

    #[test]
    fn mock_view_renderer_should_track_calls() {
        let mut mock = MockViewRenderer::new();
        let state = AppState::new((80, 24), false);

        // Test render calls
        mock.render_cursor_only(&state).unwrap();
        mock.render_content_update(&state).unwrap();
        mock.render_full(&state).unwrap();

        // Verify call counts
        assert_eq!(mock.get_call_count(&RenderCall::CursorOnly), 1);
        assert_eq!(mock.get_call_count(&RenderCall::ContentUpdate), 1);
        assert_eq!(mock.get_call_count(&RenderCall::Full), 1);
        assert_eq!(mock.get_total_call_count(), 3);
    }

    #[test]
    fn mock_view_renderer_should_track_terminal_calls() {
        let mock = MockViewRenderer::new();
        let state = AppState::new((80, 24), false);

        mock.initialize_terminal(&state).unwrap();
        mock.cleanup_terminal().unwrap();

        assert_eq!(mock.get_call_count(&RenderCall::InitializeTerminal), 1);
        assert_eq!(mock.get_call_count(&RenderCall::CleanupTerminal), 1);
    }

    #[test]
    fn mock_view_renderer_should_verify_call_sequence() {
        let mut mock = MockViewRenderer::new();
        let state = AppState::new((80, 24), false);

        mock.render_full(&state).unwrap();
        mock.render_cursor_only(&state).unwrap();
        mock.render_content_update(&state).unwrap();

        let expected = vec![
            RenderCall::Full,
            RenderCall::CursorOnly,
            RenderCall::ContentUpdate,
        ];

        assert!(mock.verify_call_sequence(&expected));

        let wrong_sequence = vec![RenderCall::CursorOnly, RenderCall::Full];

        assert!(!mock.verify_call_sequence(&wrong_sequence));
    }

    #[test]
    fn mock_view_renderer_should_capture_state_snapshots() {
        let mut mock = MockViewRenderer::new();
        let mut state = AppState::new((80, 24), false);

        // Change some state
        state.mode = EditorMode::Insert;
        state.current_pane = Pane::Response;

        mock.render_full(&state).unwrap();

        let last_call = mock.get_last_call(&RenderCall::Full).unwrap();
        let snapshot = last_call.state_snapshot.unwrap();

        assert_eq!(snapshot.mode, "Insert");
        assert_eq!(snapshot.current_pane, "Response");
    }
}
