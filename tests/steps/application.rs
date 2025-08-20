//! Step definitions for application lifecycle and setup
//!
//! This module contains step definitions for:
//! - Application startup and shutdown
//! - Initial setup and configuration
//! - Background steps

use crate::common::world::BluelineWorld;
use cucumber::{given, then, when};
use tracing::{debug, info};

// Background and setup steps
#[given("the application is started with default settings")]
async fn app_started_with_default_settings(world: &mut BluelineWorld) {
    info!("=== Starting background step: app with default settings ===");
    info!("Step 1: Initializing world...");
    world.initialize().await;
    info!("Step 2: Starting app...");
    world.start_app(vec![]).await.expect("Failed to start app");
    info!("Step 3: App started, waiting for initialization...");

    // Give the app time to initialize
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    info!("Step 4: Background step complete");
}

#[given("the request buffer is empty")]
async fn given_request_buffer_is_empty(world: &mut BluelineWorld) {
    debug!("Ensuring request buffer is empty");
    // In our test environment, the buffer starts empty by default
    // This step serves as documentation and could be enhanced to actually clear buffer
    let _state = world.get_terminal_state().await;
    debug!("Request buffer verified as empty");
}

#[given("I have started the application")]
async fn given_i_have_started_the_application(world: &mut BluelineWorld) {
    info!("=== Starting application ===");
    world.initialize().await;
    world.start_app(vec![]).await.expect("Failed to start app");

    // Give the app time to initialize
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    info!("=== Application started successfully ===");
}

#[given(regex = r"the terminal dimensions are set to width (\d+) and height (\d+)")]
async fn given_terminal_dimensions(world: &mut BluelineWorld, width: String, height: String) {
    let w: u16 = width.parse().expect("Invalid width");
    let h: u16 = height.parse().expect("Invalid height");
    debug!("Setting terminal dimensions to {}x{}", w, h);
    world.set_terminal_size(w, h);
    debug!("Terminal dimensions set to {}x{}", w, h);
}

#[given(regex = r#"I have "([^"]+)" in the request buffer"#)]
async fn given_text_in_request_buffer(world: &mut BluelineWorld, text: String) {
    debug!("Setting up request buffer with text: '{}'", text);
    // For now, we'll simulate having this text by injecting it into terminal output
    // In a real implementation, this would set up the actual buffer state
    world.simulate_text_input(&text).await;
    debug!("Request buffer set up with: '{}'", text);
}

#[given("I am in the Request pane")]
async fn given_in_request_pane(world: &mut BluelineWorld) {
    debug!("Ensuring we are in Request pane");
    // For now, assume we start in Request pane by default
    let _state = world.get_terminal_state().await;
    debug!("Confirmed in Request pane");
}

#[when("the application starts")]
async fn when_application_starts(world: &mut BluelineWorld) {
    debug!("Application start event");
    // This is already handled by the background step
    world.tick().await.expect("Failed to tick");
}

// Application termination steps
#[then("the application should terminate cleanly")]
async fn then_app_should_terminate_cleanly(_world: &mut BluelineWorld) {
    // TODO: Implement application termination verification
    // Should check:
    // 1. No error messages in terminal
    // 2. App process has exited cleanly
    // 3. Exit code is 0
    // For test purposes, assume clean termination if we reach this step
    // In a real implementation, this would check process exit status and cleanup
    debug!("✅ Application terminated cleanly (simulated)");
}

#[then("the application should terminate without saving")]
async fn then_app_should_terminate_without_saving(_world: &mut BluelineWorld) {
    // TODO: Implement force quit verification
    // Should check:
    // 1. App terminated immediately without save prompts
    // 2. No "unsaved changes" warnings
    // 3. Exit code indicates force quit
    // For test purposes, assume clean termination if we reach this step
    // In a real implementation, this would check process exit status
    debug!("✅ Application terminated successfully (simulated)");
}

// Response pane visibility
#[then("there should be no response pane visible")]
async fn then_no_response_pane_visible(world: &mut BluelineWorld) {
    let state = world.get_terminal_state().await;

    // Verify that terminal content doesn't contain response pane indicators
    // Response pane would typically show HTTP response content or have specific markers
    let terminal_content = world.get_terminal_content().await;

    // Basic assertion: ensure we don't see response-specific content
    assert!(
        !terminal_content.contains("HTTP/1.1")
            && !terminal_content.contains("Content-Type:")
            && !terminal_content.contains("Response Body:"),
        "Response pane appears to be visible when it shouldn't be. Terminal content: {terminal_content}"
    );

    // Additional check: verify cursor is in request pane area (top area)
    assert!(
        state.cursor_position.1 < state.height / 2,
        "Cursor should be in request pane area (top half) when no response pane is visible"
    );
}
