#!/bin/bash
set -e

# Release script for oxidize-pdf
# Usage: ./scripts/release.sh [patch|minor|major]

VERSION_BUMP=${1:-patch}

echo "🚀 Starting release process for $VERSION_BUMP version bump..."

# Check if we're on the correct branch
CURRENT_BRANCH=$(git branch --show-current)
if [ "$CURRENT_BRANCH" != "development" ]; then
    echo "❌ Error: Releases should be made from the 'development' branch"
    echo "   Current branch: $CURRENT_BRANCH"
    exit 1
fi

# Check if working directory is clean
if ! git diff-index --quiet HEAD --; then
    echo "❌ Error: Working directory is not clean"
    echo "   Please commit or stash your changes"
    exit 1
fi

# Check if cargo-release is installed
if ! command -v cargo-release &> /dev/null; then
    echo "📦 Installing cargo-release..."
    cargo install cargo-release
fi

# Run tests
echo "🧪 Running tests..."
cargo test --all

# Run clippy
echo "🔍 Running clippy..."
cargo clippy --all -- -D warnings

# Perform the release
echo "📋 Creating release for $VERSION_BUMP version..."
cargo release $VERSION_BUMP --execute

echo "✅ Release complete!"
echo ""
echo "Next steps:"
echo "1. Create a Pull Request from 'development' to 'main'"
echo "2. After merging, the package will be automatically published to crates.io"
echo "3. Create a GitHub release from the new tag"