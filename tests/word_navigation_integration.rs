//! Integration test to verify word navigation works correctly after async segmentation fix
//!
//! This test reproduces the exact issue found in debug.log where word navigation
//! completely failed because no characters were marked as word boundaries.

use blueline::repl::models::buffer::buffer_model::BufferModel;
use blueline::repl::models::coordinates::geometry::Position;
use blueline::repl::models::coordinates::logical_position::LogicalPosition;
use blueline::repl::models::pane_state::{Pane, PaneCapabilities, PaneState};
use blueline::repl::services::async_word_segmenter::AsyncWordSegmenter;
use std::time::Duration;

#[tokio::test]
async fn test_word_navigation_with_json_content() {
    // This is the exact text pattern from debug.log that was failing
    let test_content = r#"  "timed_out" : false,
  "total" : 1118,
  "_shards" : {
    "total" : 5,
    "successful" : 5,
    "skipped" : 0,
    "failed" : 0
  }"#;

    // Create a pane state directly for testing
    let mut pane_state = PaneState::new(
        Pane::Request,
        76,   // width
        20,   // height
        true, // wrap_enabled
        PaneCapabilities::all(),
    );
    let async_segmenter = AsyncWordSegmenter::new();

    // Insert the test content that was causing the issue
    let mut buffer = BufferModel::new(Pane::Request);
    for line in test_content.lines() {
        for ch in line.chars() {
            buffer.insert_char(ch);
        }
        buffer.insert_char('\n');
    }

    // Update the pane with the content
    pane_state.buffer = buffer;

    // Rebuild display cache
    pane_state.build_display_cache(76, true, 4);

    // Trigger async segmentation for the viewport (this was missing in the bug)
    pane_state.request_viewport_segmentation(&async_segmenter, 10);

    // Wait for background segmentation to complete
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Process the segmentation results
    let segmentation_applied = pane_state.process_segmentation_results(&async_segmenter);
    println!("Segmentation applied: {segmentation_applied}");

    // CRITICAL: Rebuild display cache after segmentation results are applied
    if segmentation_applied {
        println!("Rebuilding display cache after segmentation...");
        pane_state.build_display_cache(76, true, 4);
    }

    // Debug: Check what segmentation state we're in
    let character_buffer = pane_state.buffer.content().character_buffer();
    if let Some(line) = character_buffer.get_line(0) {
        println!("Line 0 has {} characters", line.char_count());
        for i in 0..line.char_count().min(10) {
            if let Some(buffer_char) = line.get_char(i) {
                println!(
                    "Char {}: '{}' is_word_start={} is_word_end={}",
                    i, buffer_char.ch, buffer_char.is_word_start, buffer_char.is_word_end
                );
            }
        }
    }

    // Debug: Check the display cache
    println!(
        "Display cache has {} lines",
        pane_state.display_cache.display_line_count()
    );
    if let Some(display_line) = pane_state.display_cache.get_display_line(0) {
        println!("Display line 0 has {} chars", display_line.chars.len());
        for i in 0..display_line.chars.len().min(10) {
            let dc = &display_line.chars[i];
            println!(
                "DisplayChar {}: '{}' is_word_start={} is_word_end={}",
                i,
                dc.ch(),
                dc.buffer_char.is_word_start,
                dc.buffer_char.is_word_end
            );
        }
    }

    // Now test word navigation from the beginning of first line
    pane_state.set_current_cursor_position(LogicalPosition::new(0, 0));

    // Try to find next word - this should work now
    let current_display_pos = Position::new(0, 0); // Convert logical to display position
    let next_word = pane_state.find_next_word_start_position(current_display_pos);
    println!("Next word result: {next_word:?}");

    if next_word.is_none() {
        println!("❌ SEGMENTATION NOT WORKING - No word boundaries found");
        panic!("Word boundaries were not set by async segmentation");
    }

    // The first word should be 'timed_out' at some position > 0
    let next_pos = next_word.unwrap();
    assert_eq!(next_pos.row, 0, "Should stay on same line");
    assert!(
        next_pos.col > 0,
        "Should move forward to find word boundary"
    );

    // Verify that we can navigate through multiple words
    let second_word = pane_state.find_next_word_start_position(next_pos);
    assert!(second_word.is_some(), "Should find second word");

    println!("✅ Word navigation test passed!");
    println!("First word found at: {next_pos:?}");
    println!("Second word found at: {:?}", second_word.unwrap());
}

#[tokio::test]
async fn test_word_boundary_flags_are_set() {
    // Test that characters actually have word boundary flags set after segmentation
    let test_text = "hello world test";

    let mut pane_state = PaneState::new(
        Pane::Request,
        80,   // width
        10,   // height
        true, // wrap_enabled
        PaneCapabilities::all(),
    );
    let async_segmenter = AsyncWordSegmenter::new();

    // Insert simple text
    let mut buffer = BufferModel::new(Pane::Request);
    for ch in test_text.chars() {
        buffer.insert_char(ch);
    }

    // Update the pane with the content
    pane_state.buffer = buffer;
    pane_state.build_display_cache(80, true, 4);

    // Request segmentation
    pane_state.request_viewport_segmentation(&async_segmenter, 5);

    // Wait for segmentation
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Process results
    let applied = pane_state.process_segmentation_results(&async_segmenter);
    assert!(applied, "Segmentation should be applied");

    // Check that word boundary flags are actually set
    let character_buffer = pane_state.buffer.content().character_buffer();
    if let Some(line) = character_buffer.get_line(0) {
        let mut word_starts_found = 0;

        for i in 0..line.char_count() {
            if let Some(buffer_char) = line.get_char(i) {
                if buffer_char.is_word_start {
                    word_starts_found += 1;
                    println!("Word start found at position {}: '{}'", i, buffer_char.ch);
                }
            }
        }

        assert!(
            word_starts_found > 0,
            "Should find at least one word start marker"
        );
        assert!(
            word_starts_found >= 3,
            "Should find word starts for 'hello', 'world', 'test'"
        );
    } else {
        panic!("Should have at least one line of content");
    }

    println!("✅ Word boundary flags test passed!");
}

#[test]
fn test_async_segmenter_basic_functionality() {
    // Test the async segmenter works in isolation
    let segmenter = AsyncWordSegmenter::new();

    // Send a simple segmentation request
    let result = segmenter.request_segmentation(0, "hello world".to_string(), 1);
    assert!(
        result.is_ok(),
        "Should be able to send segmentation request"
    );

    // Give it time to process
    std::thread::sleep(Duration::from_millis(50));

    // Try to get results
    let results = segmenter.try_recv_results();
    assert!(!results.is_empty(), "Should receive segmentation results");

    let result = &results[0];
    assert_eq!(result.line_id, 0);
    assert_eq!(result.generation_id, 1);
    assert_eq!(result.flags.len(), 11); // "hello world" has 11 characters

    println!("✅ Async segmenter basic functionality test passed!");
}
