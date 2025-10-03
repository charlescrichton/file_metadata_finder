#!/bin/bash
# Cross-compilation build script for Windows x86_64
# Run this on macOS or Linux to build a Windows .exe

set -e

echo "Cross-compiling for Windows x86_64..."

# Add the Windows target if not already added
rustup target add x86_64-pc-windows-gnu

# Install mingw-w64 if not available
if ! command -v x86_64-w64-mingw32-gcc &> /dev/null; then
    echo "Installing mingw-w64 for cross-compilation..."
    if command -v brew &> /dev/null; then
        echo "Installing via Homebrew..."
        brew install mingw-w64
    elif command -v apt &> /dev/null; then
        echo "Installing via apt..."
        sudo apt update && sudo apt install -y gcc-mingw-w64-x86-64
    elif command -v dnf &> /dev/null; then
        echo "Installing via dnf..."
        sudo dnf install -y mingw64-gcc
    elif command -v pacman &> /dev/null; then
        echo "Installing via pacman..."
        sudo pacman -S mingw-w64-gcc
    else
        echo "Error: Could not detect package manager. Please install mingw-w64 manually:"
        echo "  On macOS: brew install mingw-w64"
        echo "  On Ubuntu/Debian: sudo apt install gcc-mingw-w64-x86-64"
        echo "  On Fedora: sudo dnf install mingw64-gcc"
        echo "  On Arch: sudo pacman -S mingw-w64-gcc"
        exit 1
    fi
fi

# Build in release mode for Windows
echo "Building release binary..."
cargo build --release --target x86_64-pc-windows-gnu

echo ""
echo "✅ Cross-compilation complete!"
echo "📁 Windows executable: target/x86_64-pc-windows-gnu/release/file_metadata_finder.exe"
echo ""
echo "🚀 Usage on Windows (PowerShell 7 or Command Prompt):"
echo "   .\\file_metadata_finder.exe --directory \"C:\\Users\\YourName\\Documents\" --output \"scan_results.json\""
echo "   .\\file_metadata_finder.exe --directory \"D:\\MyFolder\" --output \"metadata.json\""
echo ""
echo "💡 The executable handles Windows paths correctly (C:\\, D:\\, etc.)"