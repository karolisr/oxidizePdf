#!/bin/bash

# Conventional commit helper
# Usage: ./scripts/commit-helper.sh

echo "🔧 Conventional Commit Helper"
echo "=============================="
echo ""

# Select commit type
echo "Select commit type:"
echo "1) feat     - New feature"
echo "2) fix      - Bug fix"
echo "3) docs     - Documentation only"
echo "4) style    - Code style changes"
echo "5) refactor - Code refactoring"
echo "6) perf     - Performance improvement"
echo "7) test     - Adding tests"
echo "8) chore    - Maintenance tasks"
echo "9) ci       - CI/CD changes"
echo ""

read -p "Enter choice (1-9): " choice

case $choice in
    1) TYPE="feat" ;;
    2) TYPE="fix" ;;
    3) TYPE="docs" ;;
    4) TYPE="style" ;;
    5) TYPE="refactor" ;;
    6) TYPE="perf" ;;
    7) TYPE="test" ;;
    8) TYPE="chore" ;;
    9) TYPE="ci" ;;
    *) echo "Invalid choice"; exit 1 ;;
esac

# Get scope (optional)
read -p "Enter scope (optional, e.g., parser, cli, api): " SCOPE

# Get description
read -p "Enter commit description: " DESCRIPTION

# Check for breaking change
read -p "Is this a breaking change? (y/N): " BREAKING

# Build commit message
if [ -n "$SCOPE" ]; then
    COMMIT_MSG="$TYPE($SCOPE): $DESCRIPTION"
else
    COMMIT_MSG="$TYPE: $DESCRIPTION"
fi

if [ "$BREAKING" = "y" ] || [ "$BREAKING" = "Y" ]; then
    COMMIT_MSG="$COMMIT_MSG

BREAKING CHANGE: "
    read -p "Describe the breaking change: " BREAKING_DESC
    COMMIT_MSG="$COMMIT_MSG$BREAKING_DESC"
fi

# Show the commit message
echo ""
echo "📝 Commit message:"
echo "=================="
echo "$COMMIT_MSG"
echo ""

# Confirm
read -p "Create this commit? (Y/n): " CONFIRM

if [ "$CONFIRM" != "n" ] && [ "$CONFIRM" != "N" ]; then
    git add -A
    git commit -m "$COMMIT_MSG"
    echo "✅ Commit created!"
else
    echo "❌ Commit cancelled"
fi