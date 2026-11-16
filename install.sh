#!/bin/bash
set -e

echo "Building Obsidian SSH release..."
cargo build --release

echo "Installing binaries..."
sudo cp target/release/obsidianssh /usr/local/bin/
sudo cp target/release/obsidianssh-mcp /usr/local/bin/

echo "Installing icon..."
sudo mkdir -p /usr/share/icons/hicolor/64x64/apps
sudo cp resources/logo_64.png /usr/share/icons/hicolor/64x64/apps/obsidianssh.png

echo "Installing desktop entry..."
sudo cp resources/obsidianssh.desktop /usr/share/applications/

echo "Done! Run 'obsidianssh' to start."
