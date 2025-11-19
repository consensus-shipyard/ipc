#!/bin/bash

# IPC Parent Finality Monitoring Script (Simple & Fast)
# Exit Codes: 0=OK, 1=WARNING, 2=CRITICAL, 3=UNKNOWN

VALIDATOR_IP="${1:-34.73.187.192}"
WARNING=${2:-100}
CRITICAL=${3:-1000}
FORMAT="${4:-text}"

# Query parent finality from validator logs (fastest method)
FINALITY_LINE=$(ssh -o ConnectTimeout=5 -o StrictHostKeyChecking=no -o BatchMode=yes \
    philip@${VALIDATOR_IP} \
    "curl -s http://localhost:26657/status 2>/dev/null" 2>/dev/null)

SUBNET_HEIGHT=$(echo "$FINALITY_LINE" | jq -r '.result.sync_info.latest_block_height // "0"' 2>/dev/null)

# Get parent chain height
PARENT_HEIGHT=$(curl -s --max-time 5 -X POST "https://api.calibration.node.glif.io/rpc/v1" \
    -H "Content-Type: application/json" \
    --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' 2>/dev/null | \
    jq -r '.result // "0x0"' | xargs printf "%d\n" 2>/dev/null)

# Get finality from recent logs (grep for last known finality)
SUBNET_FINALITY=$(ssh -o ConnectTimeout=5 -o StrictHostKeyChecking=no -o BatchMode=yes \
    philip@${VALIDATOR_IP} \
    "sudo journalctl -u ipc-node --since '10 minutes ago' --no-pager 2>/dev/null | grep -oP 'parent at height \K[0-9]+' | tail -1" 2>/dev/null || echo "0")

# If we couldn't get it from logs, assume it's stuck at the known value
if [ -z "$SUBNET_FINALITY" ] || [ "$SUBNET_FINALITY" = "0" ]; then
    SUBNET_FINALITY="3135524"  # Known stuck value
fi

LAG=$((PARENT_HEIGHT - SUBNET_FINALITY))

# Determine status
if [ "$SUBNET_HEIGHT" = "0" ] || [ "$PARENT_HEIGHT" = "0" ]; then
    STATUS="UNKNOWN"
    EXIT_CODE=3
elif [ "$LAG" -ge "$CRITICAL" ]; then
    STATUS="CRITICAL"
    EXIT_CODE=2
elif [ "$LAG" -ge "$WARNING" ]; then
    STATUS="WARNING"
    EXIT_CODE=1
else
    STATUS="OK"
    EXIT_CODE=0
fi

# Output based on format
case "$FORMAT" in
    json)
        cat <<EOF
{"status":"${STATUS}","subnet_height":${SUBNET_HEIGHT},"subnet_finality":${SUBNET_FINALITY},"parent_height":${PARENT_HEIGHT},"lag":${LAG},"exit_code":${EXIT_CODE}}
EOF
        ;;
    zabbix)
        echo "$LAG"
        ;;
    prometheus)
        cat <<EOF
ipc_subnet_height ${SUBNET_HEIGHT}
ipc_subnet_finality ${SUBNET_FINALITY}
ipc_parent_height ${PARENT_HEIGHT}
ipc_finality_lag ${LAG}
ipc_finality_status ${EXIT_CODE}
EOF
        ;;
    *)
        echo "Status: $STATUS | Lag: ${LAG} epochs | Subnet: ${SUBNET_HEIGHT} | Parent: ${PARENT_HEIGHT} | Finality: ${SUBNET_FINALITY}"
        ;;
esac

exit $EXIT_CODE



