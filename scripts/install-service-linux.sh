#!/bin/bash

# Install ratatoskr as a systemd service on Linux

set -e

echo "Installing ratatoskr as systemd service on Linux..."

# Install the binary
cargo install --path .

# Get current user and home directory
CURRENT_USER=$(whoami)
HOME_DIR=$(eval echo ~$CURRENT_USER)
CURRENT_DIR=$(pwd)

echo "Configuring service for user: $CURRENT_USER"
echo "Home directory: $HOME_DIR"
echo "Working directory: $CURRENT_DIR"

# Create service file from template
sed -e "s|__USER__|$CURRENT_USER|g" \
    -e "s|__HOME__|$HOME_DIR|g" \
    -e "s|__REMOTE_PATH__|$CURRENT_DIR|g" \
    scripts/ratatoskr.service.template > /tmp/ratatoskr.service

# Copy service file to systemd directory
sudo cp /tmp/ratatoskr.service /etc/systemd/system/

# Clean up temporary file
rm /tmp/ratatoskr.service

# Reload systemd daemon
sudo systemctl daemon-reload

# Enable the service to start on boot
sudo systemctl enable ratatoskr

echo "Service installed successfully!"
echo "Binary location: $HOME_DIR/.cargo/bin/ratatoskr"
echo "Working directory: $CURRENT_DIR"
echo "Environment file: $CURRENT_DIR/.env"
echo ""
echo "Use the following commands to manage the service:"
echo "  sudo systemctl start ratatoskr    # Start the service"
echo "  sudo systemctl stop ratatoskr     # Stop the service"
echo "  sudo systemctl status ratatoskr   # Check service status"
echo "  sudo systemctl restart ratatoskr  # Restart the service"
echo "  journalctl -u ratatoskr -f        # View logs" 