#!/bin/bash

# Script to check and fix status of migration issues in the Kanban project

echo "=== Current Status of Migration Issues in Kanban ==="
echo ""

echo "Checking issues #256-295 in project..."
gh project item-list 1 --owner samwisely75 --format json --limit 1000 | \
  jq -r '.items[] | select(.content.number >= 256 and .content.number <= 295) | 
         "\(.content.number): \(.content.title) - Status: \(.status)"' | \
  sort -n

echo ""
echo "=== Summary ==="
echo "All issues #256-295 should be moved to 'Ready' status manually via:"
echo "https://github.com/users/samwisely75/projects/1"
echo ""
echo "The script encountered these issues:"
echo "1. Incorrect status option ID for 'Ready' - needs to be corrected"
echo "2. API rate limits when processing issues #290-295"
echo ""
echo "Issues that still need to be added to project (if any):"
echo "Check manually which issues from #290-295 are missing from the project"
