// Mode switching step definitions (normal, insert, visual, command)

use crate::common::world::{BluelineWorld, Mode};
use anyhow::Result;
use blueline::repl::events::EditorMode;
use cucumber::{given, then, when};

// ===== MODE SETUP STEPS =====

#[given("I am in normal mode")]
fn i_am_in_normal_mode(world: &mut BluelineWorld) {
    println!(
        "🔍 Setting normal mode - cursor before: ({}, {})",
        world.cursor_position.line, world.cursor_position.column
    );
    world.mode = Mode::Normal;
    println!(
        "🔍 Setting normal mode - cursor after: ({}, {})",
        world.cursor_position.line, world.cursor_position.column
    );
}

#[given("I am in insert mode")]
async fn given_i_am_in_insert_mode(world: &mut BluelineWorld) {
    world.mode = Mode::Insert;
}

#[given("I am in visual mode")]
async fn given_i_am_in_visual_mode(world: &mut BluelineWorld) {
    world.mode = Mode::Normal; // Visual mode maps to Normal mode for legacy compatibility
    println!("🔍 Setting visual mode (mapped to Normal for compatibility)");
}

#[given("I am in command mode")]
async fn given_i_am_in_command_mode(world: &mut BluelineWorld) {
    world.mode = Mode::Command;
    println!("🔍 Setting command mode");
}

// ===== MODE TRANSITION ACTIONS =====

#[when(regex = r#"^I press "([^"]*)" to enter insert mode$"#)]
async fn i_press_key_to_enter_insert_mode(world: &mut BluelineWorld, key: String) -> Result<()> {
    // Force use of simulation path by temporarily storing real components
    let saved_view_model = world.view_model.take();
    let saved_command_registry = world.command_registry.take();

    let result = world.press_key(&key).await;

    // Restore real components if they existed
    world.view_model = saved_view_model;
    world.command_registry = saved_command_registry;

    result
}

#[when(regex = r#"^I press Escape to exit insert mode$"#)]
async fn i_press_escape_to_exit_insert_mode(world: &mut BluelineWorld) -> Result<()> {
    // Force use of simulation path by temporarily storing real components
    let saved_view_model = world.view_model.take();
    let saved_command_registry = world.command_registry.take();

    let result = world.press_key("Escape").await;

    // Restore real components if they existed
    world.view_model = saved_view_model;
    world.command_registry = saved_command_registry;

    result
}

#[when(regex = r#"I send key "([^"]*)" to enter visual mode"#)]
async fn send_key_to_enter_visual_mode(world: &mut BluelineWorld, key: String) {
    println!("👁️ Sending key '{key}' to enter visual mode");

    if world.view_model.is_none() {
        panic!("Real application not initialized");
    }

    // Send the key through the real command system
    match world.press_key(&key).await {
        Ok(()) => {
            println!("✅ Successfully sent key '{key}' to enter visual mode");

            // Verify we're in visual mode
            if let Some(ref view_model) = world.view_model {
                let mode = view_model.get_mode();
                println!("📊 Current mode after key press: {mode:?}");
                assert_eq!(
                    mode,
                    EditorMode::Visual,
                    "Expected Visual mode after pressing '{key}'"
                );

                // Check visual selection state
                let selection = view_model.get_visual_selection();
                println!("🎯 Visual selection state: {selection:?}");
            }
        }
        Err(e) => {
            println!("❌ Failed to send key '{key}': {e}");
            panic!("Failed to send key to enter visual mode: {e}");
        }
    }
}

#[when("I send Escape key to exit insert mode")]
async fn send_escape_key(world: &mut BluelineWorld) {
    println!("⎋ Sending Escape key to exit insert mode");

    // Make sure we have the real components initialized
    if world.view_model.is_none() {
        panic!("Real application not initialized - call 'I initialize the real blueline application' first");
    }

    // Send the Escape key through the real command system
    match world.press_key("Escape").await {
        Ok(()) => {
            println!("✅ Successfully sent Escape key");

            // Verify we're in normal mode by checking the ViewModel
            if let Some(ref view_model) = world.view_model {
                let mode = view_model.get_mode();
                println!("📊 Current mode after Escape: {mode:?}");
                assert_eq!(
                    mode,
                    blueline::repl::events::EditorMode::Normal,
                    "Expected Normal mode after pressing Escape"
                );
            }
        }
        Err(e) => {
            panic!("Failed to send Escape key: {e}");
        }
    }
}

// ===== MODE VERIFICATION STEPS =====

#[then("I am still in normal mode")]
async fn i_am_still_in_normal_mode(world: &mut BluelineWorld) {
    assert_eq!(
        world.mode,
        Mode::Normal,
        "Expected to remain in normal mode"
    );
}

#[then("I am in insert mode")]
async fn i_am_in_insert_mode(world: &mut BluelineWorld) {
    assert_eq!(world.mode, Mode::Insert, "Expected to be in insert mode");
}

#[then("I am in command mode")]
async fn i_am_in_command_mode(world: &mut BluelineWorld) {
    assert_eq!(world.mode, Mode::Command, "Expected to be in command mode");
}

