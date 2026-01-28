#!/usr/bin/env bash
# Gas Estimation Helper Script
# Usage: ./estimate-gas.sh <from_address> <to_address> [data] [value]

set -euo pipefail

RPC_URL="${RPC_URL:-http://34.73.187.192:8545}"
FROM_ADDR="${1}"
TO_ADDR="${2}"
DATA="${3:-0x}"
VALUE="${4:-0x0}"

# Build JSON RPC request
REQUEST=$(cat << EOF
{
  "jsonrpc":"2.0",
  "method":"eth_estimateGas",
  "params":[{
    "from":"${FROM_ADDR}",
    "to":"${TO_ADDR}",
    "data":"${DATA}",
    "value":"${VALUE}"
  }],
  "id":1
}
EOF
)

echo "Estimating gas..."
echo "=================="

# Get gas estimate
GAS_HEX=$(curl -s -X POST "${RPC_URL}" \
  -H "Content-Type: application/json" \
  -d "${REQUEST}" | jq -r '.result')

if [ "$GAS_HEX" = "null" ] || [ -z "$GAS_HEX" ]; then
  echo "Error: Failed to get gas estimate"
  exit 1
fi

# Convert and display
python3 << EOF
gas = int("${GAS_HEX}", 16)

# Different gas prices
prices = [1, 2, 5, 10, 50]

print(f"\nGas Estimate: {gas:,} gas (${GAS_HEX})")
print(f"\nEstimated Cost at Different Gas Prices:")
print("=" * 50)

for gwei in prices:
    cost_tfil = (gas * gwei) / 10**9
    cost_mtfil = cost_tfil * 1000
    print(f"  {gwei:3d} gwei: {cost_tfil:12.9f} TFIL ({cost_mtfil:8.3f} mTFIL)")

# Recommended with buffer
gas_with_buffer = int(gas * 1.2)
print(f"\nRecommended (with 20% buffer): {gas_with_buffer:,} gas")
EOF




