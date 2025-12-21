#!/bin/bash
# Build RPM package using cargo-generate-rpm
# Requires: cargo install cargo-generate-rpm

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_DIR"

echo "🔨 Building jsh RPM package..."

# Check if cargo-generate-rpm is installed
if ! command -v cargo-generate-rpm &> /dev/null; then
    echo "❌ cargo-generate-rpm not found. Installing..."
    cargo install cargo-generate-rpm
fi

# Build the release binary first
echo "📦 Building release binary..."
cargo build --release

# Strip the binary for smaller package size
echo "🔧 Stripping binary..."
strip target/release/jsh 2>/dev/null || true

# Build the .rpm package
echo "📦 Creating RPM package..."
cargo generate-rpm

# Show the created package
RPM_FILE=$(ls -t target/generate-rpm/*.rpm 2>/dev/null | head -1)
if [ -n "$RPM_FILE" ]; then
    echo ""
    echo "✅ RPM package created successfully!"
    echo "📁 Package: $RPM_FILE"
    echo ""
    echo "To install (Fedora/RHEL): sudo dnf install $RPM_FILE"
    echo "To install (older):       sudo rpm -i $RPM_FILE"
    echo "To verify:                rpm -qip $RPM_FILE"
else
    echo "❌ Failed to create RPM package"
    exit 1
fi

