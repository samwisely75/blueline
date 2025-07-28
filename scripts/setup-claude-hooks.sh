#!/bin/bash

# Claude Code Hooks Setup Script
# Automatically configures notification hooks for Claude Code

set -e

echo "🔧 Claude Code Hooks Setup Script"
echo "================================="
echo ""

# Check if Claude Code is installed
if ! command -v claude &> /dev/null; then
    echo "❌ Claude Code not found. Please install it first."
    exit 1
fi

# Check for terminal-notifier
NOTIFIER_CMD=""
if command -v terminal-notifier &> /dev/null; then
    echo "✅ terminal-notifier found"
    NOTIFIER_CMD="terminal-notifier"
else
    echo "⚠️  terminal-notifier not found. Using osascript as fallback."
    echo "   For better notifications, install with: brew install terminal-notifier"
    NOTIFIER_CMD="osascript"
fi

# Test notification system
echo ""
echo "🧪 Testing notification system..."
if [ "$NOTIFIER_CMD" = "terminal-notifier" ]; then
    terminal-notifier -title "Claude Code" -message "Notification test successful!" -sound "Glass"
else
    osascript -e 'display alert "Claude Code" message "Notification test successful!"'
fi

if [ $? -ne 0 ]; then
    echo "❌ Notification test failed. Please check your macOS notification permissions."
    exit 1
fi

echo "✅ Notification test passed!"

# Configure hooks
echo ""
echo "📝 Configuring Claude Code hooks..."

# Stop hook - Notifies when Claude completes a task
echo "Setting up Stop hook..."
claude config set --global hooks.Stop[0].matcher ""
claude config set --global hooks.Stop[0].hooks[0].type "command"
claude config set --global hooks.Stop[0].hooks[0].command "terminal-notifier -title \"Claude Code\" -message \"Task completed!\" -sound \"Glass\" 2>/dev/null || osascript -e 'display alert \"Claude Code\" message \"Task completed!\"'"

# Notification hook - Handles custom notifications from Claude
echo "Setting up Notification hook..."
claude config set --global hooks.Notification[0].matcher ""
claude config set --global hooks.Notification[0].hooks[0].type "command"
claude config set --global hooks.Notification[0].hooks[0].command "cat | jq -r '\"\\\"\" + .title + \": \" + .message + \"\\\"\"' | xargs -I {} sh -c 'terminal-notifier -title \"Claude Code\" -message {} -sound \"Glass\" 2>/dev/null || osascript -e \"display alert \\\"Claude Code\\\" message {}\"'"

# PreToolUse hook - Notifies before tool execution
echo "Setting up PreToolUse hook..."
claude config set --global hooks.PreToolUse[0].matcher ""
claude config set --global hooks.PreToolUse[0].hooks[0].type "command"
claude config set --global hooks.PreToolUse[0].hooks[0].command "terminal-notifier -title \"Claude Code\" -message \"About to execute tool\" -sound \"Tink\" 2>/dev/null || osascript -e 'display alert \"Claude Code\" message \"About to execute tool\"'"

# PostToolUse hook - Notifies after tool execution
echo "Setting up PostToolUse hook..."
claude config set --global hooks.PostToolUse[0].matcher ""
claude config set --global hooks.PostToolUse[0].hooks[0].type "command"
claude config set --global hooks.PostToolUse[0].hooks[0].command "terminal-notifier -title \"Claude Code\" -message \"Tool execution completed\" -sound \"Tink\" 2>/dev/null || osascript -e 'display alert \"Claude Code\" message \"Tool execution completed\"'"

echo ""
echo "✅ Hooks configured successfully!"

# Display current configuration
echo ""
echo "📋 Current Claude Code hooks configuration:"
claude config list --global | grep -A 20 "hooks" || echo "Unable to display hooks configuration"

echo ""
echo "🎉 Setup complete!"
echo ""
echo "📌 What's been configured:"
echo "  • Stop hook: Notifies when Claude completes a task"
echo "  • Notification hook: Shows custom messages from Claude"
echo "  • PreToolUse hook: Alerts before tool execution"
echo "  • PostToolUse hook: Alerts after tool execution"
echo ""
echo "💡 Tips:"
echo "  • Restart Claude Code for changes to take effect"
echo "  • Install terminal-notifier for better notifications: brew install terminal-notifier"
echo "  • Check ~/.claude.json to verify or customize the configuration"
echo ""
echo "📚 For more information:"
echo "  • Hooks documentation: https://docs.anthropic.com/en/docs/claude-code/hooks"