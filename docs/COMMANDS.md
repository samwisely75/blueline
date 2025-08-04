# Commands Reference

This document lists all supported commands in the blueline HTTP client application.

## Navigation and Movement

### Basic Movement (Normal Mode)

- `h` or `←` - Move cursor left
- `j` or `↓` - Move cursor down  
- `k` or `↑` - Move cursor up
- `l` or `→` - Move cursor right

### Arrow Keys (All Modes)

- `↑` `↓` `←` `→` - Move cursor in any direction, works in all modes

### Horizontal Scrolling

- `Shift + ←` or `Ctrl + ←` - Scroll left by 5 characters
- `Shift + →` or `Ctrl + →` - Scroll right by 5 characters

### Pane Navigation

- `Tab` - Switch between request and response panes

## Editor Modes

### Normal Mode

The default mode for navigation and command entry.

### Insert Mode

Mode for editing text in the request buffer.

#### Entering Insert Mode (Normal Mode → Insert Mode)

- `i` - Enter insert mode at cursor position
- `a` - Enter insert mode after cursor position  
- `A` or `Shift + A` - Enter insert mode at end of current line
- `I` or `Shift + I` - Enter insert mode at beginning of current line

#### Exiting Insert Mode (Insert Mode → Normal Mode)

- `Esc` - Exit insert mode and return to normal mode

### Command Mode

Mode for executing ex commands (vim-style commands).

#### Entering Command Mode (Normal Mode → Command Mode)

- `:` - Enter command mode

#### In Command Mode

- Type any character to add to command buffer
- `Backspace` - Remove last character from command buffer
- `Enter` - Execute the command
- `Esc` - Cancel command and return to normal mode

## Text Editing (Insert Mode)

### Text Input

- Any printable character or space - Insert character at cursor position
- `Enter` - Insert new line

### Text Deletion

- `Backspace` - Delete character before cursor
- `Delete` - Delete character at cursor position

## HTTP Request Operations

### Execute Request (Normal Mode)

- `Enter` - Execute the HTTP request in the request pane

## Ex Commands (Command Mode)

Enter command mode with `:` then type one of the following:

### Application Control

- `:q` - Quit the application
- `:q!` - Force quit the application (same as `:q`)

### Display Settings  

- `:set wrap` - Enable word wrap in both request and response panes
- `:set nowrap` - Disable word wrap in both request and response panes

## Application Control

### Force Quit

- `Ctrl + C` - Immediately terminate the application

## Cursor Behavior

The cursor shape changes based on the current mode:

- **Normal Mode**: Block cursor
- **Insert Mode**: Blinking bar cursor  
- **Command Mode**: Cursor is hidden

## Pane System

The application has two main panes:

- **Request Pane**: Where you write HTTP requests
- **Response Pane**: Where HTTP responses are displayed (appears after executing a request)

## Status Bar

The status bar displays:

- HTTP response status (when available): colored indicator, status code, message, and response time
- Current mode: NORMAL, INSERT, or COMMAND
- Current pane: REQUEST or RESPONSE  
- Cursor position: line:column

## Line Numbers

- Line numbers are displayed with a minimum width of 3 characters
- Empty lines beyond content show a tilde (`~`) in the first column
- In the request pane, line number 1 is always shown even when the buffer is empty

## Word Wrap

When word wrap is enabled (`:set wrap`):

- Long lines are visually wrapped to fit the terminal width
- Navigation commands work with the wrapped display
- Line numbers only appear on the first line of wrapped content
- Continuation lines show blank space in the line number area

When word wrap is disabled (`:set nowrap`):

- Long lines extend beyond the visible area
- Use horizontal scrolling to view content beyond the terminal width
