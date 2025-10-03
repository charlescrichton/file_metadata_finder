#!/bin/bash
# Build script for cross-compiling to x86_64-unknown-linux-musl

set -e

echo "Building for x86_64-unknown-linux-musl..."

# Add the target if not already added
rustup target add x86_64-unknown-linux-musl

# Build in release mode
cargo build --release --target x86_64-unknown-linux-musl

echo "Build complete!"
echo "Binary location: target/x86_64-unknown-linux-musl/release/file_metadata_finder"
