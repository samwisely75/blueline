use cucumber::World;

pub mod common;

pub use common::world::BluelineWorld;

/// Integration tests using Cucumber BDD framework
/// Run with: cargo test --test integration_tests
#[tokio::main]
async fn main() {
    // Serialize feature execution to prevent resource conflicts
    run_features_sequentially().await;
}

/// Run each feature file sequentially to avoid resource conflicts
async fn run_features_sequentially() {
    let features = [
        "features/application.feature",
        "features/mode_transitions.feature",
        "features/movement.feature",
        "features/editing.feature",
        "features/command_line.feature",
        "features/integration.feature",
    ];

    // Run the main features first
    println!(
        "Running {} main feature files sequentially...",
        features.len()
    );

    for (i, feature) in features.iter().enumerate() {
        println!("\n[{}/{}] Running {}...", i + 1, features.len(), feature);
        BluelineWorld::run(feature).await;
        println!("✓ {} completed successfully", feature);
    }

    // Run screen refresh scenarios individually to avoid thread_local interference
    // NOTE: These scenarios require special handling due to MockViewRenderer's thread_local storage
    // See run_screen_refresh_scenarios() documentation for detailed explanation
    println!(
        "\n[{}/{}] Running screen refresh scenarios individually...",
        features.len() + 1,
        features.len() + 1
    );
    run_screen_refresh_scenarios().await;
    println!("✓ Screen refresh scenarios completed successfully");

    println!("\n🎉 All feature files completed successfully!");
}

/// Run screen refresh scenarios individually to avoid thread_local state interference
///
/// ## Why Individual Scenario Files?
///
/// Originally, all screen refresh scenarios were in a single `screen_refresh.feature` file.
/// However, this caused test failures due to thread_local storage interference:
///
/// **Problem**: The MockViewRenderer uses `thread_local` storage for test isolation.
/// When multiple BDD scenarios run sequentially in the same test execution, they share
/// the same thread_local storage, causing state to persist between scenarios.
///
/// **Symptoms**: Tests would fail with incorrect call counts:
/// - Expected 1 render_full call, got 4 (accumulated from previous scenarios)
/// - Expected 1 render_cursor_only call, got 0 (wrong scenario's calls)
///
/// **Root Cause**: Cucumber runs all scenarios in a feature file within the same thread,
/// so `thread_local!` storage persists across scenario boundaries even when we explicitly
/// reset it in step definitions.
///
/// **Solution**: Split each scenario into its own feature file. This ensures each scenario
/// runs in complete isolation with its own fresh MockViewRenderer instance, preventing
/// any state interference between test cases.
///
/// **Files Created**:
/// - `screen_refresh_startup.feature` - Tests initialize_terminal + render_full on startup
/// - `screen_refresh_keyevents.feature` - Tests render_cursor_only for key events  
/// - `screen_refresh_textchanges.feature` - Tests render_content_update for text changes
/// - `screen_refresh_modechanges.feature` - Tests render_full for mode transitions
/// - `screen_refresh_shutdown.feature` - Tests cleanup_terminal on shutdown
///
/// This architectural decision ensures reliable, deterministic test execution while
/// maintaining comprehensive coverage of all screen refresh tracking functionality.
async fn run_screen_refresh_scenarios() {
    // Run individual feature files for each scenario to ensure complete isolation
    let scenarios = [
        "features/screen_refresh_startup.feature",
        "features/screen_refresh_keyevents.feature",
        "features/screen_refresh_textchanges.feature",
        "features/screen_refresh_modechanges.feature",
        "features/screen_refresh_shutdown.feature",
    ];

    for scenario_file in scenarios {
        BluelineWorld::run(scenario_file).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn run_integration_tests() {
        // Run features sequentially in tests as well
        run_features_sequentially().await;
    }
}
