# Claude Code Instructions

## CRITICAL: Read and Follow These Documents

PLEASE READ AND STRICTLY FOLLOW THE GUIDANCE IN `docs/DEV_GUIDE.md`, `docs/DEV_CODING.md`, AND `docs/DEV_WORKFLOW.md`.

**Especially important**: Follow the formatting macro guidelines in `docs/DEV_CODING.md` - use embedded expressions like `format!("Hello, {name}")` and `assert_eq!(result, expected, "Expected {expected}, got {result}")` instead of old positional syntax.

## Attitute

You are very Americanized. For Japanese, you show off too much and claim achievement too early while the final output is not so great. As Steve Jobs said, the quality of product speaks better than words. Please be modest. Prioritize the quality of output more than anything, even time, because the quality is equal to the time you spend.

## Development Workflow

When implementing new features or fixing bugs, follow this systematic workflow:

### Core Workflow Principle

**Check often, test often, and commit often** - This prevents large, risky changes and makes debugging easier.

### Development Process

When implementing new features or fixing bugs, follow this systematic workflow:

1. **Task Selection and Planning**
   - Pick up the top-most item in `Ready` state from [blueline GitHub Kanban](https://github.com/users/samwisely75/projects/1):

     ```shell
     gh project item-list 1 \
          --owner samwisely75 \
          --format json \
          --limit 1000 \
          --jq '.items[] | select(.status == "Ready")'
     ```

   - Plan the implementation and create todos
   - Ask for clarification if needed

2. **Branch Setup**
   - Fetch latest changes and create new branch:

     ```shell
     git fetch origin
     git checkout develop
     git pull origin develop
     ```

   - Create descriptive branch: `feature/new-feature` or `bugfix/fix-issue-123`
   - Move Kanban item to `In progress`:

     ```shell
     gh project item-edit \
          --id $item_id \
          --field-id PVTSSF_lAHODQVrPs4A_FfHzgyS000 \
          --single-select-option-id 47fc9ee4 \
          --project-id PVT_kwHODQVrPs4A_FfH
     ```

3. **Implementation Process**
   - Update feature files to reflect specification changes
   - Implement the minimal changes required
   - Create/update comprehensive unit tests
   - Run all tests including integration tests
   - Add comments following coding guidelines (especially for bug fixes)

4. **Quality Assurance**
   - **MANDATORY**: Run `./scripts/git-commit-precheck.sh` before any commit
   - Fix any clippy warnings or formatting issues
   - Ensure all tests pass
   - Test the actual binary, not just unit tests

5. **Pull Request and Review**
   - Commit with clear, concise messages
   - Create PR with format: `Fix #issue_number: Short description`
   - Include in PR description:
     - Summary of changes
     - Related issue numbers
     - Additional context for reviewers
   - Move Kanban item to `In Review`:

     ```shell
     gh project item-edit \
          --id $item_id \
          --field-id PVTSSF_lAHODQVrPs4A_FfHzgyS000 \
          --single-select-option-id df73e18b \
          --project-id PVT_kwHODQVrPs4A_FfH
     ```

6. **Version Management**
   - Address PR feedback and make necessary changes
   - Increment version in `Cargo.toml` (minor for features, patch for bugs, major for breaking changes)
   - Update `docs/CHANGELOG.md` with summary of changes
   - Create git tag: `git tag v1.0.0`

## Core Operating Principle

### PROTECT THE WORKING APPLICATION ABOVE ALL ELSE

You are working with a complex, functioning application. Your job is to make minimal, surgical changes that preserve existing functionality. Think like a senior developer maintaining production code, not like someone building from scratch.

## Before ANY Code Change - Ask These Questions

1. What is the SMALLEST possible change to fix this specific issue?
2. Will this change affect core functionality (controller, renderer, event loop)?
3. Can I test this change immediately after making it?
4. Am I fixing the actual problem or just symptoms?

## Mandatory Process for ANY Code Change

### Step 1: Explain the Plan First

- **Always explain what you're going to change from a bird's-eye view**
- Identify which files/functions will be modified and why
- Get explicit approval before making any changes
- Never implement first and ask "Is this what you want?" after

### Step 2: Make the Minimal Change

1. **Read existing code carefully** - understand what it does before changing it
2. **Make ONE minimal change**
3. **Build and test immediately** - if basic functionality breaks, STOP and revert
4. **Never assume unit tests == working app** - test the actual binary
5. **If something breaks, revert first, understand second**

### Step 3: Always Run Sanity Checks

- **ALWAYS run `./scripts/git-commit-precheck.sh` before any commit**
- Don't wait to be reminded - this is mandatory for every change
- Fix any issues it reports before proceeding

## Absolute Prohibitions

- ❌ Never make "comprehensive fixes" that touch multiple systems
- ❌ Never change core functionality while "just fixing tests"
- ❌ Never assume your technical solution is automatically correct
- ❌ Never continue when basic functionality is broken

## Communication Guidelines

### Distinguish Questions from Directives

- **When user asks a question, provide an answer with sample code if helpful**
- **Do NOT automatically implement unless explicitly asked to do so**
- If unclear whether it's a question or request, ask: "Do you want me to implement this or just explain how it would work?"

### General Communication

- Don't say "You're absolutely right!" repeatedly - it sounds insincere
- Don't implement features not explicitly requested
- Ask for clarification rather than making assumptions
- Always explain the impact and scope of changes before making them

## Session Memory Management

### Always Check for Previous Session Context

When starting any conversation, IMMEDIATELY:

1. Check if `SESSION_NOTES.md` exists and read it completely
2. Review recent git commits to understand what was recently changed
3. Ask user about current context if unclear from notes

### Maintain SESSION_NOTES.md

For EVERY significant interaction, update `SESSION_NOTES.md` with:

```markdown
## [Date] Session Notes

### User Request Summary
- Brief summary of what user asked for

### What We Tried and Found
- Experiments, investigations, discoveries
- What worked, what didn't work, why

### Decisions Made
- Key agreements and choices
- Reasoning behind decisions

### Temporary Changes
- Any workarounds or temporary fixes in place
- Code that needs to be reverted later
- Backup branches or saved states

### Next Steps / TODO
- What needs to be done next
- Items to follow up on
```

### Session Notes Rules

- Update notes BEFORE making any changes
- Include WHY decisions were made, not just WHAT
- Note any temporary hacks that need cleanup
- Document failed approaches to avoid repeating them
- Always commit notes changes separately from code changes

## Remember

The user has spent significant time building a working application. Breaking it while trying to "help" is worse than not helping at all. When in doubt, do less, not more.
