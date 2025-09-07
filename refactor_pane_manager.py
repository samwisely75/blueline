#!/usr/bin/env python3
"""
Script to refactor PaneManager and PaneState methods to remove PostCommandAction returns.
"""

import re
import os

def refactor_file(filepath):
    """Refactor a file to remove PostCommandAction returns."""
    with open(filepath, 'r') as f:
        content = f.read()
    
    original_content = content
    
    # Remove import if present
    content = re.sub(r'use crate::repl::view_models::PostCommandAction;\n', '', content)
    
    # Pattern to find methods returning Vec<PostCommandAction>
    method_pattern = r'(\s*pub fn \w+\([^)]*\)) -> Vec<PostCommandAction> \{'
    
    # Replace return type
    content = re.sub(method_pattern, r'\1 {', content)
    
    # Pattern to find method bodies that just delegate and return
    # Like: self.panes[self.current_pane].method_name(args)
    delegate_pattern = r'(\s+)(self\.panes\[self\.current_pane\]\.\w+\([^)]*\))\n'
    content = re.sub(delegate_pattern, r'\1\2;\n', content)
    
    # Similar for direct pane access
    delegate_pattern2 = r'(\s+)(self\.panes\[.*?\]\.\w+\([^)]*\))\n'
    content = re.sub(delegate_pattern2, r'\1\2;\n', content)
    
    # Handle methods that return vec![] or empty vectors
    content = re.sub(r'(\s+)vec!\[\]', r'\1// No events returned', content)
    content = re.sub(r'(\s+)Vec::new\(\)', r'\1// No events returned', content)
    
    # Handle methods that build and return vectors
    # This is more complex and needs manual review
    
    if content != original_content:
        with open(filepath, 'w') as f:
            f.write(content)
        return True
    return False

# Files to refactor
files = [
    '/Users/satoshi/Sources/samwisely75/rust/blueline/src/repl/models/app_state/pane_manager.rs',
    '/Users/satoshi/Sources/samwisely75/rust/blueline/src/repl/models/pane_state/cursor_basic.rs',
    '/Users/satoshi/Sources/samwisely75/rust/blueline/src/repl/models/pane_state/cursor_line.rs',
    '/Users/satoshi/Sources/samwisely75/rust/blueline/src/repl/models/pane_state/scrolling.rs',
    '/Users/satoshi/Sources/samwisely75/rust/blueline/src/repl/models/pane_state/word_navigation.rs',
    '/Users/satoshi/Sources/samwisely75/rust/blueline/src/repl/models/pane_state/text_operations.rs',
    '/Users/satoshi/Sources/samwisely75/rust/blueline/src/repl/models/pane_state/visual_selection.rs',
    '/Users/satoshi/Sources/samwisely75/rust/blueline/src/repl/models/pane_state/content.rs',
]

for filepath in files:
    if os.path.exists(filepath):
        if refactor_file(filepath):
            print(f"Refactored: {filepath}")
        else:
            print(f"No changes needed: {filepath}")
    else:
        print(f"File not found: {filepath}")