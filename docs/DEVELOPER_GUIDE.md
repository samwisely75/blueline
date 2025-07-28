# Developer's Guide for Blueline

This file provides guidance to human developers and Generative AI Engines when working with code in this repository.

## Application Overview

This application is a Rust-based HTTP client named `blueline`, previously known as `httpc` and `webly`. It is designed to be lightweight and profile-based, allowing users to manage HTTP requests easily. The project has been recently renamed and restructured, with a focus on REPL.

This application is built using the `crossterm` library for terminal I/O and `bluenote`, my another project enablese to make HTTP requests with profiles like `awscli`. The goal of this project is to mimic vi as closely as possible while providing an easy way to interact with HTTP endpoints in a terminal environment.

## Project

### History

I first started this project as a simple one-off HTTP command line tool like `curl`, which is left as [`httpc`](https://github.com/samwisely75/httpc), which incorporating the idea of profile in `awscli`. The app is nearly built, and I started adding a REPL interface to it to integrate Vim + Postman into one terminal app. The branch soon became too large to maintain as a single project so I decided to split the REPL capabilities as an independent project. That's the birth of `blueline`. The REPL interface is inspired by the vi editor, allowing users to navigate and manipulate HTTP requests and responses in a terminal environment. The name `blueline` comes from the blue line that separates the request and response panes in the terminal interface.

The initial implementation was done in a monolithic approach, where all the vi commmands are packed into a single file, which is archieved in `archive/mono/old_repl.rs`. As the codebase grew, it became difficult to manage and maintain so I refactored the codebase to follow the Model-View-Controller (MVC) pattern, which allowed for better separation of concerns and modularity. I also introduced packaged command specific business logics in Command Pattern.

The MVC refactoring has been successful and the app working very well, until I implemented the word wrapping feature, which required a more complex architecture to handle the display logic and cursor movement efficiently. I was one step away from completing the feature and got stuck with a big question mark in mind. At around this time, components was tightly coupled and there were a lot of violation of the Single Responsibility Principle (SRP) and Open/Closed Principle (OCP). The display logic was mixed with the command processing logic, making it difficult to test and maintain. I dediced to run the second application-wide refactoring to follow the Model-View-ViewModel (MVVM) pattern, which allows for better separation of concerns, event-driven, and better modularity.

The MVC version of the code is archived in `archive/mvc/`. The current MVVM version is in the `src/repl/` directory.

### Where Are We Now

The project is currently transitioning from MVC to MVVM/event-driven architecture. The current architecture is a mix of MVC and MVVM, where the ViewModel layer is being introduced to handle the display logic and cursor movement.The goal is to remove view concerns from Commands and centralize display logic in a ViewModel layer. See `docs/MVVM_TRANSITION_STRATEGY.md` for detailed implementation plan.

The implementation is still in progress. We are at Phase 3-4 in the documented strategy. The new MVVM foundation is being built with event-driven system and some of the fundamental commands are implemented. However due to the lose control of the Claude Code, the implementation started to get messed up. When I run the app in `cargo run`, it now looks totally different from the beautifully-crafted MVC version and is not functioning at all.

The reason of failure is because the refactoring approach was more in horizontally-split approach, where it split the phase by the application layer, rather than vertical split approach, where it splits the phase by feature. I wanted to implement one command at a time, but the Claude Code started to implement multiple commands at once, along with the framework transitions, which resulted in a lot of unfinished and broken code. The current state of the codebase is a mix of unfinished commands and broken display logic, which makes it difficult to test and maintain.

### Where Are We Going

The immediate goal is to restore the visual of the REPL terminal available in MVC codebase with the new MVVM architecture and limited implementation of the commands. The current implementation is not following what MVC codebase has implemented, like `crossterm`'s raw mode, Alternate Screen Buffer (ASB), and other terminal features. It does not have the Status Bar.

## Development Environment

### Primary Repository

https://github.com/samwisely75/blueline

### IDE

I am using Visual Studio Code with the following extensions:

- Rust Analyzer
- Copilot with Claude 4 Sonnet

### Language and Libraries

- **Language**: Rust (edition 2021)
- **Build System**: Cargo
- **Libraries**:
  - `crossterm`: For terminal I/O
  - `bluenote`: For HTTP requests with profiles that encapsulates `reqwest`
  - `anyhow`: For error handling
  - `cucumber`: For BDD testing  

### Coding Tools

- **Linting**: `cargo clippy` with strict warnings
- **Formatting**: `cargo fmt` for code formatting
- **Pre-commit Hooks**: Automatically run `cargo fmt` and `cargo clippy

### Test Tools

- **Unit Tests**: Standard Rust unit tests using `#[cfg(test)]` and `#[test]` attributes
- **Integration Tests**: Located in `tests/` directory, that runs cucumber features stored in `features/`. Mock view renderer using thread-local storage for testing without terminal I/O. Tests run sequentially to avoid resource conflicts.

### CI/CD Pipelines

Since this is a command line tool, it does not have a CD pipeline. CI and release automation is handled by GitHub Actions.

- **Continuous Integration**: `.github/workflows/ci.yml`
- **Release Automation**: `.github/workflows/release.yml`

### Documentations

All documents except `README.md` and `CLAUDE.md` are managed under `docs/` directory.

### Issue Tracking

Currently we are using `docs/ISSUES.md` to track issues and feature requests. We will migrate to GitHub Issues once the project is stable.

## Development Guidelines

### Coding Style

We strictly follow Rust's official style guide. Plus use `cargo fmt` for formatting, and `cargo clippy` for linting with strict warnings.

### Error Handling

- Uses `anyhow::Result` for error propagation
- Commands return `Result<()>` and errors are displayed in status bar
- Network errors show detailed connection information in verbose mode

### Build and Test

Run the following commands before commiting any chanages to repository:

```bash
# Build
cargo build
cargo build --release

# Run tests
cargo test
cargo test --test integration_tests

# Linting and formatting (REQUIRED before commits)
cargo fmt                   # Format code
cargo clippy --all-targets --all-features -- -D warnings  # Lint check
```

The project uses pre-commit hooks that enforce code quality:

- Automatically runs `cargo fmt` check
- Runs `cargo clippy` with strict warnings
- Rejects commits with any warnings

Install hooks: `./scripts/install-hooks.sh`

### Release Process

The release process is automated using GitHub Actions. We can trigger a release by pushing a release branch with a semantic version, such as `release/1.0.0`. The release process will:

- Build the project
- Run tests
- Create a release artifact
- Publish the release to GitHub
- Publish Homebrew formula to `samwisely75/tap` repository

### Coding Guidelines

Everyone, including Generative AI Engine like Copilot and Claude Code, must follow these guidelines when making changes to the codebase. These guidelines are sacred and must be strictly followed to ensure code quality, maintainability, and consistency across the project.

1. **Keep the change minimal**: Always respect and embrace the KISS, YAGNI, and DRY principles. Do not make large changes in one go. Make small, incremental changes that can be easily tested and reviewed. You must ensure the code compiles and passes tests at every change in a file, before moving on to the next. NEVER DO ANY EXTRA CHANGES OUTSIDE OF MY ORIGINAL REQUEST WITHOUT MY EXPLICIT PERMISSION. Inadequate is better than over-engineering.

1. **Ask for clarification first**: Before making ANY changes, ask specific questions like the followings. Never assume you know what the user wants without confirming:
   - "Which specific parts should I remove?"
   - "Should I preserve the existing functionality while removing only the unused abstractions?"
   - "Are you referring to removing unused interfaces or actual working features?"
   - "Do you also want me to implement Y to support a corner case Z?"

   Please display your questions in a bold text with a question mark icon in the beginning.

   And if you are asking multiple questions, please use a numbered list format for me to answer them by number.

   If you ask questions, do not proceed any further until you receive a clear answer.

1. **Preserve the original functionality**: Always keep the original functionality intact unless explicitly asked to remove it. If you are unsure about what to remove, ask for clarification. Do not remove any working methods, data structures, or functionality unless it is confirmed that they are not needed.

1. **Answer the question**: If you are asked a question, provide a direct answer. You don't know if that's meant to be a change request so NEVER change the code. If you see a point of improvement by the question, just suggest it and ask if I want to make the change.

1. **Keep the code clean**: We do refactoring a lot during the implementations and some codes would be remained unused. Always review your changes and unused warnings, and remove any unused code, imports, or variables. Do not leave any commented-out code in the final version. If you are unsure about whether to remove something, ask for clarification. Avoid using `#[allow(unused)]` attributes unless absolutely necessary. 

1. **Test it, test it, test it**: Always write unit tests. For **all** functions, without exception. The test codes will be a specification of the app. All the instructions I give must be written somewhere as the test code. Name the test functions to dictate the expected behavior clearly, e.g., `X_should_do_Y_and_return_Z()` where X is the target function. Sometimes you may need to write multiple tests for the same function to cover different scenarios. Do not persist inputs like profile INI and test request in a file; You must create them in the test code itself.

1. **Leave notes for others and future self**: Write comments that explain the purpose of the code and what problems it solves, focusing on consequences rather than just descriptions. Combine the objective and reasoning into natural, flowing explanations that describe what would happen without the code. For example, instead of saying "This validates input," explain "Validate input to prevent SQL injection attacks that would compromise the database." Use comments to explain the big picture and the reasoning behind complex logic, not what the code does line by line. The code itself should be self-documenting through descriptive function and variable names. Avoid marketing language like "sophisticated" or "advanced" - stick to technical facts. Always assume the reader has no prior knowledge of the code or libraries. Use `//!` for module-level documentation and `///` for function-level documentation.

1. **Handle errors in a standard way**: Use Rust's `Result` and `Option` types for error handling. Propagate errors using the `?` operator. Use anyhow as a standard error type for convenience. Use anyhow::Result for functions that can return errors.

1. **Use embedded expressions for format! macro**: Use embedded expressions for string formatting, e.g., `format!("Hello, {name}")` instead of `format!("Hello, {}", name)`. The latter is deprecated in Rust 2021 edition.

1. **Measure before claiming victory**: Run `cargo clippy --all-targets --all-features -- -D warnings` and `cargo fmt` before you say it's complete to ensure code quality and consistency at every change you make. I.e., do not have me stuck at the pre-commit stage and ask you to run these commands again and again.

1. **No mod.rs files**: Do not create `mod.rs` files. Instead, use the file name as the module name directly. For example, if you have a file named `commands.rs`, it should be declared as `pub mod commands;` in the parent module.

## Technical Notes

1. **Git commit message**: If the terminal command is too long, contains backtick and emojis, or contains special characters like `|`, `&`, `;`, or `>`, it may not be rendered correctly in the terminal. Git commit is the primary use case for this issue. In such cases, you can use the following workaround:
   - Use a single backtick for inline code formatting, e.g., \`command\`.
   - Use triple backticks for code blocks, e.g., \`\`\`bash
     command
     \`\`\`.
   - If the command is too long, split it into multiple lines using `\` at the end of each line.

## Development Workflow

1. Pick up the first unresolved item from `docs/ISSUES.md`.
2. Plan the implementation and todos. Ask for clarification if needed.
3. Update the feature file to dictate the specification of the feature. If the feature is already implemented, update the existing test to reflect the new behavior.
4. Implement the changes.
5. Create or update comprehensive unit tests for the changes.
6. Run all tests including integration tests to ensure everything works as expected.
7. Run `/scripts/git-commit-precheck.sh` to see if the code passes pre-commit checks. If it fails, fix the issues and run the script again until it passes.
   - This script will run `cargo clippy --all-targets --all-features -- -D warnings` and `cargo fmt` to ensure code quality and formatting.
   - If you are using a Generative AI Engine like Claude Code, make sure to run this script before committing any changes.
8. Notify Master about the changes and ask for review.
9. Address any feedback and make necessary changes.
10. Repeat the process until Master approves the changes.
11. Once approved, check off the item in `docs/ISSUES.md` and update the issue status to "Done".
12. Increment the version number in `Cargo.toml`. If a new feature is added, increment the minor version. If a bug is fixed, increment the patch version. If a breaking change is made, increment the major version.
13. Update the changelog in `docs/CHANGELOG.md` with a summary of the changes made.
14. Commit all changes with a clear and concise commit message. Run `cargo clippy --all-targets --all-features -- -D warnings` and `cargo fmt` again to ensure the code is clean and formatted.
15. Create a git tag for the same version number with "v", e.g., `git tag v1.0.0`.
16. Go to the step 1.
