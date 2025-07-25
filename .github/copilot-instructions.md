---
Provide project context and coding guidelines that AI should follow when generating code, answering questions, or reviewing changes.

# Project Context

This project is a Rust-based HTTP client  named `blueline`, previously known as `httpc` and `webly`. It is designed to be lightweight and profile-based, allowing users to manage HTTP requests easily. The project has been recently renamed and restructured, with a focus on REPL.

The goal of this project is to mimic vi as closely as possible with providing a easy way to interact with HTTP endpoints in a terminal environment. 

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

# Core Development Principles from Refactoring Experience

## Mindset for Sustainable Code Evolution

**DRY (Don't Repeat Yourself)**: When you see the same logic pattern 3+ times, extract it into reusable abstractions. Look beyond surface-level code similarity to identify conceptual patterns that can be generalized through traits or helper functions.

**Preserve First, Improve Second**: Never sacrifice working functionality for architectural beauty. The best refactoring maintains identical external behavior while improving internal structure. If it breaks, it's not refactoring—it's rewriting.

**Measure Before Claiming Victory**: Quantify improvements with concrete metrics (line count, modularity, complexity). If you can't measure the improvement, question whether the change was worth the effort.

**Incremental Courage Over Big Bang**: Large architectural changes succeed through small, verifiable steps. Each step should compile, test, and function identically to the previous state. Bold vision, careful execution.

**Recognize Natural Boundaries**: Code wants to separate along natural fault lines—data vs. presentation vs. control logic. Listen to where the code wants to split rather than forcing artificial divisions.

**Early Filtering Patterns**: When processing events or commands, filter irrelevant cases as early as possible (like `is_relevant()` methods). This prevents unnecessary computation and keeps hot paths lean.

**Observer Pattern for Loose Coupling**: When one component needs to react to changes in another, prefer observer patterns over direct coupling. This enables independent evolution of components.

**Traits for Behavioral Contracts**: Use traits to define behavioral contracts that multiple types can implement. This enables code reuse while maintaining type safety and clear interfaces.

## Refactoring Wisdom

**Start with Working Code**: You cannot improve what doesn't work. Get it working first, then make it better. Working ugly code beats elegant broken code every time.

**Know When to Stop**: Not every piece of code needs to be perfectly architected. Sometimes "good enough" really is good enough. Focus optimization efforts where they matter most.

**Embrace Temporary Ugliness**: During refactoring, code may temporarily become uglier before it becomes cleaner. This is normal and necessary—don't abandon the effort during the awkward middle phase.

**Test the Transformation**: The only way to verify that refactoring preserved behavior is through testing. If the code isn't testable, make it testable first, then refactor.