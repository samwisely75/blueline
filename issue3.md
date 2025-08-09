## Problem

Text append functionality is broken when double-byte characters are at the end of a horizontally scrolled line. Multiple related issues occur during text insertion and cursor navigation.

## Test Scenario Setup

- Request pane width: 112 bytes
- Line content: 55 double-byte characters (110 display columns)
- Starting position: REQUEST 1:1
- Command: Press `A` to append at end of line

## Expected Behavior After `A` Command

1. Screen scrolls to show the end of the line
2. Cursor positions at the right of the 55th character
3. New characters should be visible when typed
4. Cursor navigation should work correctly

## Current Behavior - Multiple Issues

### Problem 1: Invisible Character Insertion
**Steps to reproduce:**
1. After `A` command, cursor is positioned correctly at end of line
2. Type `a` (single-byte character)
3. **Bug**: Character is not displayed on screen despite having space for one character to the edge

### Problem 2: Inconsistent Display After Deletion/Reinsertion
**Steps to reproduce:**
1. From Problem 1 state (invisible `a`)
2. Press Backspace to delete `a`
3. Type `a` again
4. **Bug**: Now `a` shows up correctly
5. Type `b` - this displays correctly too

### Problem 3: Broken Cursor Navigation and Scrolling
**Steps to reproduce:**
1. From Problem 2 state (with visible `ab` at end)
2. Press `0` to go back to position 1:1
3. Press `A` again to append
4. **Bug**: Cursor goes to right of 55th char, in front of existing `ab` (should be after `ab`)
5. Press right arrow multiple times
6. **Bug**: Screen does not scroll right to show the `ab` characters
7. Press left arrow once
8. **Bug**: Screen suddenly scrolls right showing `ab`, cursor moves between `a` and `b`

## Impact

- Text editing is unreliable with double-byte characters and horizontal scrolling
- Cursor positioning is incorrect after mode transitions
- Display rendering is inconsistent
- Navigation behavior is unpredictable

## Root Causes (Suspected)

This appears to involve multiple interconnected issues:
1. Display rendering not accounting for character width during append
2. Cursor positioning logic inconsistent between display and logical positions
3. Horizontal scrolling logic not properly synchronized with cursor position
4. Mode transition (`A` command) not properly handling scrolled state

## Related Files

This likely involves:
- Text insertion/append logic
- Horizontal scrolling implementation
- Cursor positioning after mode changes
- Display rendering for multibyte characters
- Screen coordinate calculation

## Technical Notes

This is a complex interaction between multiple systems in the MVVM architecture:
- ViewModel cursor state management
- View rendering logic
- Event handling for text insertion
- Scroll position synchronization
