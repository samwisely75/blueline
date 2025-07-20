---
applyTo: '**'
---
Provide project context and coding guidelines that AI should follow when generating code, answering questions, or reviewing changes.

# Project Context

This project is a Rust-based HTTP client named `httpc`, previously known as `webly`. It is designed to be lightweight and profile-based, allowing users to manage HTTP requests easily. The project has been recently renamed and restructured, with a focus on improving usability and functionality.

## REPL mode

### Request / Response Pane

The project includes a REPL (Read-Eval-Print Loop) mode that allows users to interactively test HTTP requests. This mode is designed to be user-friendly and supports various HTTP methods, headers, and body content.

Since we are using terminal UI, I will split the terminal to two logical panes: Request and Response. The Request pane is where users can type their HTTP requests, and the Response pane displays the results of those requests. The Like the vi's multiple windows mode, user can switch between the Request and Response panes using keyboard shortcuts and navigate though the content by moving the cursor. 

The Request pane supports syntax highlighting for HTTP methods, headers, and body content, making it easier to read and write requests.

Both panes support basic vi commands (discussed below) for navigation and editing, allowing users to use familiar commands to interact with the REPL. The Response pane is for read-only; users cannot edit the output.

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

10. **Commenting**: Write comments that explain the purpose of the code and what problems it solves, focusing on consequences rather than just descriptions. Combine the objective and reasoning into natural, flowing explanations that describe what would happen without the code. For example, instead of saying "This validates input," explain "Validate input to prevent SQL injection attacks that would compromise the database." Use comments to explain the big picture and the reasoning behind complex logic, not what the code does line by line. The code itself should be self-documenting through descriptive function and variable names. Avoid marketing language like "sophisticated" or "advanced" - stick to technical facts. Always assume the reader has no prior knowledge of the code or libraries. Use `//!` for module-level documentation and `///` for function-level documentation.

# Refactoring Guidelines

## When Asked to "Remove" or "Simplify" Code

**CRITICAL**: When the user asks to remove complexity or features that "weren't in the original", follow these strict guidelines:

1. **Ask for Clarification First**: Before making ANY changes, ask specific questions:
   - "Which specific parts should I remove?"
   - "Should I preserve the existing functionality while removing only the unused abstractions?"
   - "Are you referring to removing unused interfaces or actual working features?"

2. **Preserve ALL Working Functionality**: Never remove or gut working features. The user's request to "remove what was NOT there in the original" means:
   - Remove unused abstract interfaces that add complexity
   - Remove over-engineered patterns that aren't being used
   - Remove dead code and empty implementations
   - **DO NOT** remove actual working methods, data structures, or functionality

3. **Make Incremental Changes**: 
   - Change one file at a time
   - Test compilation after each change
   - Ask for feedback before proceeding to the next component

4. **Follow KISS and YAGNI Correctly**:
   - KISS = Keep the working code simple, remove unnecessary abstractions
   - YAGNI = Remove features/interfaces that aren't currently needed
   - **NOT** = Remove all features and leave empty stubs

5. **When in Doubt, STOP and ASK**: If unclear about scope, always ask for clarification rather than making assumptions.

## Design Pattern Implementation

When implementing design patterns, ensure they add value without over-engineering:
- Only implement what's immediately needed
- Avoid creating elaborate hierarchies of unused interfaces
- Keep the original functionality intact while adding pattern structure
- Remember: patterns should simplify code maintenance, not complicate it