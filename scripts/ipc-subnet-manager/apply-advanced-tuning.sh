#!/usr/bin/env bash
set -euo pipefail

# Apply Advanced Performance Tuning to Existing Nodes
# This script updates CometBFT and Fendermint configs without reinitializing

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/lib/colors.sh"

# Validator IPs
VALIDATORS=(
    "34.73.187.192"
    "35.237.175.224"
    "34.75.205.89"
)

log_header "Advanced Performance Tuning"
echo ""

log_info "This will apply the following optimizations:"
echo "  • Ultra-fast consensus timeouts (propose: 500ms, prevote/precommit: 200ms)"
echo "  • Optimized timeout deltas for faster recovery"
echo "  • Enhanced P2P bandwidth (20MB/s send/recv)"
echo "  • Faster parent finality polling (5s instead of 10s)"
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
log_section "Updating CometBFT Configurations"
echo ""

for ip in "${VALIDATORS[@]}"; do
    log_info "Updating validator at $ip..."

    # Update consensus timeouts
    ssh -o StrictHostKeyChecking=no philip@$ip "sudo su - ipc -c '
        cd ~/.ipc-node/cometbft/config

        # Backup original
        cp config.toml config.toml.before-advanced-tuning

        # Update consensus timeouts
        sed -i \"s/^timeout_propose = .*/timeout_propose = \\\"500ms\\\"/\" config.toml
        sed -i \"s/^timeout_prevote = .*/timeout_prevote = \\\"200ms\\\"/\" config.toml
        sed -i \"s/^timeout_precommit = .*/timeout_precommit = \\\"200ms\\\"/\" config.toml

        # Update timeout deltas
        sed -i \"s/^timeout_propose_delta = .*/timeout_propose_delta = \\\"100ms\\\"/\" config.toml
        sed -i \"s/^timeout_prevote_delta = .*/timeout_prevote_delta = \\\"50ms\\\"/\" config.toml
        sed -i \"s/^timeout_precommit_delta = .*/timeout_precommit_delta = \\\"50ms\\\"/\" config.toml

        # Update empty blocks
        sed -i \"s/^create_empty_blocks_interval = .*/create_empty_blocks_interval = \\\"0s\\\"/\" config.toml

        # Update P2P rates
        sed -i \"s/^send_rate = .*/send_rate = 20971520/\" config.toml
        sed -i \"s/^recv_rate = .*/recv_rate = 20971520/\" config.toml
        sed -i \"s/^max_packet_msg_payload_size = .*/max_packet_msg_payload_size = 10240/\" config.toml

        # Verify critical changes
        echo \"\"
        echo \"Updated timeouts:\"
        grep \"^timeout_propose \\|^timeout_prevote \\|^timeout_precommit \\|^timeout_commit\" config.toml
    '" 2>/dev/null

    log_success "✓ CometBFT config updated for $ip"
done

echo ""
log_section "Updating Fendermint Configurations"
echo ""

for ip in "${VALIDATORS[@]}"; do
    log_info "Updating Fendermint on $ip..."

    # Update IPC settings
    ssh -o StrictHostKeyChecking=no philip@$ip "sudo su - ipc -c '
        cd ~/.ipc-node/fendermint/config

        # Backup original
        cp default.toml default.toml.before-advanced-tuning

        # Update IPC vote settings
        sed -i \"s/^vote_timeout = .*/vote_timeout = 30/\" default.toml

        # Update topdown settings
        sed -i \"s/^chain_head_delay = .*/chain_head_delay = 5/\" default.toml
        sed -i \"s/^proposal_delay = .*/proposal_delay = 5/\" default.toml
        sed -i \"s/^max_proposal_range = .*/max_proposal_range = 50/\" default.toml
        sed -i \"s/^polling_interval = .*/polling_interval = 5/\" default.toml
        sed -i \"s/^exponential_back_off = .*/exponential_back_off = 3/\" default.toml
        sed -i \"s/^exponential_retry_limit = .*/exponential_retry_limit = 3/\" default.toml
        sed -i \"s/^parent_http_timeout = .*/parent_http_timeout = 30/\" default.toml

        # Verify critical changes
        echo \"\"
        echo \"Updated IPC settings:\"
        grep \"^vote_timeout \\|^polling_interval \\|^chain_head_delay\" default.toml | head -3
    '" 2>/dev/null

    log_success "✓ Fendermint config updated for $ip"
done

echo ""
log_section "Restarting All Nodes"
echo ""

cd "$SCRIPT_DIR"
./ipc-manager restart --yes

echo ""
log_section "Advanced Tuning Applied!"
echo ""

log_success "✓ All validators updated with advanced performance tuning"
echo ""

log_info "Expected improvements:"
echo "  • Block time: 0.65s → 0.35-0.50s"
echo "  • Throughput: ~90 blocks/min → 120-180 blocks/min"
echo "  • Parent finality: every ~20 blocks → every ~10 blocks"
echo "  • Cross-msg latency: ~20s → ~10s"
echo ""

log_info "Monitor performance:"
echo "  ./ipc-manager watch-blocks     # Watch block production"
echo "  ./ipc-manager watch-finality   # Watch parent finality"
echo "  ./ipc-manager info             # Full health check"
echo ""

log_info "To revert changes, restore from backups:"
echo "  config.toml.before-advanced-tuning"
echo "  default.toml.before-advanced-tuning"
echo ""

