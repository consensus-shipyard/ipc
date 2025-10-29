#!/usr/bin/env bash
set -euo pipefail

# Fix Bottom-Up Checkpointing Error
# Disables bottom-up checkpointing for federated subnets

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/lib/colors.sh"

# Validator IPs
VALIDATORS=(
    "34.73.187.192"
    "35.237.175.224"
    "34.75.205.89"
)

log_header "Fixing Bottom-Up Checkpointing Error"
echo ""

log_info "This will disable bottom-up checkpointing on all validators"
log_info "Bottom-up checkpointing is not needed for federated subnets"
echo ""

log_warn "This will restart all validators!"
echo ""

read -p "Continue? (y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    log_info "Cancelled."
    exit 0
fi

echo ""
log_section "Updating Fendermint Configurations"
echo ""

for ip in "${VALIDATORS[@]}"; do
    log_info "Updating validator at $ip..."

    # Add bottomup.enabled = false to fendermint config
    ssh -o StrictHostKeyChecking=no philip@$ip "sudo su - ipc -c '
        cd ~/.ipc-node/fendermint/config

        # Backup original
        cp default.toml default.toml.before-bottomup-fix

        # Check if bottomup section already exists
        if grep -q \"\\[ipc.bottomup\\]\" default.toml; then
            echo \"  bottomup section exists, updating...\"
            # Update existing section
            sed -i \"/\\[ipc.bottomup\\]/,/^\\[/ s/^enabled = .*/enabled = false/\" default.toml
        else
            echo \"  Adding bottomup section...\"
            # Find the [ipc] section and add bottomup config after it
            # Insert after the last ipc.topdown line
            awk \"/\\[ipc.topdown\\]/{flag=1} flag && /^\\[/ && !/\\[ipc/{print \"\\n[ipc.bottomup]\\nenabled = false\\n\"; flag=0} 1\" default.toml > default.toml.tmp
            mv default.toml.tmp default.toml
        fi

        # Verify the change
        echo \"\"
        echo \"Verification:\"
        if grep -A1 \"\\[ipc.bottomup\\]\" default.toml | grep -q \"enabled = false\"; then
            echo \"  ✓ Bottom-up checkpointing disabled\"
        else
            echo \"  ✗ Failed to disable bottom-up checkpointing\"
            exit 1
        fi
    '" 2>/dev/null

    if [ $? -eq 0 ]; then
        log_success "✓ Config updated for $ip"
    else
        log_error "✗ Failed to update $ip"
        exit 1
    fi
done

echo ""
log_section "Restarting All Nodes"
echo ""

cd "$SCRIPT_DIR"
./ipc-manager restart --yes

echo ""
log_section "Fix Applied!"
echo ""

log_success "✓ Bottom-up checkpointing disabled on all validators"
echo ""

log_info "The error 'failed to broadcast checkpoint signature' should no longer appear"
echo ""

log_info "Monitor logs to verify:"
echo "  ssh philip@34.73.187.192 \"sudo su - ipc -c 'tail -f ~/.ipc-node/logs/*.log'\""
echo ""

log_info "To revert changes, restore from backups:"
echo "  default.toml.before-bottomup-fix"
echo ""

