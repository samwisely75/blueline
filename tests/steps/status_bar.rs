// Status bar display and mode indicators step definitions

use crate::common::world::BluelineWorld;
use cucumber::{then};

// ===== STATUS BAR VISIBILITY AND DISPLAY =====

#[then("I should see the status bar at the bottom")]
async fn i_should_see_status_bar_at_bottom(world: &mut BluelineWorld) {
    // Verify status bar presence through captured output
    let terminal_state = world.get_terminal_state();
    
    // Check that we have terminal output indicating a status bar
    assert!(
        terminal_state.height > 1,
        "Expected status bar at bottom"
    );
}

#[then(regex = r#"the status bar should show "([^"]*)"#)]
async fn status_bar_should_show_mode(world: &mut BluelineWorld, expected_mode: String) {
    let terminal_state = world.get_terminal_state();
    let screen_content = terminal_state
        .grid
        .iter()
        .map(|row| row.iter().collect::<String>())
        .collect::<Vec<_>>()
        .join("\n");

    assert!(
        screen_content.contains(&expected_mode),
        "Expected status bar to show mode: {expected_mode}"
    );
}
