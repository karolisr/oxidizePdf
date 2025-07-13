#!/bin/bash
set -e

# Simple version bump script
# Usage: ./scripts/bump-version.sh [patch|minor|major]

VERSION_BUMP=${1:-patch}
CURRENT_VERSION=$(grep "^version" Cargo.toml | head -1 | cut -d'"' -f2)

echo "Current version: $CURRENT_VERSION"

# Parse current version
IFS='.' read -r -a version_parts <<< "$CURRENT_VERSION"
MAJOR="${version_parts[0]}"
MINOR="${version_parts[1]}"
PATCH="${version_parts[2]}"

# Calculate new version
case $VERSION_BUMP in
    patch)
        NEW_PATCH=$((PATCH + 1))
        NEW_VERSION="$MAJOR.$MINOR.$NEW_PATCH"
        ;;
    minor)
        NEW_MINOR=$((MINOR + 1))
        NEW_VERSION="$MAJOR.$NEW_MINOR.0"
        ;;
    major)
        NEW_MAJOR=$((MAJOR + 1))
        NEW_VERSION="$NEW_MAJOR.0.0"
        ;;
    *)
        echo "Invalid version bump type: $VERSION_BUMP"
        echo "Usage: $0 [patch|minor|major]"
        exit 1
        ;;
esac

echo "New version: $NEW_VERSION"

# Update workspace version
sed -i "s/version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" Cargo.toml

# Update Cargo.lock
cargo check > /dev/null 2>&1

echo "✅ Version bumped to $NEW_VERSION"
echo ""
echo "Next steps:"
echo "1. Update CHANGELOG.md with new changes"
echo "2. Commit changes: git commit -am \"chore: Bump version to $NEW_VERSION\""
echo "3. Create tag: git tag -a v$NEW_VERSION -m \"Release v$NEW_VERSION\""
echo "4. Push changes: git push && git push --tags"