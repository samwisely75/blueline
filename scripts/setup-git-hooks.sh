#!/bin/sh
#
# Install Git hooks for development
# Run this script to set up pre-commit hooks that enforce code quality
#

set -e

echo "🔧 Installing Git hooks..."

# Configure Git to use .githooks directory (tracked in repository)
git config core.hooksPath .githooks

echo "✅ Git hooks configured successfully!"
echo ""
echo "The pre-commit hook will now:"
echo "  • Run 'cargo clippy --all-targets --all-features -- -D warnings'"
echo "  • Reject commits if any clippy warnings are found"
echo "  • Ensure modern format string syntax is used"
echo ""
echo "💡 The hooks are now tracked in the repository at .githooks/"
echo "💡 To bypass the hook in emergencies, use: git commit --no-verify"
