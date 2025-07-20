#!/bin/sh
#
# Install Git hooks for development
# Run this script to set up pre-commit hooks that enforce code quality
#

set -e

echo "🔧 Installing Git hooks..."

# Make the hook executable and copy it to .git/hooks/
chmod +x scripts/hooks/pre-commit
cp scripts/hooks/pre-commit .git/hooks/pre-commit

echo "✅ Pre-commit hook installed successfully!"
echo ""
echo "The pre-commit hook will now:"
echo "  • Run 'cargo clippy --all-targets --all-features -- -D warnings'"
echo "  • Reject commits if any clippy warnings are found"
echo "  • Ensure modern format string syntax is used"
echo ""
echo "To bypass the hook in emergencies, use: git commit --no-verify"
