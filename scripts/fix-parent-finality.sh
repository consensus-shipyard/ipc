#!/bin/bash

# Fix Parent Finality Voting for IPC Subnet
# This script helps diagnose and fix parent finality issues

set -e

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

# Navigate to the ipc-subnet-manager directory
cd "$SCRIPT_DIR/ipc-subnet-manager"

echo "🔧 Fixing Parent Finality Issues"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo "📊 Current Status:"
./ipc-manager info 2>/dev/null | grep -A 5 "Parent Finality"
echo ""

echo "❌ Issues Identified:"
echo "  1. No parent finality votes being sent/received"
echo "  2. Relayer error: '/r314159 has no child'"
echo "  3. 79,754+ parent RPC errors"
echo "  4. Cross-chain fund transactions stuck in mempool"
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "💡 Solution: Restart Validators with Proper Config"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo "This will:"
echo "  • Restart all validator nodes"
echo "  • Re-sync parent finality"
echo "  • Clear stuck transactions from mempool"
echo "  • Resume cross-chain message processing"
echo ""

read -p "Proceed with restart? (yes/no): " answer

if [ "$answer" != "yes" ]; then
    echo "Operation cancelled."
    exit 0
fi

echo ""
echo "🔄 Step 1: Stopping validators..."
./ipc-manager stop

echo ""
echo "⏳ Waiting 10 seconds..."
sleep 10

echo ""
echo "🚀 Step 2: Starting validators..."
./ipc-manager start

echo ""
echo "⏳ Waiting 30 seconds for startup..."
sleep 30

echo ""
echo "🔍 Step 3: Checking status..."
./ipc-manager info | grep -A 10 "Parent Finality"

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Restart Complete!"
echo ""
echo "Next steps:"
echo "  1. Monitor for 5-10 minutes"
echo "  2. Check if parent finality votes appear: ./ipc-manager dashboard"
echo "  3. If transactions still stuck after 10 min, check L1 fund() calls"
echo ""
echo "To monitor: ./ipc-manager dashboard"
echo ""



