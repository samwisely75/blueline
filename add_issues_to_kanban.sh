#!/bin/bash

# Script to add remaining migration issues to blueline Kanban project
# and move them to Ready status

set -e  # Exit on any error

# Project details
PROJECT_NUMBER=1
PROJECT_ID="PVT_kwHODQVrPs4A_FfH"
OWNER="samwisely75"

# Status field and Ready option IDs from CLAUDE.md
STATUS_FIELD_ID="PVTSSF_lAHODQVrPs4A_FfHzgyS000"
READY_OPTION_ID="02b2059e"  # Updated to correct Ready option ID

# Function to add issue to project and move to Ready
add_and_move_issue() {
    local issue_number=$1
    local issue_url="https://github.com/samwisely75/blueline/issues/${issue_number}"
    
    echo "Processing issue #${issue_number}..."
    
    # Add issue to project
    echo "  Adding to project..."
    if ! gh project item-add $PROJECT_NUMBER --url "$issue_url" --owner "$OWNER"; then
        echo "  Failed to add issue #${issue_number} to project"
        return 1
    fi
    
    # Small delay to avoid rate limiting
    sleep 2
    
    # Get the item ID for this issue
    echo "  Getting item ID..."
    local item_id
    item_id=$(gh project item-list $PROJECT_NUMBER --owner "$OWNER" --format json --limit 1000 | \
              jq -r ".items[] | select(.content.number == $issue_number) | .id" 2>/dev/null || echo "")
    
    if [ -z "$item_id" ]; then
        echo "  Could not find item ID for issue #${issue_number}"
        return 1
    fi
    
    echo "  Item ID: $item_id"
    
    # Move to Ready status
    echo "  Moving to Ready status..."
    if gh project item-edit \
        --id "$item_id" \
        --field-id "$STATUS_FIELD_ID" \
        --single-select-option-id "$READY_OPTION_ID" \
        --project-id "$PROJECT_ID"; then
        echo "  ✅ Successfully processed issue #${issue_number}"
    else
        echo "  ⚠️  Added issue #${issue_number} to project but failed to set status"
    fi
    
    # Delay between issues to avoid rate limiting
    sleep 3
}

# Function to check if issue already exists in project
issue_exists_in_project() {
    local issue_number=$1
    gh project item-list $PROJECT_NUMBER --owner "$OWNER" --format json --limit 1000 | \
        jq -e ".items[] | select(.content.number == $issue_number)" >/dev/null 2>&1
}

echo "=== Adding remaining migration issues to blueline Kanban ==="
echo "Project: $PROJECT_NUMBER ($PROJECT_ID)"
echo "Owner: $OWNER"
echo ""

# Process issues #283-295 (the ones not yet in project)
failed_issues=()
for issue_num in {283..295}; do
    if issue_exists_in_project $issue_num; then
        echo "Issue #$issue_num already exists in project, skipping..."
        continue
    fi
    
    if ! add_and_move_issue $issue_num; then
        failed_issues+=($issue_num)
    fi
    
    echo ""
done

echo "=== Summary ==="
if [ ${#failed_issues[@]} -eq 0 ]; then
    echo "✅ All issues successfully added to Kanban and moved to Ready status!"
else
    echo "⚠️  Some issues failed to process:"
    for issue in "${failed_issues[@]}"; do
        echo "  - Issue #$issue"
    done
    echo ""
    echo "You may need to:"
    echo "  1. Wait for API rate limits to reset"
    echo "  2. Manually add/move these issues via the web interface"
    echo "  3. Re-run this script for failed issues only"
fi

echo ""
echo "Project URL: https://github.com/users/samwisely75/projects/1"
