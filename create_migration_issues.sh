#!/bin/bash

# Script to create GitHub issues for migrating command structs to unified command system

# Function to create a migration issue
create_issue() {
    local command_name="$1"
    local file_path="$2"
    local title="Migrate ${command_name} to unified command system"
    local body="## Description
Migrate the ${command_name} struct from the legacy command system to the new unified command system.

## Background
As part of the architectural migration described in issue #226, we need to move all command implementations from the legacy system to the unified command system. This command currently uses the old CommandEvent-based architecture and needs to be converted to the new UnifiedCommand trait.

## Current Implementation
- Location: ${file_path}
- Uses legacy Command trait with CommandEvent emission
- Processed through apply_command_event method in AppViewModel

## Migration Requirements
1. Create new unified command implementation
2. Implement UnifiedCommand trait instead of Command trait
3. Return ViewEvent instead of CommandEvent
4. Handle business logic directly instead of delegating to AppViewModel
5. Update command registry to use new implementation
6. Remove old command struct after migration is complete
7. Update tests to work with new implementation

## Acceptance Criteria
- [ ] New unified command struct created in src/repl/unified_commands/
- [ ] UnifiedCommand trait properly implemented
- [ ] Business logic moved from apply_command_event to the command itself
- [ ] Returns appropriate ViewEvent(s)
- [ ] Command registry updated
- [ ] Old command struct removed
- [ ] Tests updated and passing
- [ ] No regression in functionality

## Related Issues
- #226 (Architecture migration)
- #228 (Example migration)

## Labels
- enhancement
- refactoring
- unified-commands"

    gh issue create --title "$title" --body "$body" --label "enhancement,refactoring"
}

# Navigation Commands
create_issue "MoveCursorLeftCommand" "src/repl/commands/navigation.rs"
create_issue "MoveCursorRightCommand" "src/repl/commands/navigation.rs"
create_issue "MoveCursorUpCommand" "src/repl/commands/navigation.rs"
create_issue "MoveCursorDownCommand" "src/repl/commands/navigation.rs"
create_issue "ScrollLeftCommand" "src/repl/commands/navigation.rs"
create_issue "ScrollRightCommand" "src/repl/commands/navigation.rs"
create_issue "GoToTopCommand" "src/repl/commands/navigation.rs"
create_issue "GoToBottomCommand" "src/repl/commands/navigation.rs"
create_issue "NextWordCommand" "src/repl/commands/navigation.rs"
create_issue "PreviousWordCommand" "src/repl/commands/navigation.rs"
create_issue "EndOfWordCommand" "src/repl/commands/navigation.rs"
create_issue "BeginningOfLineCommand" "src/repl/commands/navigation.rs"
create_issue "EndOfLineCommand" "src/repl/commands/navigation.rs"
create_issue "HomeKeyCommand" "src/repl/commands/navigation.rs"
create_issue "EndKeyCommand" "src/repl/commands/navigation.rs"
create_issue "PageDownCommand" "src/repl/commands/navigation.rs"
create_issue "PageUpCommand" "src/repl/commands/navigation.rs"
create_issue "HalfPageDownCommand" "src/repl/commands/navigation.rs"
create_issue "HalfPageUpCommand" "src/repl/commands/navigation.rs"

# Editing Commands
create_issue "InsertCharCommand" "src/repl/commands/editing.rs"
create_issue "InsertNewLineCommand" "src/repl/commands/editing.rs"
create_issue "InsertTabCommand" "src/repl/commands/editing.rs"
create_issue "DeleteCharCommand" "src/repl/commands/editing.rs"
create_issue "DeleteCharAtCursorCommand" "src/repl/commands/editing.rs"

# Mode Commands
create_issue "EnterInsertModeCommand" "src/repl/commands/mode.rs"
create_issue "ExitInsertModeCommand" "src/repl/commands/mode.rs"
create_issue "ExitVisualBlockInsertModeCommand" "src/repl/commands/mode.rs"
create_issue "EnterVisualModeCommand" "src/repl/commands/mode.rs"
create_issue "ExitVisualModeCommand" "src/repl/commands/mode.rs"
create_issue "EnterVisualLineModeCommand" "src/repl/commands/mode.rs"
create_issue "EnterVisualBlockModeCommand" "src/repl/commands/mode.rs"
create_issue "EnterCommandModeCommand" "src/repl/commands/mode.rs"
create_issue "AppendAtEndOfLineCommand" "src/repl/commands/mode.rs"
create_issue "InsertAtBeginningOfLineCommand" "src/repl/commands/mode.rs"
create_issue "AppendAfterCursorCommand" "src/repl/commands/mode.rs"
create_issue "ExCommandModeCommand" "src/repl/commands/mode.rs"

# Pane Commands
create_issue "SwitchPaneCommand" "src/repl/commands/pane.rs"

# App Commands
create_issue "AppTerminateCommand" "src/repl/commands/app.rs"

# Ex Commands
create_issue "GoToLineCommand" "src/repl/commands/ex_commands.rs"
create_issue "QuitCommand" "src/repl/commands/ex_commands.rs"

echo "All migration issues created successfully!"
