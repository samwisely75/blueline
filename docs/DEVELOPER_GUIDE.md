# Developer's Guide for Blueline

This file provides guidance to human developers and Generative AI Engines when working with code in this repository.

## Table of Contents

- [Application Overview](#application-overview)
- [Project](#project)
  - [History](#history)
  - [Where Are We Now](#where-are-we-now)
  - [Where Are We Heading](#where-are-we-heading)
- [Development Environment](#development-environment)
  - [Primary Repository](#primary-repository)
  - [IDE](#ide)
  - [Language and Libraries](#language-and-libraries)
  - [Coding Tools](#coding-tools)
  - [Test Tools](#test-tools)
  - [CI/CD Pipelines](#cicd-pipelines)
  - [Documentations](#documentations)
  - [Issue Tracking](#issue-tracking)
- [Development Guidelines](#development-guidelines)
  - [Coding Style](#coding-style)
  - [Error Handling](#error-handling)
  - [Build and Test](#build-and-test)
  - [Release Process](#release-process)
  - [Coding Guidelines](#coding-guidelines)
- [Technical Notes](#technical-notes)
- [Development Workflow](#development-workflow)

## Application Overview

This application is a Rust-based HTTP client named `blueline`, previously known as `httpc` and `webly`. It is designed to be lightweight and profile-based, allowing users to manage HTTP requests easily. The project has been recently renamed and restructured, with a focus on REPL.

This application is built using the `crossterm` library for terminal I/O and `bluenote`, my another project enablese to make HTTP requests with profiles like `awscli`. The goal of this project is to mimic vi as closely as possible while providing an easy way to interact with HTTP endpoints in a terminal environment.

## Project

### History

I first started this project as a simple one-off HTTP command line tool like `curl`, which is left as [`httpc`](https://github.com/samwisely75/httpc), which incorporating the idea of profile in `awscli`. The app is nearly built, and I started adding a REPL interface to it to integrate Vim + Postman into one terminal app. The branch soon became too large to maintain as a single project so I decided to split the REPL capabilities as an independent project. That's the birth of `blueline`. The REPL interface is inspired by the vi editor, allowing users to navigate and manipulate HTTP requests and responses in a terminal environment. The name `blueline` comes from the blue line that separates the request and response panes in the terminal interface.

The initial implementation was done in a monolithic approach, where all the vi commmands are packed into a single file, which is archieved in `archive/mono/old_repl.rs`. As the codebase grew, it became difficult to manage and maintain so I refactored the codebase to follow the Model-View-Controller (MVC) pattern, which allowed for better separation of concerns and modularity. I also introduced packaged command specific business logics in Command Pattern.

The MVC refactoring has been successful and the app working very well, until I implemented the word wrapping feature, which required a more complex architecture to handle the display logic and cursor movement efficiently. I was one step away from completing the feature and got stuck with a big question mark in mind. At around this time, components was tightly coupled and there were a lot of violation of the Single Responsibility Principle (SRP) and Open/Closed Principle (OCP). The display logic was mixed with the command processing logic, making it difficult to test and maintain. I dediced to run the second application-wide refactoring to follow the Model-View-ViewModel (MVVM) pattern, which allows for better separation of concerns, event-driven, and better modularity.

The MVC version of the code is archived in `archive/mvc/`. The current MVVM version is in the `src/repl/` directory.

The codebase is successfully transitioned from MVC to MVVM/event-driven architecture. The architecture is a mix of MVC and MVVM, where the ViewModel layer is being introduced to handle the display logic and cursor movement. Please see `MVVM_ARCHITECTURE.md` for the details of the architecture.

### Where Are We Now

MVVM transition is nearly complete and fixing the last few issues. The REPL interface is working well, but still lacks some features like yank and paste, command history, and syntax highlighting. Thanks to the new architecture, the codebase is now more modular and easier to maintain.

### Where Are We Heading

Once all the basic vim commands are implemented, we will release it as v1.0.0. After that, we will make the app connect to other user's console and collaborate over the terminal. The goal is to make the app useful on the field of consulting, where clients needs consultant's help to update/troubleshoot their systems via REST APIs. The app will allow users to share their terminal session with others, making it easy to collaborate and troubleshoot issues in real-time.

## Development Environment

### Primary Repository

<https://github.com/samwisely75/blueline>

### IDE

Visual Studio Code with the following extensions:

- Claude Code plugin
- Copilot with Claude 4 Sonnet primarily
- Rust Analyzer plugin

### Language and Libraries

- **Language**: Rust (edition 2021)
- **Build System**: Cargo
- **Depending Libraries (Major Ones Only)**:
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

   Please display your questions in a bold text with a question mark icon in the beginning. If you ask multiple questions, please use a numbered list for me to answer them with the number. And if you ask questions, do not proceed any further until you receive a clear answer.

1. **Preserve the original functionality**: Always keep the original functionality intact unless explicitly asked to remove it. If you are unsure about what to remove, ask for clarification. Do not remove any working methods, data structures, or functionality unless it is confirmed that they are not needed.

1. **Answer the question**: If you are asked a question, provide a direct answer. You don't know if that's meant to be a change request so NEVER change the code. If you see a point of improvement by the question, just suggest it and ask if I want to make the change.

1. **Keep the code clean**: We do refactoring a lot during the implementations and some codes would be remained unused. Always review your changes and unused warnings, and remove any unused code, imports, or variables. Do not leave any commented-out code in the final version. If you are unsure about whether to remove something, ask for clarification. Avoid using `#[allow(unused)]` attributes unless absolutely necessary.

1. **Test it, test it, test it**: Always write unit tests. For **all** functions, without exception. The test codes will be a specification of the app. All the instructions I give must be written somewhere as the test code. Name the test functions to dictate the expected behavior clearly, e.g., `X_should_do_Y_and_return_Z()` where X is the target function. Sometimes you may need to write multiple tests for the same function to cover different scenarios. Do not persist inputs like profile INI and test request in a file; You must create them in the test code itself.

1. **Leave notes for others and future self**: Write comments that explain the purpose of the code and what problems it solves, focusing on consequences rather than just descriptions. Combine the objective and reasoning into natural, flowing explanations that describe what would happen without the code. For example, instead of saying "This validates input," explain "Validate input to prevent SQL injection attacks that would compromise the database." Use comments to explain the big picture and the reasoning behind complex logic, not what the code does line by line. The code itself should be self-documenting through descriptive function and variable names. Avoid marketing language like "sophisticated" or "advanced" - stick to technical facts. Always assume the reader has no prior knowledge of the code or libraries. Use `//!` for module-level documentation and `///` for function-level documentation.

1. **Handle errors in a standard way**: Use Rust's `Result` and `Option` types for error handling. Propagate errors using the `?` operator. Use anyhow as a standard error type for convenience. Use anyhow::Result for functions that can return errors.

1. **Use embedded expressions for format! macro**: Use embedded expressions for string formatting, e.g., `format!("Hello, {name}")` instead of `format!("Hello, {}", name)`. The latter is deprecated in Rust 2021 edition.

1. **Measure before claiming victory**: Run `cargo clippy --all-targets --all-features -- -D warnings` and `cargo fmt` before you say it's complete to ensure code quality and consistency at every change you make. I.e., do not have me stuck at the pre-commit stage and ask you to run these commands again and again.

1. **Git commit message**: If the terminal command is too long, contains backtick and emojis, or contains special characters like `|`, `&`, `;`, or `>`, it may not be rendered correctly in the terminal. Git commit is the primary use case for this issue. In such cases, you can use the following workaround:
   - Use a single backtick for inline code formatting, e.g., \`command\`.
   - Use triple backticks for code blocks, e.g., \`\`\`bash
     command
     \`\`\`.
   - If the command is too long, split it into multiple lines using `\` at the end of each line.

## Markdown Guidelines

1. **Follow the Markdown standards**: Use the standard Markdown syntax for headings, lists, code blocks, and links. Do not use any non-standard syntax or extensions.
2. **Empty line after headings**: Always leave an empty line after a heading to ensure proper rendering in Markdown viewers.

## Development Workflow

Developers and Generative AI Engines like Claude Code should strictly follow this workflow to implement new features or fix bugs in the codebase. This process ensures that changes are made systematically, tested thoroughly, and documented properly.

1. Pick up the top-most item in the `Ready` state on [blueline GitHub Kanban](https://github.com/users/samwisely75/projects/1) by the following command. If the result is empty, ask for it.

   ```shell
   gh project item-list 1 --owner samwisely75 --format json --jq '.items[] | select(.status == "Ready")'
   ```

2. Plan the implementation and todos. Ask for clarification if needed.
3. Create a new branch from the `develop` branch with a descriptive name, e.g., `feature/new-feature` or `bugfix/fix-issue-123`.
4. Move the Kanban item to the `In progress` state. The command is:

   ```shell
   gh project item-edit \
       --id $item_id \
       --field-id PVTSSF_lAHODQVrPs4A_FfHzgyS000 \
       --single-select-option-id 47fc9ee4 \
       --project-id PVT_kwHODQVrPs4A_FfH
   ```

5. Update the feature file to dictate the specification of the feature. If the feature is already implemented, update the existing test to reflect the new behavior.
6. Implement the changes.
7. Create or update comprehensive unit tests for the changes.
8. Run all tests including integration tests to ensure everything works as expected.
9. Make sure to leave comments following the coding guidelines. It is particularly important to leave comments when you change the code for a bug fix.
10. Run `/scripts/git-commit-precheck.sh` to see if the code passes pre-commit checks. If it fails, fix the issues and run the script again until it passes.
    - This script will run `cargo clippy --all-targets --all-features -- -D warnings` and `cargo fmt` to ensure code quality and formatting.
    - If you are using a Generative AI Engine like Claude Code, make sure to run this script before committing any changes.
11. Commit all changes with a clear and concise commit message. Do not leave out the code files you updated to fix clippy warnings and what `cargo fmt` modified.
12. Create a pull request (PR) with a clear and concise description of the changes made, including the issue number if applicable.
    - The PR title should be descriptive and follow the format `Fix #issue_number: Short description of the change`.
    - The PR description should include:
      - A summary of the changes made
      - The issue number(s) related to the changes
      - Any additional context or information that may be helpful for reviewers
13. Move the Kanban item to the `In Review` state. The command is:

   ```shell
   gh project item-edit \
       --id $item_id \
       --field-id PVTSSF_lAHODQVrPs4A_FfHzgyS000 \
       --single-select-option-id df73e18b \
       --project-id PVT_kwHODQVrPs4A_FfH
   ```

14. Address any feedback on PR and make necessary changes.
15. Increment the version number in `Cargo.toml`. If a new feature is added, increment the minor version. If a bug is fixed, increment the patch version. If a breaking change is made, increment the major version.
16. Update the changelog in `docs/CHANGELOG.md` with a summary of the changes made.
17. Create a git tag for the same version number with "v", e.g., `git tag v1.0.0`.
18. Go to step 1.
