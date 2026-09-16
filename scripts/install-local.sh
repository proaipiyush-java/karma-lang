#!/usr/bin/env sh
set -eu

PREFIX="${KARMA_PREFIX:-$HOME/.local}"
BIN_DIR="$PREFIX/bin"

echo "Building Karma v0.1 in release mode..."
cargo build --release

mkdir -p "$BIN_DIR"
cp target/release/karma "$BIN_DIR/karma"
chmod +x "$BIN_DIR/karma"

echo "Installed: $BIN_DIR/karma"
echo "If needed, add this to your shell PATH:"
echo "  export PATH=\"$BIN_DIR:\$PATH\""
