#!/bin/bash

# Quick Anvil Launcher for IPC Development
# One-command setup for local development with IPC keystore

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}🚀 Quick Anvil Launcher${NC}"
echo ""

# Check if IPC keystore exists
if [ ! -f "$HOME/.ipc/evm_keystore.json" ]; then
    echo "⚠️  No IPC keystore found. Creating sample accounts..."

    # Create .ipc directory if it doesn't exist
    mkdir -p "$HOME/.ipc"

    # Ask if user wants to import keys or use defaults
    read -p "Do you want to import an existing private key? (y/N): " -n 1 -r
    echo

    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo "Use 'ipc-cli wallet import --keystore-path ~/.ipc' to import your keys"
        echo "Then run this script again."
        exit 0
    else
        echo "Continuing with default setup..."
    fi
fi

# Run the main setup script
exec "$SCRIPT_DIR/setup-anvil-with-ipc-keys.sh"

