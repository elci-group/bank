#!/bin/bash
# Bank Installation Script
# This script builds and installs the bank utility

set -e

echo "🏦 Installing Bank utility..."
echo

# Check if we're in the right directory
if [[ ! -f "Cargo.toml" ]] || [[ ! -f "src/main.rs" ]]; then
    echo "❌ Error: Please run this script from the bank directory"
    exit 1
fi

# Check if cargo is available
if ! command -v cargo &> /dev/null; then
    echo "❌ Error: Cargo not found. Please install Rust first."
    echo "   Visit: https://rustup.rs/"
    exit 1
fi

# Build the utility
echo "🔨 Building bank utility..."
cd .. # Go to workspace root
cargo build --release -p bank
cd bank

# Check if build was successful
if [[ ! -f "../target/release/bank" ]]; then
    echo "❌ Error: Build failed"
    exit 1
fi

echo "✅ Build successful!"
echo

# Optional: Install to system PATH
read -p "📦 Install bank to /usr/local/bin? (y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    if command -v sudo &> /dev/null; then
        sudo cp "../target/release/bank" /usr/local/bin/
        echo "✅ Bank installed to /usr/local/bin/bank"
    else
        echo "❌ Error: sudo not available. Please manually copy '../target/release/bank' to your PATH"
        exit 1
    fi
else
    echo "ℹ️  Bank binary is available at: $(pwd)/../target/release/bank"
    echo "   You can add this to your PATH or create a symlink"
fi

echo
echo "🎉 Installation complete!"
echo "   Run 'bank --help' to get started"