use std::process::Command;

fn blueline_binary() -> String {
    env!("CARGO_BIN_EXE_blueline").to_string()
}

#[test]
fn test_help_command() {
    let output = Command::new(blueline_binary())
        .arg("--help")
        .output()
        .expect("Failed to execute blueline");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("A lightweight, profile-based HTTP client"));
    assert!(stdout.contains("Usage:"));
    assert!(stdout.contains("--profile"));
    assert!(stdout.contains("--verbose"));
    // Should NOT contain old single-execution mode arguments
    assert!(!stdout.contains("METHOD"));
    assert!(!stdout.contains("URL"));
}

#[test]
fn test_version_command() {
    let output = Command::new(blueline_binary())
        .arg("--version")
        .output()
        .expect("Failed to execute blueline");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("blueline"));
}

#[test]
fn test_only_profile_and_verbose_options() {
    // Test that we can run with only --profile option
    let output = Command::new(blueline_binary())
        .args(["--profile", "test"])
        .arg("--help") // Use help to exit immediately
        .output()
        .expect("Failed to execute blueline");

    assert!(output.status.success());
}

#[test]
fn test_repl_mode_required() {
    // Test that running without any arguments tries to start REPL mode
    // This should fail due to missing endpoint configuration, but the failure
    // should be about configuration, not about missing arguments
    let output = Command::new(blueline_binary())
        .stdin(std::process::Stdio::null()) // No input
        .output()
        .expect("Failed to execute blueline");

    // Should exit with error due to missing endpoint configuration
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Endpoint cannot be empty") || stderr.contains("Error"));
}
