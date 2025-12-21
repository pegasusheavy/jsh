#!/bin/bash
# Build all packages (Debian and RPM)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "🚀 Building all jsh packages..."
echo ""

# Build Debian package
echo "=========================================="
echo "Building Debian Package"
echo "=========================================="
"$SCRIPT_DIR/build-deb.sh"
echo ""

# Build RPM package
echo "=========================================="
echo "Building RPM Package"
echo "=========================================="
"$SCRIPT_DIR/build-rpm.sh"
echo ""

echo "=========================================="
echo "🎉 All packages built successfully!"
echo "=========================================="