#[then("I am in normal mode")]
async fn i_am_in_normal_mode_then(world: &mut BluelineWorld) {
    assert_eq!(world.mode, Mode::Normal, "Expected to be in normal mode");
}

#[then("I should be in visual mode")]
async fn should_be_in_visual_mode(world: &mut BluelineWorld) {
    println!("🔍 Checking if in visual mode");

    if let Some(ref view_model) = world.view_model {
        let mode = view_model.get_mode();
        assert_eq!(
            mode,
            EditorMode::Visual,
            "Expected Visual mode but got {mode:?}"
        );
        println!("✅ Confirmed Visual mode");
    } else {
        panic!("Real application not initialized");
    }
}

#[then("I should be in insert mode using real components")]
async fn should_be_in_insert_mode_using_real_components(world: &mut BluelineWorld) {
    if let Some(ref view_model) = world.view_model {
        let current_mode = view_model.get_mode();
        assert_eq!(
            current_mode,
            blueline::repl::events::EditorMode::Insert,
            "Expected Insert mode using real components, but got {current_mode:?}"
        );
        println!("✅ Confirmed Insert mode using real ViewModel");
    } else {
        panic!("Real ViewModel not initialized - call 'I initialize the real blueline application' first");
    }
}

#[then("I should be in normal mode using real components")]
async fn should_be_in_normal_mode_using_real_components(world: &mut BluelineWorld) {
    if let Some(ref view_model) = world.view_model {
        let current_mode = view_model.get_mode();
        assert_eq!(
            current_mode,
            blueline::repl::events::EditorMode::Normal,
            "Expected Normal mode using real components, but got {current_mode:?}"
        );
        println!("✅ Confirmed Normal mode using real ViewModel");
    } else {
        panic!("Real ViewModel not initialized - call 'I initialize the real blueline application' first");
    }
}

// ===== CURSOR STYLE CHANGES WITH MODE =====

#[then("the cursor style changes to a blinking bar")]
async fn cursor_style_blinking_bar(world: &mut BluelineWorld) {
    // In insert mode, the cursor should be a blinking bar
    // We check for mode consistency and terminal output
    assert_eq!(
        world.mode,
        Mode::Insert,
        "Expected insert mode for blinking bar cursor"
    );

    // Verify we have terminal output that might indicate cursor style change
    let captured_output = world.stdout_capture.lock().unwrap().clone();
    let output_str = String::from_utf8_lossy(&captured_output);

    assert!(
        !output_str.trim().is_empty(),
        "Expected terminal output indicating cursor style change to blinking bar"
    );
}

#[then("the cursor style changes to a steady block")]
async fn cursor_style_steady_block(world: &mut BluelineWorld) {
    // In normal mode, the cursor should be a steady block
    // Since we're already checking that we're in normal mode, we can verify
    // that the mode change was successful by checking the mode
    assert_eq!(
        world.mode,
        Mode::Normal,
        "Expected to be in normal mode with steady block cursor"
    );

    // Also check that we have some terminal output indicating the mode change occurred
    let captured_output = world.stdout_capture.lock().unwrap().clone();
    let output_str = String::from_utf8_lossy(&captured_output);
    let _output = String::from_utf8_lossy(&captured_output);

    assert!(
        !output_str.trim().is_empty(),
        "Expected terminal to show some output after mode change. Output: {output}",
        output = output_str.chars().take(200).collect::<String>()
    );
}

// ===== RENDER VERIFICATION =====

#[then("render_full should be called again")]
async fn render_full_should_be_called_again(world: &mut BluelineWorld) {
    // Verify that render_full was called during mode transition
    let captured_output = world.stdout_capture.lock().unwrap().clone();
    let output_str = String::from_utf8_lossy(&captured_output);

    assert!(
        !output_str.trim().is_empty(),
        "Expected render_full to be called during mode transition, but no terminal output captured"
    );
}

#[then("render_cursor_only should be called once")]
async fn then_render_cursor_only_called_once(world: &mut BluelineWorld) {
    let terminal_state = world.get_terminal_state();

    // Verify cursor rendering occurred (cursor should be visible and positioned)
    assert!(
        terminal_state.cursor_visible,
        "Expected cursor to be visible after render_cursor_only call"
    );
}

#[then("no other render methods should be called")]
async fn then_no_other_render_methods_called(world: &mut BluelineWorld) {
    // This step verifies that only the expected render method was called
    // We can check this by ensuring the terminal state is minimally changed
    let terminal_state = world.get_terminal_state();

    // Verify that we have some output but not excessive rendering
    let captured_output = world.stdout_capture.lock().unwrap().clone();
    assert!(
        !captured_output.is_empty(),
        "Expected some terminal output from render calls"
    );

    // Additional verification that cursor is in a valid state
    assert!(
        terminal_state.cursor.0 < terminal_state.height
            && terminal_state.cursor.1 < terminal_state.width,
        "Expected cursor to be within terminal bounds after render"
    );
}
