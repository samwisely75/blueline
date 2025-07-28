# Claude Code Instructions

PLEASE READ `docs/DEVELOPER_GUIDE.md` BEFORE YOU DO ANYTHING.

## Notifications to Users

When a task is completed or you need to ask the user to confirm before executing a command, be sure to execute the following command to notify the user:

If you request confirmation, be sure to run the command before doing so.

```bash
# Try terminal-notifier first (if installed), fallback to osascript alerts
terminal-notifier -title "Claude Code" -message "<notification message for user>" -sound "CallBell" 2>/dev/null || osascript -e 'display alert "Claude Code" message "<notification message for user>"'
```

### Examples

#### Task Completion

```bash
terminal-notifier -title "Claude Code" -message "Task completed successfully!" -sound "CallBell" 2>/dev/null || osascript -e 'display alert "Claude Code" message "Task completed successfully!"'
```

#### Requesting Confirmation

```bash
terminal-notifier -title "Claude Code" -message "Please confirm the action" -sound "CallBell" 2>/dev/null || osascript -e 'display alert "Claude Code" message "Please confirm the action"'
```

#### Error Notifications

```bash
terminal-notifier -title "Claude Code" -message "Error occurred during execution" -sound "Basso" 2>/dev/null || osascript -e 'display alert "Claude Code" message "Error occurred during execution"'
```

### Notes

- This method relies on Claude Code's judgment to send notifications
- The notification will only work on macOS systems
- Tries terminal-notifier first (install with: brew install terminal-notifier)
- Falls back to osascript alerts if terminal-notifier is not available
- terminal-notifier provides better notification experience but requires installation
- osascript alerts are modal but work without additional dependencies
