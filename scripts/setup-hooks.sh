#!/bin/bash
# Setup git hooks for jsh development
# Run this script after cloning the repository

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
HOOKS_DIR="$PROJECT_ROOT/.githooks"
GIT_HOOKS_DIR="$PROJECT_ROOT/.git/hooks"

echo "🔧 Setting up git hooks for jsh..."

# Check if we're in a git repository
if [ ! -d "$PROJECT_ROOT/.git" ]; then
    echo "❌ Error: Not a git repository. Run this script from the project root."
    exit 1
fi

# Check if commitlint is installed
if ! command -v commitlint &> /dev/null; then
    echo "📦 Installing commitlint-rs..."
    cargo install commitlint-rs
fi

# Create hooks directory if it doesn't exist
mkdir -p "$GIT_HOOKS_DIR"

# Copy hooks
echo "📋 Installing commit-msg hook..."
cp "$HOOKS_DIR/commit-msg" "$GIT_HOOKS_DIR/commit-msg"
chmod +x "$GIT_HOOKS_DIR/commit-msg"

echo ""
echo "✅ Git hooks installed successfully!"
echo ""
echo "The following hooks are now active:"
echo "  • commit-msg - Validates conventional commit messages"
echo ""
echo "Commit message format: <type>(<scope>): <description>"
echo ""
echo "Types: feat, fix, docs, style, refactor, perf, test, build, ci, chore, revert"
echo ""
echo "Examples:"
echo "  feat(parser): add match expression support"
echo "  fix(lexer): handle escaped quotes in strings"
echo "  docs: update README with Fish compatibility"

