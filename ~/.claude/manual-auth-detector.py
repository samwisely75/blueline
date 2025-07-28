#!/usr/bin/env python3
"""
Claude Code Manual Authorization Detector
Only sends notifications when manual authorization is required (not in auto-approval mode)
"""

import json
import sys
import os
import subprocess
from pathlib import Path

def send_notification(message, sound="CallBell"):
    """Send macOS notification"""
    try:
        # Try terminal-notifier first
        subprocess.run([
            "terminal-notifier", 
            "-title", "Claude Code", 
            "-message", message, 
            "-sound", sound
        ], check=True, capture_output=True)
    except (subprocess.CalledProcessError, FileNotFoundError):
        # Fallback to osascript
        subprocess.run([
            "osascript", 
            "-e", 
            f'display alert "Claude Code" message "{message}"'
        ], check=True, capture_output=True)

def is_auto_approval_mode():
    """Detect if Claude Code is in auto-approval mode"""
    
    # Method 1: Check for Shift+Tab auto-accept mode indicator
    # This would require checking Claude's internal state, which we can't access directly
    
    # Method 2: Check environment variables that might indicate auto-approval
    auto_env_vars = [
        'CLAUDE_AUTO_APPROVE',
        'CLAUDE_AUTO_ACCEPT', 
        'ANTHROPIC_AUTO_APPROVE'
    ]
    
    for var in auto_env_vars:
        if os.getenv(var, '').lower() in ['true', '1', 'yes', 'on']:
            return True
    
    # Method 3: Check if running in non-interactive mode
    if not sys.stdin.isatty():
        return True
        
    # Method 4: Check if we're in a CI environment
    ci_vars = ['CI', 'CONTINUOUS_INTEGRATION', 'GITHUB_ACTIONS', 'TRAVIS', 'CIRCLECI']
    if any(os.getenv(var) for var in ci_vars):
        return True
        
    # Method 5: Check if Claude Code was started with --dangerously-skip-permissions
    # This would be in the process arguments, but we can't access them directly
    
    return False

def main():
    """Main function to process hook input and decide whether to notify"""
    try:
        # Read hook input from stdin
        hook_data = json.load(sys.stdin)
        
        # Extract tool information
        tool_name = hook_data.get('tool_name', 'Unknown Tool')
        
        # Check if we're in auto-approval mode
        if is_auto_approval_mode():
            # In auto-approval mode, don't send notification
            sys.exit(0)
        
        # Check if this tool typically requires manual approval
        sensitive_tools = [
            'Bash', 'Edit', 'MultiEdit', 'Write', 'WebFetch', 
            'mcp__', 'NotebookEdit'  # MCP tools and notebook editing
        ]
        
        requires_approval = any(tool in tool_name for tool in sensitive_tools)
        
        if requires_approval:
            send_notification(f"Manual authorization required for: {tool_name}")
        
        sys.exit(0)
        
    except Exception as e:
        # If there's any error, fail silently to avoid breaking Claude Code
        sys.exit(0)

if __name__ == "__main__":
    main()