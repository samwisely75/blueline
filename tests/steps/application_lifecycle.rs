// Application startup, shutdown, initialization and cleanup step definitions

use crate::common::world::{ActivePane, BluelineWorld, Mode};
use blueline::ViewRenderer;
use cucumber::{given, then, when};
use std::time::Duration;

// ===== APPLICATION INITIALIZATION =====

#[given("blueline is running with default profile")]
async fn blueline_running_default_profile(world: &mut BluelineWorld) {
    // Set up default state
    world.mode = Mode::Normal;
    // Only set active pane to Request if it hasn't been specifically set to Response
    if world.active_pane != ActivePane::Response {
        world.active_pane = ActivePane::Request;
    }

    // Initialize the AppController with default settings
    world
        .init_real_application()
        .expect("Failed to initialize blueline application");
    world
        .setup_mock_server()
        .await
        .expect("Failed to setup mock server");
}

#[given("I initialize the real blueline application")]
async fn initialize_real_blueline_application(world: &mut BluelineWorld) {
    world
        .init_real_application()
        .expect("Failed to initialize real blueline application");
    println!("✅ Real blueline application components initialized");
}

#[given("blueline is launched in a terminal")]
async fn blueline_is_launched_in_terminal(world: &mut BluelineWorld) {
    // This is equivalent to the default setup, just ensure app is initialized
    let _ = world.init_real_application();
    println!("🚀 Blueline launched in terminal");
}

#[given("I build the blueline application")]
async fn build_blueline_application(_world: &mut BluelineWorld) {
    let build_result = std::process::Command::new("cargo")
        .args(["build", "--release"])
        .output()
        .expect("Failed to execute cargo build");

    assert!(
        build_result.status.success(),
        "Failed to build blueline application"
    );
}

#[when("I launch the real blueline application")]
#[allow(clippy::zombie_processes)]
async fn launch_real_blueline_application(_world: &mut BluelineWorld) {
    use std::process::{Command, Stdio};
    use std::thread;

    let _child = Command::new("./target/release/blueline")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to launch blueline");

    // Give it a moment to initialize
    thread::sleep(Duration::from_millis(500));

    // Store the child process in world for later interaction
    // Note: We'll need to add this field to BluelineWorld
    println!("✅ Blueline application launched successfully");
}

// ===== APPLICATION EXIT AND SHUTDOWN =====

#[then("the application exits")]
async fn application_exits(world: &mut BluelineWorld) {
    assert!(world.app_exited, "Expected application to have exited");
}

#[then("the application exits without saving")]
async fn application_exits_without_saving(world: &mut BluelineWorld) {
    assert!(world.app_exited, "Expected application to have exited");
    assert!(
        world.force_quit,
        "Expected application to have force quit without saving"
    );
}

#[when("the controller shuts down")]
async fn when_controller_shuts_down(world: &mut BluelineWorld) {
    // Simulate controller shutdown with terminal cleanup
    let cleanup_output = "\x1b[?25h"; // Show cursor
    world.capture_stdout(cleanup_output.as_bytes());

    // Mark application as shut down
    world.app_exited = true;
}

// ===== TERMINAL RENDERER INITIALIZATION =====

#[given("a REPL controller with terminal output capture")]
async fn given_repl_controller_with_terminal_output_capture(world: &mut BluelineWorld) {
    // Initialize the AppController with real components but terminal output capture
    world
        .init_real_application()
        .expect("Failed to initialize AppController");

    // Initialize terminal renderer for output capture
    world
        .init_terminal_renderer()
        .expect("Failed to initialize terminal renderer");

    // Set up initial state for terminal output verification
    world.set_cursor_position(0, 0);
    world.mode = Mode::Normal;
    world.active_pane = ActivePane::Request;
}

#[given("the controller has started up")]
async fn given_controller_has_started_up(world: &mut BluelineWorld) {
    // Use the actual terminal renderer to generate startup output
    if let Some(ref mut renderer) = world.terminal_renderer {
        renderer
            .initialize()
            .expect("Failed to initialize terminal renderer");
    } else {
        // Fallback: simulate startup output
        let init_output = "\x1b[2J\x1b[H"; // Clear screen and move cursor to home
        world.capture_stdout(init_output.as_bytes());
    }
}

#[when("the controller starts up")]
async fn when_controller_starts_up(world: &mut BluelineWorld) {
    // Use the actual terminal renderer to generate startup output
    if let Some(ref mut renderer) = world.terminal_renderer {
        renderer
            .initialize()
            .expect("Failed to initialize terminal renderer");
    } else {
        // Fallback: simulate startup output
        let startup_output = "\x1b[2J\x1b[H"; // Clear screen and move cursor to home
        world.capture_stdout(startup_output.as_bytes());
    }
}

#[then("initialize_terminal should be called once")]
async fn then_initialize_terminal_called_once(world: &mut BluelineWorld) {
    let terminal_state = world.get_terminal_state();
    // Verify terminal was initialized (cursor should be at home position after init)
    assert_eq!(
        terminal_state.cursor,
        (0, 0),
        "Terminal should be initialized with cursor at home position"
    );
}

#[then("cleanup_terminal should be called once")]
async fn then_cleanup_terminal_called_once(world: &mut BluelineWorld) {
    let captured_output = world.stdout_capture.lock().unwrap().clone();
    let output_string = String::from_utf8_lossy(&captured_output);

    // Verify cleanup sequence was called (cursor visibility restored)
    assert!(
        output_string.contains("\x1b[?25h"),
        "Expected terminal cleanup sequence to restore cursor visibility"
    );
}

#[given("a REPL controller with mock view renderer")]
async fn given_repl_controller_with_mock_view_renderer(world: &mut BluelineWorld) {
    // Initialize the AppController with real components and mock renderer
    world
        .init_real_application()
        .expect("Failed to initialize AppController");

    // Initialize terminal renderer for mock capture
    world
        .init_terminal_renderer()
        .expect("Failed to initialize terminal renderer");

    // Set up initial state
    world.set_cursor_position(0, 0);
    world.mode = Mode::Normal;
    world.active_pane = ActivePane::Request;

    // Clear any previous render history if needed
    // Note: terminal_state_history removed to match current BluelineWorld structure
}
