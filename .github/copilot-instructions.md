# blueline Copilot Instructions
---
Provide project context and coding guidelines that AI should follow when generating code, answering questions, or reviewing changes.

## Project Context

This project is a Rust-based HTTP client  named `blueline`, previously known as `httpc` and `webly`. It is designed to be lightweight and profile-based, allowing users to manage HTTP requests easily. The project has been recently renamed and restructured, with a focus on REPL.

The goal of this project is to mimic vi as closely as possible with providing a easy way to interact with HTTP endpoints in a terminal environment. 

## Refactoring to MVC 

The project is being refactored to follow the Model-View-Controller (MVC) architecture. The previous monolithic structure is preserved in /archive/old_repl.rs. Please refer to this file for the original implementation and functionality. The new structure will separate concerns into distinct components:

## General Coding Guidelines

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

1. **Keep the code clean**: I do refactoring a lot during the implementations and some codes would be remained unused. Always review your changes and unused warnings, and remove any unused code, imports, or variables. Do not leave any commented-out code in the final version. If you are unsure about whether to remove something, ask for clarification.

1. **Test it, test it, test it**: Always write unit tests. For **all** functions, without exception. The test codes will be a specification of the app. All the instructions I give must be written somewhere as the test code. Name the test functions to dictate the expected behavior clearly, e.g., `X_should_do_Y_and_return_Z()` where X is the target function. Sometimes you may need to write multiple tests for the same function to cover different scenarios. Do not persist inputs like profile INI and test request in a file; You must create them in the test code itself. 

1. **Leave notes for others and future self**: Write comments that explain the purpose of the code and what problems it solves, focusing on consequences rather than just descriptions. Combine the objective and reasoning into natural, flowing explanations that describe what would happen without the code. For example, instead of saying "This validates input," explain "Validate input to prevent SQL injection attacks that would compromise the database." Use comments to explain the big picture and the reasoning behind complex logic, not what the code does line by line. The code itself should be self-documenting through descriptive function and variable names. Avoid marketing language like "sophisticated" or "advanced" - stick to technical facts. Always assume the reader has no prior knowledge of the code or libraries. Use `//!` for module-level documentation and `///` for function-level documentation.

1. **Handle errors in a standard way**: Use Rust's `Result` and `Option` types for error handling. Propagate errors using the `?` operator. Use anyhow as a standard error type for convenience. Use anyhow::Result for functions that can return errors.

1. **Use embedded expressions for format! macro**: Use embedded expressions for string formatting, e.g., `format!("Hello, {name}")` instead of `format!("Hello, {}", name)`. The latter is deprecated in Rust 2021 edition.

1. **Measure before claiming victory**: Run `cargo clippy --all-targets --all-features -- -D warnings` and `cargo fmt` before you say it's complete to ensure code quality and consistency at every change you make. I.e., do not have me stuck at the pre-commit stage and ask you to run these commands again and again.

1. **No mod.rs files**: Do not create `mod.rs` files. Instead, use the file name as the module name directly. For example, if you have a file named `commands.rs`, it should be declared as `pub mod commands;` in the parent module.

## Technical Difficulties

1. **Git commit message**: If the terminal command is too long, contains backtick and emojis, or contains special characters like `|`, `&`, `;`, or `>`, it may not be rendered correctly in the terminal. Git commit is the primary use case for this issue. In such cases, you can use the following workaround:
   - Use a single backtick for inline code formatting, e.g., \`command\`.
   - Use triple backticks for code blocks, e.g., \`\`\`bash
     command
     \`\`\`.
   - If the command is too long, split it into multiple lines using `\` at the end of each line. 
