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

# Require TELEGRAM_BOT_TOKEN in the current environment
if [ -z "${TELEGRAM_BOT_TOKEN:-}" ]; then
    echo "Error: TELEGRAM_BOT_TOKEN is not set in the environment." >&2
    exit 1
fi

# Create service file from template
sed -e "s|__USER__|$CURRENT_USER|g" \
    -e "s|__HOME__|$HOME_DIR|g" \
    -e "s|__REMOTE_PATH__|$CURRENT_DIR|g" \
    -e "s|__TELEGRAM_BOT_TOKEN__|$TELEGRAM_BOT_TOKEN|g" \
    scripts/ratatoskr.service.template > /tmp/ratatoskr.service

# Copy service file to systemd directory
sudo cp /tmp/ratatoskr.service /etc/systemd/system/

# Clean up temporary file
rm /tmp/ratatoskr.service

# Reload systemd daemon
sudo systemctl daemon-reload

# Enable and start the service
sudo systemctl enable ratatoskr
sudo systemctl start ratatoskr

echo "Service installed successfully!"
echo "Binary location: $HOME_DIR/.cargo/bin/ratatoskr"
echo "Working directory: $CURRENT_DIR"
echo ""
echo "Use the following commands to manage the service:"
echo "  sudo systemctl start ratatoskr    # Start the service"
echo "  sudo systemctl stop ratatoskr     # Stop the service"
echo "  sudo systemctl status ratatoskr   # Check service status"
echo "  sudo systemctl restart ratatoskr  # Restart the service"
echo "  journalctl -u ratatoskr -f        # View logs" 