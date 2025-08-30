# Agent Command Migration Guide

## Overview

This guide provides step-by-step instructions for agents performing command system migrations using the new **Dynamic Command Discovery System**. This system enables **zero-conflict parallel development** for unlimited agents working simultaneously.

## 🔧 Prerequisites

### Required Reading (CRITICAL)
**Agents MUST read and strictly follow these documents before starting:**

1. **`docs/DEV_GUIDE.md`** - Core development guidelines and architecture
2. **`docs/DEV_WORKFLOW.md`** - Complete development workflow and process  
3. **`docs/DEV_CODING.md`** - Coding standards and formatting requirements
4. **`SESSION_NOTES.md`** - Current session context and previous work

### System Requirements
- Temporary repository clone (NOT primary repo)
- Git configured with proper credentials
- Rust toolchain with `cargo` available
- Access to GitHub CLI (`gh`) for PR management

## 🚀 Zero-Conflict Migration Workflow

### Step 1: Environment Setup

```bash
# 1. Clone to temporary location using proper directory structure
cd ~/Sources/samwisely75/rust/temp
git clone https://github.com/samwisely75/blueline.git blueline-dev-[ISSUE_NUMBER]
cd blueline-dev-[ISSUE_NUMBER]

# 2. Checkout develop branch and create feature branch
git checkout develop
git pull origin develop
git checkout -b feature/[command-name]-command-[issue-number]

# 3. Verify you're in the right location
pwd  # Should show: ~/Sources/samwisely75/rust/temp/blueline-dev-[ISSUE_NUMBER]
```

### Step 2: Analysis and Planning

```bash
# 1. Read current context and guidelines
cat SESSION_NOTES.md
cat docs/DEV_GUIDE.md
cat docs/DEV_WORKFLOW.md
cat docs/DEV_CODING.md

# 2. Examine the target method in AppViewModel
grep -n "handle_[method_name]" src/repl/view_models/app_view_model.rs

# 3. Study existing unified commands for patterns
ls src/repl/unified_commands/*.rs
# Look at similar commands like yank.rs, cut_selection.rs, etc.
```

### Step 3: Implementation (The Revolutionary Way)

#### A. Create the Unified Command File

```bash
# Create new command file
touch src/repl/unified_commands/[command_name].rs
```

#### B. Implement Following 3G Framework Pattern

```rust
//! # [CommandName] Command
//!
//! Description of command functionality and purpose

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::repl::models::events::view_events::ViewEvent;
use crate::repl::models::pane_state::EditorMode;
use crate::repl::unified_commands::{Command, CommandContext, ExecutionContext};
use crate::register_command;  // Import the macro

/// [Command description]
#[derive(Debug, Default)]
pub struct [CommandName];

impl [CommandName] {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Command for [CommandName] {
    fn is_relevant(&self, key_event: KeyEvent, mode: EditorMode, context: &CommandContext) -> bool {
        // TODO: Implement relevance logic based on existing handle_* method
        // Follow patterns from existing commands
    }

    fn execute(&self, context: &mut ExecutionContext) -> Result<Vec<ViewEvent>> {
        // TODO: Port business logic from existing handle_* method
        // Emit appropriate ViewEvents for UI updates
        Ok(vec![])
    }

    fn name(&self) -> &'static str {
        "[CommandName]"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // TODO: Add comprehensive unit tests following existing patterns
}

// 🚀 THIS IS THE MAGIC LINE - ZERO CONFLICTS!
register_command!([CommandName], "[CommandName]");
```

#### C. Update Module Exports (ZERO CONFLICTS!)

🚀 **Revolutionary Update**: We now have **TRUE ZERO CONFLICTS** through reserved placeholders!

```bash
# Find your assigned GitHub issue number and comment line in mod.rs
grep -n "Issue #[YOUR_ISSUE_NUMBER]" src/repl/unified_commands/mod.rs

# Replace YOUR assigned comment line with the actual module declaration
# Example: Replace "// pub mod move_left;" with "pub mod move_left;"
sed -i 's|// pub mod [your_command_name];|pub mod [your_command_name];|' src/repl/unified_commands/mod.rs
```

**How the Zero-Conflict System Works:**

