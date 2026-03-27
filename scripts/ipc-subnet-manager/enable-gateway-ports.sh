#!/bin/bash
# Enable GatewayPorts on remote VMs to allow SSH reverse tunneling
# This may be needed if the tunnels can't be established

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Validator info (from config)
VALIDATORS=(
    "philip@34.73.187.192"
    "philip@35.237.175.224"
    "philip@34.75.205.89"
)

echo -e "${GREEN}Checking/enabling GatewayPorts on remote VMs...${NC}"
echo ""

for validator in "${VALIDATORS[@]}"; do
    echo -e "${YELLOW}Configuring: ${validator}${NC}"

    # Check if GatewayPorts is enabled
    ssh "${validator}" "sudo grep -q '^GatewayPorts' /etc/ssh/sshd_config || echo 'Not configured'"

    # Enable GatewayPorts
    ssh "${validator}" "sudo sh -c 'echo \"GatewayPorts yes\" >> /etc/ssh/sshd_config' && sudo systemctl restart sshd"

    echo -e "  ${GREEN}✓${NC} GatewayPorts enabled and SSH restarted"
    echo ""
done

echo -e "${GREEN}All VMs configured!${NC}"

