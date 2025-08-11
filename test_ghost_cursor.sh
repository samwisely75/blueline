#!/bin/bash

# Test script to verify ghost cursor fix
# This script simulates rapid key presses that previously caused ghost cursors

echo "Testing ghost cursor fix for issue #107"
echo "Building the application..."
cargo build --release

echo ""
echo "Test scenarios:"
echo "1. Press h,j,k,l rapidly for navigation"
echo "2. Press w,b,e rapidly for word navigation"  
echo "3. Type characters rapidly in insert mode"
echo "4. Use Shift+Left/Right for horizontal scrolling"
echo "5. Expand/shrink visual selection rapidly with v and movement keys"
echo ""
echo "Starting blueline - please test the above scenarios..."
echo "Press Ctrl+C to exit when done testing"

./target/release/blueline