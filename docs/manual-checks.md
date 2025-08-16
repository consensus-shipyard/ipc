# IPC Manual Checks

This document contains manual methods for checking various aspects of IPC subnet deployments using command-line tools like `cast` and `ipc-cli`. These checks are useful for debugging, validation, and understanding the state of your IPC network.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Subnet Approval Status](#subnet-approval-status)
- [Troubleshooting](#troubleshooting)

## Prerequisites

Before running these manual checks, ensure you have:

- [Foundry](https://book.getfoundry.sh/getting-started/installation) installed (`cast` command)
- IPC CLI built: `cargo build --release --bin ipc-cli`
- Access to an RPC endpoint for the parent chain
- Gateway contract address
- Subnet ID to check

## Subnet Approval Status

### Overview

The most reliable way to check if a subnet is approved (registered) in a gateway is to use the `getSubnet()` method directly on the gateway contract. This returns a boolean indicating whether the subnet exists in the gateway's registry.

### Method 1: Direct `getSubnet()` Call (Recommended)

This is the most authoritative method to check subnet approval status.

```bash
#!/bin/bash

# Configuration
GATEWAY_ADDR="0x77aa40b105843728088c0132e43fc44348881da8"
RPC_URL="http://localhost:8545"
SUBNET_ID="/r31337/t410fmdoryansepnf2szlzr7pj6mivlvlpfs6qzajty"

# Extract root chain ID (remove /r prefix)
ROOT_CHAIN=$(echo "$SUBNET_ID" | grep -oE '/r[0-9]+' | sed 's|/r||')
echo "Root chain: $ROOT_CHAIN"

# Convert f410 address to Ethereum address using IPC CLI
F410_ADDR=$(echo "$SUBNET_ID" | grep -oE 't410[0-9a-z]+')
if [[ -n "$F410_ADDR" ]]; then
    ACTOR_ETH=$(./target/release/ipc-cli util f4-to-eth-addr --addr "$F410_ADDR" 2>/dev/null | grep -oE '0x[a-fA-F0-9]{40}')
    echo "Actor address: $ACTOR_ETH"
    ROUTE_ARRAY="[$ACTOR_ETH]"
else
    # Root network case
    ROUTE_ARRAY="[]"
fi

# Call getSubnet with SubnetID struct: (uint64 root, address[] route)
echo "Checking approval for subnet: $SUBNET_ID"
RESULT=$(cast call "$GATEWAY_ADDR" \
  "getSubnet((uint64,address[]))(bool,(uint256,uint256,uint256,uint64,uint64,(uint64,address[])))" \
  "($ROOT_CHAIN,$ROUTE_ARRAY)" \
  --rpc-url "$RPC_URL")

if [[ $? -eq 0 && -n "$RESULT" ]]; then
    # Parse the boolean result (first 32 bytes after 0x)
    FOUND_HEX=$(echo "$RESULT" | cut -c3-66)
    FOUND_DEC=$(cast --to-dec "0x$FOUND_HEX" 2>/dev/null)

    if [[ "$FOUND_DEC" == "1" ]]; then
        echo "✅ Subnet IS APPROVED and registered in gateway"
    else
        echo "❌ Subnet is NOT APPROVED"
        echo "💡 Run: ipc-cli subnet approve --subnet $SUBNET_ID"
    fi
else
    echo "❌ Failed to query gateway contract"
fi
```

### Method 2: Check Root Network Approval

For root networks (e.g., `/r31337`), the route array is empty:

```bash
# Root network approval check
GATEWAY_ADDR="0x77aa40b105843728088c0132e43fc44348881da8"
RPC_URL="http://localhost:8545"
ROOT_CHAIN="31337"

cast call "$GATEWAY_ADDR" \
  "getSubnet((uint64,address[]))(bool,tuple)" \
  "($ROOT_CHAIN,[])" \
  --rpc-url "$RPC_URL"
```

### Method 3: Using `listSubnets()` (Less Reliable)

This method requires parsing complex ABI-encoded data but can be useful to see all registered subnets:

```bash
# Get all registered subnets
RESULT=$(cast call "$GATEWAY_ADDR" "listSubnets()" --rpc-url "$RPC_URL")

# Check total count first
TOTAL=$(cast call "$GATEWAY_ADDR" "totalSubnets()" --rpc-url "$RPC_URL")
COUNT=$(cast --to-dec "$TOTAL" 2>/dev/null || echo "0")

echo "Gateway has $COUNT registered subnet(s)"

if [[ "$COUNT" == "0" ]]; then
    echo "❌ No subnets registered in gateway"
else
    echo "📋 Registered subnets data (ABI-encoded):"
    echo "$RESULT"
fi
```

### Understanding SubnetID Structure

The `SubnetID` struct in Solidity has this format:
```solidity
struct SubnetID {
    uint64 root;        // Chain ID of root network
    address[] route;    // Array of subnet actor addresses from root to target
}
```

Examples:
- Root network `/r31337`: `(31337, [])`
- L2 subnet `/r31337/0x123...`: `(31337, [0x123...])`
- L3 subnet `/r31337/0x123.../0x456...`: `(31337, [0x123..., 0x456...])`

### Converting f410 to Ethereum Addresses

IPC uses f410 addresses in subnet IDs, but the gateway contract expects Ethereum addresses:

```bash
# Using IPC CLI
F410_ADDR="t410fmdoryansepnf2szlzr7pj6mivlvlpfs6qzajty"
ETH_ADDR=$(./target/release/ipc-cli util f4-to-eth-addr --addr "$F410_ADDR")
echo "f410: $F410_ADDR"
echo "Ethereum: $ETH_ADDR"
```

### Complete One-Liner

```bash
# Quick subnet approval check (replace variables)
cast call 0x77aa40b105843728088c0132e43fc44348881da8 "getSubnet((uint64,address[]))(bool,tuple)" "(31337,[0x...])" --rpc-url http://localhost:8545 | cut -c3-66 | xargs -I {} cast --to-dec 0x{} | grep -q "1" && echo "✅ APPROVED" || echo "❌ NOT APPROVED"
```

## Set Federated Power, Bootstrap, and Approve Subnet

For federated subnets, you must follow this specific order: set federated power → verify bootstrap → approve subnet.

### Complete Federated Subnet Setup Workflow

```bash
# Configuration
SUBNET_ID="/r31337/t410fhchbtsjh2u2vb62fy4jrhjbus5ztl4bbazr34pq"
SUBNET_ACTOR="0x388e19c927d53550fb45c71313a434977335f021"
GATEWAY_ADDR="0xf953b3a269d80e3eb0f2947630da976b896a8c5b"
RPC_URL="http://localhost:8545"

# Validator configuration
VALIDATOR_ADDRESS="0x70997970c51812dc3a010c7d01b50e0d17dc79c8"
VALIDATOR_PUBKEY="04ba5734d8f7091719471e7f7ed6b9df170dc70cc661ca05e688601ad984f068b0d67351e5f06073092499336ab0839ef8a521afd334e53807205fa2f08eec74f4"
VALIDATOR_POWER="1"
FROM_ADDRESS="0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266"
```

### Step 1: Verify Subnet Configuration

Before setting federated power, check the subnet's requirements:

```bash
echo "🔍 Checking subnet configuration..."

# Check permission mode (should be 1 for federated)
PERMISSION_MODE=$(cast call $SUBNET_ACTOR "permissionMode()" --rpc-url $RPC_URL)
MODE_DEC=$(cast --to-dec $PERMISSION_MODE)
echo "📋 Permission Mode: $MODE_DEC (should be 1 for federated)"

# Check minimum validators required
MIN_VALIDATORS=$(cast call $SUBNET_ACTOR "minValidators()" --rpc-url $RPC_URL)
MIN_VAL_DEC=$(cast --to-dec $MIN_VALIDATORS)
echo "📋 Minimum Validators Required: $MIN_VAL_DEC"

# Check current bootstrap status
BOOTSTRAPPED=$(cast call $SUBNET_ACTOR "bootstrapped()" --rpc-url $RPC_URL)
BOOTSTRAPPED_DEC=$(cast --to-dec $BOOTSTRAPPED)
echo "📋 Currently Bootstrapped: $BOOTSTRAPPED_DEC (0=false, 1=true)"
```

### Step 2: Set Federated Power (This automatically bootstraps if requirements are met)

```bash
echo "⚡ Setting federated power for validators..."

# Set federated power - this will automatically bootstrap the subnet if minimum requirements are met
ipc-cli subnet set-federated-power \
  --subnet $SUBNET_ID \
  --validator-addresses $VALIDATOR_ADDRESS \
  --validator-pubkeys $VALIDATOR_PUBKEY \
  --validator-power $VALIDATOR_POWER \
  --from $FROM_ADDRESS

echo "✅ Federated power command executed"
```

### Step 3: Verify Bootstrap Status

After setting federated power, the subnet should automatically bootstrap:

```bash
echo "🔄 Verifying bootstrap status..."

# Wait a moment for transaction to be mined
sleep 2

# Check if subnet is now bootstrapped
BOOTSTRAPPED_AFTER=$(cast call $SUBNET_ACTOR "bootstrapped()" --rpc-url $RPC_URL)
BOOTSTRAPPED_AFTER_DEC=$(cast --to-dec $BOOTSTRAPPED_AFTER)

if [[ "$BOOTSTRAPPED_AFTER_DEC" == "1" ]]; then
    echo "✅ Subnet is now BOOTSTRAPPED"
else
    echo "❌ Subnet is still NOT BOOTSTRAPPED"
    echo "💡 Check if you have enough validators or sufficient power"
    exit 1
fi

# Verify genesis validators were set
GENESIS_VALIDATORS=$(cast call $SUBNET_ACTOR "getGenesisValidators()" --rpc-url $RPC_URL 2>/dev/null)
if [[ -n "$GENESIS_VALIDATORS" && "$GENESIS_VALIDATORS" != "0x" ]]; then
    echo "✅ Genesis validators are configured"
else
    echo "❌ Genesis validators not found"
fi
```

### Step 4: Check Approval Status (Should still be false before approval)

```bash
echo "🔍 Checking current approval status..."

# Check if subnet is approved in gateway
APPROVAL_RESULT=$(cast call "$GATEWAY_ADDR" \
  "getSubnet((uint64,address[]))(bool,tuple)" \
  "(31337,[$SUBNET_ACTOR])" \
  --rpc-url "$RPC_URL" 2>/dev/null)

if [[ $? -eq 0 && -n "$APPROVAL_RESULT" ]]; then
    FOUND_HEX=$(echo "$APPROVAL_RESULT" | cut -c3-66)
    FOUND_DEC=$(cast --to-dec "0x$FOUND_HEX" 2>/dev/null || echo "0")

    if [[ "$FOUND_DEC" == "1" ]]; then
        echo "✅ Subnet is already APPROVED in gateway"
    else
        echo "📋 Subnet is bootstrapped but NOT YET APPROVED in gateway"
        echo "💡 This is expected - proceed to approval step"
    fi
else
    echo "❌ Failed to check approval status"
fi
```

### Step 5: Approve the Subnet

Now that the subnet is bootstrapped, approval should work:

```bash
echo "✅ Approving subnet in gateway..."

# Approve the subnet
ipc-cli subnet approve --subnet $SUBNET_ID --from $FROM_ADDRESS

echo "✅ Approval command executed"
```

### Step 6: Verify Final Approval Status

```bash
echo "🔄 Verifying final approval status..."

# Wait for transaction to be mined
sleep 2

# Check final approval status
FINAL_APPROVAL=$(cast call "$GATEWAY_ADDR" \
  "getSubnet((uint64,address[]))(bool,tuple)" \
  "(31337,[$SUBNET_ACTOR])" \
  --rpc-url "$RPC_URL" 2>/dev/null)

if [[ $? -eq 0 && -n "$FINAL_APPROVAL" ]]; then
    FINAL_FOUND_HEX=$(echo "$FINAL_APPROVAL" | cut -c3-66)
    FINAL_FOUND_DEC=$(cast --to-dec "0x$FINAL_FOUND_HEX" 2>/dev/null || echo "0")

    if [[ "$FINAL_FOUND_DEC" == "1" ]]; then
        echo "🎉 SUCCESS: Subnet is fully BOOTSTRAPPED and APPROVED!"
        echo "📋 Subnet Status Summary:"
        echo "   - Bootstrapped: ✅"
        echo "   - Approved in Gateway: ✅"
        echo "   - Ready for use: ✅"
    else
        echo "❌ Subnet is bootstrapped but approval failed"
        echo "💡 Check transaction logs for approval errors"
    fi
else
    echo "❌ Failed to verify final approval status"
fi
```

### Complete One-Shot Script

```bash
#!/bin/bash
# Complete federated subnet setup script

set -e  # Exit on any error

# Configuration - UPDATE THESE VALUES
SUBNET_ID="/r31337/t410fhchbtsjh2u2vb62fy4jrhjbus5ztl4bbazr34pq"
SUBNET_ACTOR="0x388e19c927d53550fb45c71313a434977335f021"
GATEWAY_ADDR="0xf953b3a269d80e3eb0f2947630da976b896a8c5b"
RPC_URL="http://localhost:8545"
VALIDATOR_ADDRESS="0x70997970c51812dc3a010c7d01b50e0d17dc79c8"
VALIDATOR_PUBKEY="04ba5734d8f7091719471e7f7ed6b9df170dc70cc661ca05e688601ad984f068b0d67351e5f06073092499336ab0839ef8a521afd334e53807205fa2f08eec74f4"
VALIDATOR_POWER="1"
FROM_ADDRESS="0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266"

echo "🚀 Starting federated subnet setup process..."
echo "📋 Subnet: $SUBNET_ID"
echo ""

# Step 1: Verify subnet configuration
echo "1️⃣ Verifying subnet configuration..."
PERMISSION_MODE=$(cast call $SUBNET_ACTOR "permissionMode()" --rpc-url $RPC_URL)
MODE_DEC=$(cast --to-dec $PERMISSION_MODE)
if [[ "$MODE_DEC" != "1" ]]; then
    echo "❌ Subnet is not federated (mode: $MODE_DEC). Expected mode: 1"
    exit 1
fi
echo "✅ Subnet is federated"

# Step 2: Set federated power
echo ""
echo "2️⃣ Setting federated power..."
ipc-cli subnet set-federated-power \
  --subnet "$SUBNET_ID" \
  --validator-addresses "$VALIDATOR_ADDRESS" \
  --validator-pubkeys "$VALIDATOR_PUBKEY" \
  --validator-power "$VALIDATOR_POWER" \
  --from "$FROM_ADDRESS"

# Step 3: Verify bootstrap
echo ""
echo "3️⃣ Verifying bootstrap status..."
sleep 3  # Wait for transaction
BOOTSTRAPPED=$(cast call $SUBNET_ACTOR "bootstrapped()" --rpc-url $RPC_URL)
if [[ "$(cast --to-dec $BOOTSTRAPPED)" != "1" ]]; then
    echo "❌ Subnet failed to bootstrap"
    exit 1
fi
echo "✅ Subnet is bootstrapped"

# Step 4: Approve subnet
echo ""
echo "4️⃣ Approving subnet..."
ipc-cli subnet approve --subnet "$SUBNET_ID" --from "$FROM_ADDRESS"

# Step 5: Verify approval
echo ""
echo "5️⃣ Verifying approval..."
sleep 3  # Wait for transaction
APPROVAL_RESULT=$(cast call "$GATEWAY_ADDR" \
  "getSubnet((uint64,address[]))(bool,tuple)" \
  "(31337,[$SUBNET_ACTOR])" \
  --rpc-url "$RPC_URL")
APPROVED=$(cast --to-dec $(echo "$APPROVAL_RESULT" | cut -c3-66))

if [[ "$APPROVED" == "1" ]]; then
    echo "🎉 SUCCESS: Subnet is fully configured and approved!"
else
    echo "❌ Approval verification failed"
    exit 1
fi
```

## Subnet Actor Diagnostics

Before checking approval status, it's often helpful to diagnose the subnet actor's current state.

### Check Subnet Actor Configuration

```bash
# Configuration
SUBNET_ACTOR="0x388e19c927d53550fb45c71313a434977335f021"  # Your subnet actor address
RPC_URL="http://localhost:8545"

# Check permission mode (0=Collateral, 1=Federated, 2=Static)
PERMISSION_MODE=$(cast call $SUBNET_ACTOR "permissionMode()" --rpc-url $RPC_URL)
MODE_DEC=$(cast --to-dec $PERMISSION_MODE)
case $MODE_DEC in
    0) echo "📋 Permission Mode: Collateral" ;;
    1) echo "📋 Permission Mode: Federated" ;;
    2) echo "📋 Permission Mode: Static" ;;
    *) echo "📋 Permission Mode: Unknown ($MODE_DEC)" ;;
esac

# Check minimum validators required
MIN_VALIDATORS=$(cast call $SUBNET_ACTOR "minValidators()" --rpc-url $RPC_URL)
echo "📋 Minimum Validators Required: $(cast --to-dec $MIN_VALIDATORS)"

# Check if subnet is bootstrapped
BOOTSTRAPPED=$(cast call $SUBNET_ACTOR "bootstrapped()" --rpc-url $RPC_URL)
BOOTSTRAPPED_DEC=$(cast --to-dec $BOOTSTRAPPED)
if [[ "$BOOTSTRAPPED_DEC" == "1" ]]; then
    echo "✅ Subnet is BOOTSTRAPPED"
else
    echo "❌ Subnet is NOT BOOTSTRAPPED"
fi

# Check total validator power
TOTAL_POWER=$(cast call $SUBNET_ACTOR "getTotalCurrentPower()" --rpc-url $RPC_URL 2>/dev/null || echo "0x0")
echo "📊 Total Current Power: $(cast --to-dec $TOTAL_POWER 2>/dev/null || echo "0")"

# Check minimum activation collateral (for collateral subnets)
MIN_COLLATERAL=$(cast call $SUBNET_ACTOR "minActivationCollateral()" --rpc-url $RPC_URL 2>/dev/null || echo "0x0")
echo "💰 Min Activation Collateral: $(cast --to-dec $MIN_COLLATERAL 2>/dev/null || echo "0")"
```

### Federated Subnet Setup Requirements

For federated subnets (permission mode = 1), you must set validator federated power before approval:

```bash
# Check if validators have been set (for federated subnets)
if [[ "$MODE_DEC" == "1" ]]; then
    echo "🔍 Federated subnet detected - checking validator setup..."

    # Try to get genesis validators
    GENESIS_VALIDATORS=$(cast call $SUBNET_ACTOR "getGenesisValidators()" --rpc-url $RPC_URL 2>/dev/null || echo "")

    if [[ -n "$GENESIS_VALIDATORS" && "$GENESIS_VALIDATORS" != "0x" ]]; then
        echo "✅ Genesis validators are configured"
    else
        echo "❌ No genesis validators found"
        echo "💡 Set federated power first: ipc-cli subnet set-federated-power --subnet <subnet_id> --validator <addr> --public-key <key> --power <power>"
    fi
fi
```

### Complete Subnet Diagnostics Script

```bash
#!/bin/bash
# Complete subnet diagnostic check

SUBNET_ID="/r31337/t410fhchbtsjh2u2vb62fy4jrhjbus5ztl4bbazr34pq"
SUBNET_ACTOR="0x388e19c927d53550fb45c71313a434977335f021"
GATEWAY_ADDR="0xf953b3a269d80e3eb0f2947630da976b896a8c5b"
RPC_URL="http://localhost:8545"

echo "🔍 Subnet Diagnostics for: $SUBNET_ID"
echo "🏠 Subnet Actor: $SUBNET_ACTOR"
echo "🌐 Gateway: $GATEWAY_ADDR"
echo ""

# 1. Check subnet actor exists
echo "1️⃣ Checking subnet actor contract..."
ACTOR_CODE=$(cast code $SUBNET_ACTOR --rpc-url $RPC_URL)
if [[ "$ACTOR_CODE" == "0x" ]]; then
    echo "❌ Subnet actor contract does not exist!"
    exit 1
else
    echo "✅ Subnet actor contract exists"
fi

# 2. Check permission mode
echo ""
echo "2️⃣ Checking subnet configuration..."
PERMISSION_MODE=$(cast call $SUBNET_ACTOR "permissionMode()" --rpc-url $RPC_URL)
MODE_DEC=$(cast --to-dec $PERMISSION_MODE)
case $MODE_DEC in
    0) echo "📋 Permission Mode: Collateral" ;;
    1) echo "📋 Permission Mode: Federated" ;;
    2) echo "📋 Permission Mode: Static" ;;
    *) echo "📋 Permission Mode: Unknown ($MODE_DEC)" ;;
esac

# 3. Check bootstrap status
BOOTSTRAPPED=$(cast call $SUBNET_ACTOR "bootstrapped()" --rpc-url $RPC_URL)
BOOTSTRAPPED_DEC=$(cast --to-dec $BOOTSTRAPPED)
if [[ "$BOOTSTRAPPED_DEC" == "1" ]]; then
    echo "✅ Subnet is BOOTSTRAPPED"
else
    echo "❌ Subnet is NOT BOOTSTRAPPED"
fi

# 4. Check minimum validators
MIN_VALIDATORS=$(cast call $SUBNET_ACTOR "minValidators()" --rpc-url $RPC_URL)
echo "📋 Minimum Validators Required: $(cast --to-dec $MIN_VALIDATORS)"

# 5. Check approval status
echo ""
echo "3️⃣ Checking approval status..."
ROOT_CHAIN="31337"
RESULT=$(cast call "$GATEWAY_ADDR" \
  "getSubnet((uint64,address[]))(bool,tuple)" \
  "($ROOT_CHAIN,[$SUBNET_ACTOR])" \
  --rpc-url "$RPC_URL" 2>/dev/null)

if [[ $? -eq 0 && -n "$RESULT" ]]; then
    FOUND_HEX=$(echo "$RESULT" | cut -c3-66)
    FOUND_DEC=$(cast --to-dec "0x$FOUND_HEX" 2>/dev/null || echo "0")

    if [[ "$FOUND_DEC" == "1" ]]; then
        echo "✅ Subnet IS APPROVED in gateway"
    else
        echo "❌ Subnet is NOT APPROVED in gateway"
    fi
else
    echo "❌ Failed to check approval status"
fi

# 6. Recommendations
echo ""
echo "💡 Recommendations:"
if [[ "$BOOTSTRAPPED_DEC" == "0" ]]; then
    if [[ "$MODE_DEC" == "1" ]]; then
        echo "   → Set federated power for validators: ipc-cli subnet set-federated-power"
    else
        echo "   → Add validators with stake: ipc-cli subnet join"
    fi
else
    if [[ "$FOUND_DEC" == "0" ]]; then
        echo "   → Approve the subnet: ipc-cli subnet approve --subnet $SUBNET_ID"
    else
        echo "   → Subnet is fully configured and approved! 🎉"
    fi
fi
```

## Troubleshooting

### Common Issues

1. **Federated subnet not bootstrapped**: Must set federated power for minimum number of validators first
2. **Invalid f410 to Ethereum conversion**: Ensure you're using the correct conversion method
3. **Wrong gateway address**: Verify the gateway address matches your subnet configuration
4. **RPC connection issues**: Check that your RPC URL is accessible and correct
5. **Malformed SubnetID**: Ensure the route array contains valid Ethereum addresses
6. **Insufficient validators**: Check that you have at least `minValidators` configured

### Debugging Tips

1. **Check if gateway exists**:
   ```bash
   cast code $GATEWAY_ADDR --rpc-url $RPC_URL
   ```

2. **Verify total subnets count**:
   ```bash
   cast call $GATEWAY_ADDR "totalSubnets()" --rpc-url $RPC_URL
   ```

3. **Test with a known working subnet**: Use a subnet you know is approved to verify your commands work

4. **Check subnet actor configuration**: Use the diagnostic script above

### Error Codes

- `✅ APPROVED`: Subnet is registered and approved in the gateway
- `❌ NOT APPROVED`: Subnet exists but is not registered in the gateway
- `❌ NOT BOOTSTRAPPED`: Subnet actor exists but needs validator setup
- `❌ Failed to query`: Network, RPC, or contract call issues

---

*This document will be expanded with additional manual checks as they are identified and validated.*
