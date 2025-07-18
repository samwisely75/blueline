---
applyTo: '**'
---
Provide project context and coding guidelines that AI should follow when generating code, answering questions, or reviewing changes.

# Project Context

This project is a Rust-based HTTP client named `httpc`, previously known as `webly`. It is designed to be lightweight and profile-based, allowing users to manage HTTP requests easily. The project has been recently renamed and restructured, with a focus on improving usability and functionality.

## REPL mode

### Input / Output Pane

The project includes a REPL (Read-Eval-Print Loop) mode that allows users to interactively test HTTP requests. This mode is designed to be user-friendly and supports various HTTP methods, headers, and body content.

Since we are using terminal UI, I will split the terminal to two logical panes: Input and Output. The Input pane is where users can type their HTTP requests, and the Output pane displays the results of those requests. The Like the vi's multiple windows mode, user can switch between the Input and Output panes using keyboard shortcuts and navigate though the content by moving the cursor. 

The Input pane supports syntax highlighting for HTTP methods, headers, and body content, making it easier to read and write requests.

Both panes support basic vi commands (discussed below) for navigation and editing, allowing users to use familiar commands to interact with the REPL. The Output pane is for read-only; users cannot edit the output.

Think the app has two buffers simultaneously.

### Vi commands

The app supports Vi-style commands for navigation and editing within the REPL mode. Users can use familiar Vi commands to move around, edit their input, and execute HTTP requests.

# Coding Guidelines

1. **Code Structure**: Organize code into modules and packages logically. Follow Rust's conventions for module naming and organization.

2. **Error Handling**: Use Rust's `Result` and `Option` types for error handling. Propagate errors using the `?` operator. Use anyhow as a standard error type for convenience.

3. **Testing**: Write unit tests for all public functions. Use the `#[cfg(test)]` module for test cases.

4. **Security**: Follow best practices for secure coding, especially when handling user input and making network requests.

5. **Dependencies**: Keep dependencies up to date and minimize the use of external crates where possible.

6. **Code Reviews**: Participate in code reviews and provide constructive feedback to peers.

7. **String Formatting**: Use embedded expressions for string formatting, e.g., `format!("Hello, {name}")` instead of `format!("Hello, {}", name)`. The latter is deprecated in Rust 2021 edition.

8. **Continuous Integration**: Use CI/CD pipelines to automate testing and deployment processes.

9. Run cargo clippy and cargo fmt before submitting pull requests to ensure code quality and consistency.