- ✅ **Pre-allocated Placeholders**: All command slots (issues #231-295) are pre-reserved as comments
- ✅ **Assigned Replacements**: Each agent replaces only their assigned comment line
- ✅ **No Line Additions**: No new lines added = **zero conflicts**  
- ✅ **Comment to Code**: Simple replacement operation impossible to conflict
- ✅ **Parallel Safety**: 100+ agents can work simultaneously without any conflicts

**Example Workflow:**
```bash
# Your issue #235: InsertCharCommand
# Before: // pub mod insert_char;
# After:  pub mod insert_char;

# Use sed for safe replacement
sed -i 's|// pub mod insert_char;|pub mod insert_char;|' src/repl/unified_commands/mod.rs
```

**Verification:**
```bash
# Verify your change was applied correctly
grep "pub mod [your_command_name];" src/repl/unified_commands/mod.rs
```

This is the **silver bullet** - true zero-conflict parallel development! 🎯

#### D. Remove Old AppViewModel Method

```rust
// In src/repl/view_models/app_view_model.rs
// Comment out or remove the handle_[method_name] method
// Update event handling to ignore the old command event
```

### Step 4: Testing (Unit Tests Only)

```bash
# Run unit tests - integration tests will run on GitHub Actions
cargo test [command_name] --lib
cargo test unified_commands::dynamic_registry --lib

# Run full unit test suite  
cargo test --lib --quiet

# IMPORTANT: Skip integration tests locally due to Mac 32+ thread limitations
# Integration tests will run automatically on GitHub Actions CI/CD
```

### Step 5: Quality Assurance (MANDATORY)

```bash
# ALWAYS run the mandatory precheck before any commit
./scripts/git-commit-precheck.sh

# Fix any issues reported by clippy, fmt, or tests
# Follow formatting guidelines from docs/DEV_CODING.md
```

### Step 6: Commit and PR Creation

```bash
# Commit with proper message format
git add -A
git commit -m "feat: Migrate handle_[method_name] to [CommandName]

- Create [CommandName] following 3G framework pattern
- Port business logic from handle_[method_name] method
- Add comprehensive unit tests
- Register command using dynamic discovery system
- Remove old method from AppViewModel

Fixes #[ISSUE_NUMBER] - Part of command system refactoring #224"

# Push branch and create PR
git push origin feature/[command-name]-command-[issue-number]
gh pr create \
  --title "Fix #[ISSUE_NUMBER]: Migrate handle_[method_name] to [CommandName]" \
  --body "## Summary
Migrates \`handle_[method_name]\` to the new unified command system.

## Changes
- ✅ Created \`[CommandName]\` with 3G framework pattern
- ✅ Ported all business logic from existing method
- ✅ Added comprehensive unit tests
- ✅ Used dynamic discovery system (zero conflicts!)
- ✅ Removed old method from AppViewModel

## Testing
- ✅ Unit tests pass
- ✅ Integration tests will run on CI

## Related Issues
- Fixes #[ISSUE_NUMBER]
- Part of #224 (command system refactoring)

🎯 This uses the new dynamic discovery system - zero merge conflicts!"
```


## 🚨 Critical Requirements

### Must-Follow Standards

1. **Read Required Docs**: DEV_GUIDE.md, DEV_WORKFLOW.md, DEV_CODING.md, SESSION_NOTES.md
2. **Use Temporary Repos**: Never work in primary repo
3. **Follow 3G Framework**: is_relevant() + execute() + comprehensive tests
4. **Run Precheck Script**: `./scripts/git-commit-precheck.sh` before every commit
5. **Proper Error Handling**: Use Result types and proper error propagation
6. **Formatting Guidelines**: Use embedded expressions like `format!("Hello, {name}")`

### Testing Strategy

- ✅ **Unit Tests**: Run locally, must pass
- ✅ **Integration Tests**: Skip locally (Mac thread limits), run on GitHub Actions
- ✅ **Precheck Script**: Always run before commit

### Quality Gates

- All unit tests pass
- Clippy warnings fixed
- Code properly formatted  
- Business logic preserved
- ViewEvents properly emitted

## 🎯 Success Criteria

### For Each Migration

- [ ] Target method completely migrated to unified command
- [ ] All business logic preserved and tested
- [ ] Command uses `register_command!` macro (zero conflicts!)
- [ ] Old method removed from AppViewModel
- [ ] Comprehensive unit tests added
- [ ] PR created with proper description
- [ ] CI tests pass (integration tests run on GitHub Actions)

### For Parallel Development

- [ ] Multiple agents can work simultaneously without conflicts
- [ ] Each agent works in separate temporary repo clone
- [ ] Registry conflicts eliminated via dynamic discovery
- [ ] PRs merge cleanly without manual conflict resolution

## 📚 Reference Materials

### Essential Reading

- `docs/DEV_GUIDE.md` - Architecture and patterns
- `docs/DEV_WORKFLOW.md` - Complete development process
- `docs/DEV_CODING.md` - Coding standards and formatting  
- `SESSION_NOTES.md` - Current session context

### Code Examples

- `src/repl/unified_commands/yank.rs` - YankSelectionCommand pattern
- `src/repl/unified_commands/change_selection.rs` - Selection command pattern
- `src/repl/unified_commands/visual_block_insert.rs` - Visual block pattern

### System Architecture

- `src/repl/unified_commands/dynamic_registry.rs` - Dynamic discovery system
- `src/lib.rs` - register_command! macro definition
- `src/repl/view_models/app_view_model.rs` - Integration point

---

## Summary

This guide enables unlimited parallel agent development with zero conflicts. The dynamic discovery system is the silver bullet for scaling command system refactoring! 🚀
