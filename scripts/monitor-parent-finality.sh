#!/bin/bash

# IPC Parent Finality Monitoring Script
# Compatible with Zabbix, Nagios, Prometheus, and other monitoring systems
#
# Exit Codes:
#   0 = OK - Finality is healthy
#   1 = WARNING - Finality lag is high
#   2 = CRITICAL - Finality lag is too high or parent finality stuck
#   3 = UNKNOWN - Unable to fetch metrics
#
# Usage:
#   ./monitor-parent-finality.sh [--validator-ip IP] [--warning EPOCHS] [--critical EPOCHS] [--format FORMAT]
#
# Options:
#   --validator-ip IP    Validator IP to query (default: from config)
#   --warning EPOCHS     Warning threshold in epochs (default: 100)
#   --critical EPOCHS    Critical threshold in epochs (default: 1000)
#   --format FORMAT      Output format: text|json|prometheus (default: text)
#   --quiet              Only output metrics, no descriptive text

set -euo pipefail

# Default configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CONFIG_FILE="${SCRIPT_DIR}/ipc-subnet-manager/ipc-subnet-config.yml"
VALIDATOR_IP=""
WARNING_THRESHOLD=100
CRITICAL_THRESHOLD=1000
OUTPUT_FORMAT="text"
QUIET=false

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --validator-ip)
            VALIDATOR_IP="$2"
            shift 2
            ;;
        --warning)
            WARNING_THRESHOLD="$2"
            shift 2
            ;;
        --critical)
            CRITICAL_THRESHOLD="$2"
            shift 2
            ;;
        --format)
            OUTPUT_FORMAT="$2"
            shift 2
            ;;
        --quiet)
            QUIET=true
            shift
            ;;
        --help)
            sed -n '2,/^$/p' "$0" | sed 's/^# //'
            exit 0
            ;;
        *)
            echo "Unknown option: $1" >&2
            exit 3
            ;;
    esac
done

# Get validator IP from config if not specified
if [ -z "$VALIDATOR_IP" ] && [ -f "$CONFIG_FILE" ]; then
    VALIDATOR_IP=$(grep -A 1 "validators:" "$CONFIG_FILE" | grep "ip:" | head -1 | awk '{print $NF}' | tr -d '"')
fi

if [ -z "$VALIDATOR_IP" ]; then
    echo "ERROR: No validator IP specified and couldn't read from config" >&2
    exit 3
fi

# Function to query CometBFT RPC
query_cometbft() {
    local endpoint="$1"
    curl -s --max-time 5 "http://${VALIDATOR_IP}:26657${endpoint}" 2>/dev/null || echo "{}"
}

# Function to query Ethereum RPC
query_eth_rpc() {
    local method="$1"
    local params="${2:-[]}"
    curl -s --max-time 5 -X POST "http://${VALIDATOR_IP}:8545" \
        -H "Content-Type: application/json" \
        --data "{\"jsonrpc\":\"2.0\",\"method\":\"${method}\",\"params\":${params},\"id\":1}" 2>/dev/null || echo "{}"
}

# Function to query parent RPC
query_parent_rpc() {
    local parent_rpc
    if [ -f "$CONFIG_FILE" ]; then
        parent_rpc=$(grep "parent_rpc:" "$CONFIG_FILE" | awk '{print $NF}' | tr -d '"')
    else
        parent_rpc="https://api.calibration.node.glif.io/rpc/v1"
    fi

    curl -s --max-time 5 -X POST "$parent_rpc" \
        -H "Content-Type: application/json" \
        --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' 2>/dev/null || echo "{}"
}

