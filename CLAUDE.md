# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Blueline is a lightweight HTTP client with a REPL interface featuring vim-style navigation. It's built with Rust and uses a clean MVC architecture with a command pattern for user interactions.

## Essential Commands

### Build and Development
```bash
# Build
cargo build
cargo build --release

# Run with profiles
cargo run -- -p staging     # Use staging profile from ~/.blueline/profile
cargo run -- -v            # Verbose mode showing connection details

# Linting and formatting (REQUIRED before commits)
cargo fmt                   # Format code
cargo clippy --all-targets --all-features -- -D warnings  # Lint check
```

### Testing
```bash
# Run all tests
cargo test

# Run BDD tests only
cargo test --test integration_tests

# Run specific feature test
cargo test --test integration_tests -- features/movement.feature

# Run with output
cargo test -- --nocapture
```

### Git Hooks
The project uses pre-commit hooks that enforce code quality:
- Automatically runs `cargo fmt` check
- Runs `cargo clippy` with strict warnings
- Rejects commits with any warnings

Install hooks: `./scripts/install-hooks.sh`

## Architecture

### MVC Pattern
- **Model** (`src/repl/model.rs`): Application state, buffers, modes
- **View** (`src/repl/view.rs`): Terminal rendering with split panes
- **Controller** (`src/repl/controller.rs`): Event loop and command orchestration

### Command Pattern
All user interactions are implemented as stateless commands in `src/repl/commands/`:
- Each command implements the `Command` trait
- Commands operate on `AppState` through the `process()` method
- Categories: movement, editing, mode transitions, command line

### Display Cache
The `DisplayCache` (`src/repl/display_cache.rs`) is a performance optimization that:
- Pre-calculates wrapped text layouts
- Enables O(1) cursor positioning
- Uses background threads for Request pane updates
- Maintains both logical and display positions

### Testing Infrastructure
- BDD tests using Cucumber framework in `features/` directory
- Mock view renderer using thread-local storage for testing without terminal I/O
- Tests run sequentially to avoid resource conflicts

## Key Development Patterns

### Dual Position Tracking
The codebase maintains two cursor position systems:
- **Logical positions**: Line/column in source text
- **Display positions**: Account for wrapped lines in terminal

### Profile Configuration
Profiles are stored in `~/.blueline/profile` as INI files:
- Headers prefixed with `@` (e.g., `@authorization`)
- Connection settings: `host`, `insecure`, `ca_cert`, `proxy`
- Authentication: `user`, `password`

### Error Handling
- Uses `anyhow::Result` for error propagation
- Commands return `Result<()>` and errors are displayed in status bar
- Network errors show detailed connection information in verbose mode

## Current Development

Currently on `feature/display-cache` branch implementing performance optimizations for text rendering and cursor movement with wrapped lines.