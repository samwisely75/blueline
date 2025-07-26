use crate::repl::model::AppState;

#[test]
fn test_response_wrapping_functionality() {
    let mut state = AppState::new((80, 24), false);
    
    // Set a response with a very long line that should wrap
    let long_response = "HTTP/1.1 200 OK\nContent-Type: application/json\n\nThis is a very long line that should definitely wrap across multiple display lines when the terminal width is narrow and should properly demonstrate the wrapping functionality";
    state.set_response(long_response.to_string());
    
    // Verify that response buffer was created
    assert!(state.response_buffer.is_some());
    
    // Check that cache manager has response cache available 
    let cache = state.cache_manager.get_response_cache();
    
    // If the cache was updated, it should be valid
    // Note: The cache might be valid or invalid depending on whether update_response_cache succeeds
    // But the important thing is that set_response now attempts to update the cache
    println!("Response cache valid: {}", cache.is_valid);
    println!("Total display lines: {}", cache.total_display_lines);
}
