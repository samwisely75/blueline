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
async fn run_screen_refresh_scenarios() {
    // Create individual feature files for each scenario to ensure complete isolation
    let scenarios = ["features/screen_refresh_single.feature"];

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
