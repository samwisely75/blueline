# Developer's Guide for Blueline

This file provides guidance to human developers and Generative AI Engines when working with code in this repository.

## Table of Contents

- [Application Overview](#application-overview)
- [Project](#project)
  - [History](#history)
  - [Where Are We Now](#where-are-we-now)
  - [Where Are We Heading](#where-are-we-heading)
- [Requirements](#requirements)
- [Application Architecture](#application-architecture)
- [Development Environment](#development-environment)
  - [Primary Repository](#primary-repository)
  - [IDE](#ide)
  - [Language and Libraries](#language-and-libraries)
  - [Coding Tools](#coding-tools)
  - [Test Tools](#test-tools)
  - [CI/CD Pipelines](#cicd-pipelines)
  - [Issue Tracking](#issue-tracking)
- [Development Guidelines](#development-guidelines)
  - [Coding Style](#coding-style)
  - [Build, Test and Debug](#build-test-and-debug)
  - [Release Process](#release-process)
- [Documentation Guidelines](#documentation-guidelines)

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

## Requirements

All the requirements are documented in a form of Gherkin feature files under the `features/` directory. The feature files are written in a way that describes the expected behavior of the application in a human-readable format. The feature files are used to generate tests using the `cucumber` library, which allows us to run the tests against the application and verify that it behaves as expected.

## Application Architecture

The architecture of the application is based on the Model-View-ViewModel (MVVM) pattern, which allows for better separation of concerns and modularity. Please see `ARCHITECTURE.md` for the detail designs and implementations.

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

### Issue Tracking

We use GitHub Issues for tracking bugs, feature requests, and tasks. The issues are organized into a Kanban board using GitHub Projects. Please check the [Development Workflow](docs/DEV_WORKFLOW.md) for the details of the workflow.

## Development Guidelines

### Coding Style

We strictly follow Rust's official style guide. Plus use `cargo fmt` for formatting, and `cargo clippy` for linting with strict warnings. We also follow the coding guidelines in [Coding Guidelines](docs/DEV_CODING.md) to ensure code quality, maintainability, and consistency across the project. These guidelines are sacred and must be strictly followed to ensure code quality, maintainability, and consistency across the project.

### Build, Test and Debug

The strategy for building and debugging the application is documented in the [Debugging Strategy](docs/DEV_DEBUG.md). The strategy includes:

- Terminal emulation
- Debug logging
- Cursor position tracking

For the integration tests, we use `cucumber` to run the feature files in the `features/` directory. The tests are designed to cover the expected behavior of the application as described in the feature files. The tests are run using the `cargo test` command, which will automatically run all the tests in the project.
Please also refer to the [Test Architecture](docs/TEST_ARCHITECTURE.md) and [Rules for Testing](docs/TEST_RULES.md) for the guidelines on writing tests and ensuring code quality.

The precheck for the git commit is automated. Please run `./scripts/git-commit-precheck.sh` before commiting any chanages to repository.

### Release Process

The release process is automated using GitHub Actions. We can trigger a release by pushing a release branch with a semantic version, such as `release/1.0.0`. The release process will:

- Build the project
- Run tests
- Create a release artifact
- Publish the release to GitHub
- Publish Homebrew formula to `samwisely75/tap` repository

## Documentation Guidelines

1. **Document Store**: All documents except `README.md` and `CLAUDE.md` are managed under `docs/` directory.
1. **Markdown**: All documents are written in Markdown format. Use standard Markdown syntax for headings, lists, code blocks, and links. Always leave an empty line after a heading to ensure proper rendering in Markdown viewers. Use `markdownlint` command to check and fix the Markdown syntax errors.
