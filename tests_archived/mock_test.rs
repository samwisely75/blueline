/// Simple isolated test to debug the mock renderer behavior
use blueline::ViewRenderer;

mod common;
use common::mock_view::{MockViewRenderer, RenderCall};

#[tokio::test]
async fn test_mock_renderer_isolated() {
    let mut mock = MockViewRenderer::new();
    let state = blueline::AppState::new((80, 24), false);

    // Test the exact sequence we expect in our first scenario
    mock.initialize_terminal(&state).unwrap();
    mock.render_full(&state).unwrap();

    // Check the call counts
    assert_eq!(mock.get_call_count(&RenderCall::InitializeTerminal), 1);
    assert_eq!(mock.get_call_count(&RenderCall::Full), 1);

    println!("✅ Isolated mock test passed!");
}

#[tokio::test]
async fn test_mock_renderer_with_clear() {
    let mut mock = MockViewRenderer::new();
    let state = blueline::AppState::new((80, 24), false);

    // Simulate some setup calls
    mock.initialize_terminal(&state).unwrap();
    mock.render_full(&state).unwrap();
    mock.render_content_update(&state).unwrap();

    // Clear calls
    mock.clear_calls();

    // Do the action we want to test
    mock.render_cursor_only(&state).unwrap();

    // Check that only the action we want is recorded
    assert_eq!(mock.get_call_count(&RenderCall::CursorOnly), 1);
    assert_eq!(mock.get_call_count(&RenderCall::Full), 0);
    assert_eq!(mock.get_call_count(&RenderCall::ContentUpdate), 0);

    println!("✅ Mock renderer clear test passed!");
}