# Fetch metrics using ipc-manager watch-finality output
fetch_metrics() {
    local subnet_height parent_chain_height subnet_finality finality_lag time_since_last_commit status exit_code

    # Get data from watch-finality (run once)
    local finality_output
    finality_output=$(cd "${SCRIPT_DIR}/ipc-subnet-manager" && timeout 10 ./ipc-manager watch-finality --duration 5 2>/dev/null | tail -2 | head -1)

    # Parse the output: Time | Iter | Subnet Finality | Parent Chain | Lag | Subnet Height | Status
    if [ -n "$finality_output" ]; then
        subnet_finality=$(echo "$finality_output" | awk '{print $5}')
        parent_chain_height=$(echo "$finality_output" | awk '{print $7}')
        finality_lag=$(echo "$finality_output" | awk '{print $9}')
        subnet_height=$(echo "$finality_output" | awk '{print $11}')
    else
        # Fallback: query directly
        subnet_height=$(query_cometbft "/status" | jq -r '.result.sync_info.latest_block_height // "0"' 2>/dev/null || echo "0")

        local parent_data
        parent_data=$(query_parent_rpc)
        parent_chain_height=$(echo "$parent_data" | jq -r '.result // "0x0"' | xargs printf "%d\n" 2>/dev/null || echo "0")

        # Query subnet finality from validator
        subnet_finality=$(ssh -o ConnectTimeout=3 -o StrictHostKeyChecking=no -o BatchMode=yes \
            "$(whoami)@${VALIDATOR_IP}" \
            "curl -s http://localhost:26657/status 2>/dev/null | jq -r '.result.sync_info.latest_block_height // \"0\"'" 2>/dev/null || echo "0")

        finality_lag=$((parent_chain_height - subnet_finality))
    fi

    # Ensure we have valid numbers
    subnet_height=${subnet_height:-0}
    subnet_finality=${subnet_finality:-0}
    parent_chain_height=${parent_chain_height:-0}
    finality_lag=${finality_lag:-$((parent_chain_height - subnet_finality))}

    # Try to get last commit time from logs
    time_since_last_commit=$(ssh -o ConnectTimeout=3 -o StrictHostKeyChecking=no -o BatchMode=yes \
        "$(whoami)@${VALIDATOR_IP}" \
        "sudo journalctl -u ipc-node --since '1 hour ago' --no-pager | grep -i 'ParentView' | tail -1 | awk '{print \$1,\$2,\$3}'" 2>/dev/null || echo "unknown")

    # Determine status
    if [ "$subnet_height" -eq 0 ] || [ "$parent_chain_height" -eq 0 ]; then
        status="UNKNOWN"
        exit_code=3
    elif [ "$finality_lag" -ge "$CRITICAL_THRESHOLD" ]; then
        status="CRITICAL"
        exit_code=2
    elif [ "$finality_lag" -ge "$WARNING_THRESHOLD" ]; then
        status="WARNING"
        exit_code=1
    else
        status="OK"
        exit_code=0
    fi

    # Output based on format
    case "$OUTPUT_FORMAT" in
        json)
            cat <<EOF
{
  "status": "${status}",
  "exit_code": ${exit_code},
  "metrics": {
    "subnet_height": ${subnet_height},
    "subnet_finality": ${subnet_finality},
    "parent_chain_height": ${parent_chain_height},
    "finality_lag": ${finality_lag},
    "warning_threshold": ${WARNING_THRESHOLD},
    "critical_threshold": ${CRITICAL_THRESHOLD}
  },
  "last_commit": "${time_since_last_commit}",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF
            ;;
        prometheus)
            cat <<EOF
# HELP ipc_subnet_height Current subnet block height
# TYPE ipc_subnet_height gauge
ipc_subnet_height ${subnet_height}

# HELP ipc_subnet_finality Parent epoch finalized by subnet
# TYPE ipc_subnet_finality gauge
ipc_subnet_finality ${subnet_finality}

# HELP ipc_parent_chain_height Current parent chain height
# TYPE ipc_parent_chain_height gauge
ipc_parent_chain_height ${parent_chain_height}

# HELP ipc_finality_lag Epochs behind parent chain
# TYPE ipc_finality_lag gauge
ipc_finality_lag ${finality_lag}

# HELP ipc_finality_status Status (0=OK, 1=WARNING, 2=CRITICAL, 3=UNKNOWN)
# TYPE ipc_finality_status gauge
ipc_finality_status ${exit_code}
EOF
            ;;
        text)
            if [ "$QUIET" = true ]; then
                echo "${status}|lag=${finality_lag}"
            else
                echo "=================================="
                echo "IPC Parent Finality Monitor"
                echo "=================================="
                echo "Status: ${status}"
                echo ""
                echo "Metrics:"
                echo "  Subnet Height:      ${subnet_height}"
                echo "  Subnet Finality:    ${subnet_finality}"
                echo "  Parent Chain:       ${parent_chain_height}"
                echo "  Finality Lag:       ${finality_lag} epochs"
                echo ""
                echo "Thresholds:"
                echo "  Warning:            ${WARNING_THRESHOLD} epochs"
                echo "  Critical:           ${CRITICAL_THRESHOLD} epochs"
                echo ""
                if [ "$finality_lag" -ge "$CRITICAL_THRESHOLD" ]; then
                    echo "⚠️  CRITICAL: Finality lag exceeds ${CRITICAL_THRESHOLD} epochs!"
                    echo "   Cross-chain messages may not be processed."
                    echo "   Action required: Check parent RPC and restart validators."
                elif [ "$finality_lag" -ge "$WARNING_THRESHOLD" ]; then
                    echo "⚠️  WARNING: Finality lag is high (${finality_lag} epochs)"
                    echo "   Monitor closely. Consider checking validator logs."
                else
                    echo "✅ OK: Finality is healthy"
                fi
                echo ""
                echo "Last Check: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
                echo "=================================="
            fi
            ;;
    esac

    return $exit_code
}

# Main execution
fetch_metrics
exit $?

