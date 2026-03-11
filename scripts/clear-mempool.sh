#!/bin/bash

# Clear Stuck Mempool Transactions
# This script helps diagnose and clear stuck transactions in the IPC subnet mempool
#
# Usage: ./clear-mempool.sh [VALIDATOR_IP] [SSH_USER]
#   VALIDATOR_IP: IP address of the validator node (default: prompts user)
#   SSH_USER: SSH username for the validator (default: current user)

set -e

# Accept parameters or use defaults
VALIDATOR_IP="${1:-}"
SSH_USER="${2:-$USER}"

# Prompt for IP if not provided
if [ -z "$VALIDATOR_IP" ]; then
    read -p "Enter validator IP address: " VALIDATOR_IP
    if [ -z "$VALIDATOR_IP" ]; then
        echo "Error: Validator IP is required"
        exit 1
    fi
fi

echo "🔍 Analyzing stuck mempool transactions..."
echo "   Validator: $SSH_USER@$VALIDATOR_IP"
echo ""

# Check mempool status
echo "📊 Mempool Status:"
MEMPOOL=$(ssh -o ConnectTimeout=5 -o StrictHostKeyChecking=no "$SSH_USER@$VALIDATOR_IP" \
    "curl -s http://localhost:26657/num_unconfirmed_txs" 2>/dev/null)

N_TXS=$(echo "$MEMPOOL" | jq -r '.result.n_txs')
TOTAL_BYTES=$(echo "$MEMPOOL" | jq -r '.result.total_bytes')

echo "  Stuck transactions: $N_TXS"
echo "  Total bytes: $TOTAL_BYTES"
echo ""

if [ "$N_TXS" = "0" ]; then
    echo "✅ No stuck transactions!"
    exit 0
fi

# Check if subnet is producing blocks
echo "📦 Block Production:"
STATUS=$(ssh -o ConnectTimeout=5 -o StrictHostKeyChecking=no "$SSH_USER@$VALIDATOR_IP" \
    "curl -s http://localhost:26657/status" 2>/dev/null)

HEIGHT=$(echo "$STATUS" | jq -r '.result.sync_info.latest_block_height')
echo "  Current height: $HEIGHT"
echo ""

# Wait and check if blocks are still being produced
echo "⏳ Waiting 10 seconds to check block production..."
sleep 10

STATUS2=$(ssh -o ConnectTimeout=5 -o StrictHostKeyChecking=no "$SSH_USER@$VALIDATOR_IP" \
    "curl -s http://localhost:26657/status" 2>/dev/null)
HEIGHT2=$(echo "$STATUS2" | jq -r '.result.sync_info.latest_block_height')

BLOCKS_PRODUCED=$((HEIGHT2 - HEIGHT))
echo "  Blocks produced: $BLOCKS_PRODUCED"
echo ""

if [ "$BLOCKS_PRODUCED" -lt 1 ]; then
    echo "❌ WARNING: Subnet is not producing blocks!"
    echo "   The mempool transactions cannot be cleared until block production resumes."
    echo ""
    echo "💡 Solution: Restart the subnet nodes"
    echo "   Run: cd scripts/ipc-subnet-manager && ./ipc-manager restart"
    exit 1
fi

echo "✅ Subnet is producing blocks"
echo ""

# Solutions
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "💡 Solutions to Clear Stuck Transactions"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo "Option 1: Wait for automatic processing (Recommended)"
echo "  - Cross-chain messages may need parent chain confirmations"
echo "  - Wait 10-20 minutes and check again"
echo ""

echo "Option 2: Flush the mempool (Nuclear option)"
echo "  - This will clear ALL pending transactions"
echo "  - You'll need to resubmit any valid transactions"
echo "  - Command:"
echo "    ssh $SSH_USER@$VALIDATOR_IP 'sudo systemctl stop cometbft && rm -rf ~/.cometbft/data/mempool.wal && sudo systemctl start cometbft'"
echo ""

echo "Option 3: Restart the subnet"
echo "  - Use the subnet manager (if available):"
# Get script directory dynamically
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
if [ -d "$SCRIPT_DIR/ipc-subnet-manager" ]; then
    echo "    cd $SCRIPT_DIR/ipc-subnet-manager"
    echo "    ./ipc-manager restart"
else
    echo "    (ipc-subnet-manager not found in $SCRIPT_DIR)"
fi
echo ""

echo "Option 4: Check transaction validity"
echo "  - These may be invalid cross-chain messages"
echo "  - Check parent chain for failed fund() calls"
echo "  - Verify you have sufficient balance on L1"
echo ""

# Offer to clear automatically
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
read -p "Do you want to flush the mempool now? (yes/no): " answer

if [ "$answer" = "yes" ]; then
    echo ""
    echo "🧹 Flushing mempool..."

    ssh -o ConnectTimeout=5 -o StrictHostKeyChecking=no "$SSH_USER@$VALIDATOR_IP" \
        "sudo systemctl stop cometbft" 2>/dev/null || true

    sleep 2

    ssh -o ConnectTimeout=5 -o StrictHostKeyChecking=no "$SSH_USER@$VALIDATOR_IP" \
        "rm -rf ~/.cometbft/data/mempool.wal" 2>/dev/null || true

    ssh -o ConnectTimeout=5 -o StrictHostKeyChecking=no "$SSH_USER@$VALIDATOR_IP" \
        "sudo systemctl start cometbft" 2>/dev/null || true

    echo "✅ Mempool flushed. Waiting for subnet to restart..."
    sleep 10

    # Verify
    MEMPOOL_NEW=$(ssh -o ConnectTimeout=5 -o StrictHostKeyChecking=no "$SSH_USER@$VALIDATOR_IP" \
        "curl -s http://localhost:26657/num_unconfirmed_txs" 2>/dev/null)
    N_TXS_NEW=$(echo "$MEMPOOL_NEW" | jq -r '.result.n_txs')

    echo "  New mempool size: $N_TXS_NEW transactions"

    if [ "$N_TXS_NEW" = "0" ]; then
        echo "✅ Success! Mempool cleared."
    else
        echo "⚠️  Some transactions may have returned to mempool"
    fi
else
    echo "Operation cancelled."
fi

echo ""



