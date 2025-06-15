#!/bin/bash

# Install ratatoskr as a macOS LaunchAgent

set -e

echo "Installing ratatoskr as macOS LaunchAgent..."

# Install the binary
cargo install --path .

# Create necessary directories
mkdir -p /usr/local/var/ratatoskr
mkdir -p /usr/local/var/log

# Copy plist file to LaunchAgents directory
cp scripts/com.sectorflabs.ratatoskr.plist ~/Library/LaunchAgents/

echo "Service installed successfully!"
echo "Use the following commands to manage the service:"
echo "  launchctl load ~/Library/LaunchAgents/com.sectorflabs.ratatoskr.plist    # Start the service"
echo "  launchctl unload ~/Library/LaunchAgents/com.sectorflabs.ratatoskr.plist  # Stop the service"
echo "  launchctl list | grep ratatoskr                                          # Check service status"
echo "  tail -f /usr/local/var/log/ratatoskr.log                                 # View logs" 