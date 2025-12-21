#!/bin/bash
# Build Debian package using cargo-deb
# Requires: cargo install cargo-deb

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_DIR"

echo "🔨 Building jsh Debian package..."

# Check if cargo-deb is installed
if ! command -v cargo-deb &> /dev/null; then
    echo "❌ cargo-deb not found. Installing..."
    cargo install cargo-deb
fi

# Build the release binary first
echo "📦 Building release binary..."
cargo build --release

# Build the .deb package
echo "📦 Creating Debian package..."
cargo deb

# Show the created package
DEB_FILE=$(ls -t target/debian/*.deb 2>/dev/null | head -1)
if [ -n "$DEB_FILE" ]; then
    echo ""
    echo "✅ Debian package created successfully!"
    echo "📁 Package: $DEB_FILE"
    echo ""
    echo "To install: sudo dpkg -i $DEB_FILE"
    echo "To verify:  dpkg -I $DEB_FILE"
else
    echo "❌ Failed to create Debian package"
    exit 1
fi

