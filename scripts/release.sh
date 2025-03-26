#!/bin/bash

set -e

# Function to display usage
show_usage() {
  echo "Usage: $0 [major|minor|patch]"
  echo "  major: Bump the major version (x.0.0)"
  echo "  minor: Bump the minor version (0.x.0)"
  echo "  patch: Bump the patch version (0.0.x)"
}

# Check if an argument was provided
if [ $# -ne 1 ]; then
  show_usage
  exit 1
fi

# Validate the argument
case "$1" in
  major|minor|patch)
    VERSION_TYPE=$1
    ;;
  *)
    echo "Error: Invalid version type."
    show_usage
    exit 1
    ;;
esac

# Ensure there are no uncommitted changes
if ! git diff-index --quiet HEAD --; then
  echo "Error: You have uncommitted changes. Please commit or stash them first."
  exit 1
fi

# Install cargo-release if not already installed
if ! command -v cargo-release &> /dev/null; then
  echo "Installing cargo-release..."
  cargo install cargo-release
fi

# Get the current version before bumping
CURRENT_VERSION=$(grep '^version =' Cargo.toml | head -1 | sed 's/.*= "\(.*\)".*/\1/')
echo "Current version: $CURRENT_VERSION"

# Run cargo release
echo "Bumping $VERSION_TYPE version..."
cargo release $VERSION_TYPE --no-publish --execute --no-push

# Get the new version after bumping
NEW_VERSION=$(grep '^version =' Cargo.toml | head -1 | sed 's/.*= "\(.*\)".*/\1/')
echo "New version: $NEW_VERSION"

# Push the changes and tag
echo "Pushing changes and tags to GitHub..."
git push origin main --tags

echo "Release process initiated for v$NEW_VERSION"
echo "GitHub Actions will now build and upload the binaries."
echo "You can monitor the progress at: https://github.com/CJHwong/rs-git-msg/actions"
