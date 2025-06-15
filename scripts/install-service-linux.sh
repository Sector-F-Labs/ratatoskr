#!/bin/bash

# Install ratatoskr as a systemd service on Linux

set -e

echo "Installing ratatoskr as systemd service on Linux..."

# Install the binary
cargo install --path .

# Copy service file to systemd directory
sudo cp scripts/ratatoskr.service /etc/systemd/system/

# Reload systemd daemon
sudo systemctl daemon-reload

# Enable the service to start on boot
sudo systemctl enable ratatoskr

echo "Service installed successfully!"
echo "Use the following commands to manage the service:"
echo "  sudo systemctl start ratatoskr    # Start the service"
echo "  sudo systemctl stop ratatoskr     # Stop the service"
echo "  sudo systemctl status ratatoskr   # Check service status"
echo "  sudo systemctl restart ratatoskr  # Restart the service"
echo "  journalctl -u ratatoskr -f        # View logs" 