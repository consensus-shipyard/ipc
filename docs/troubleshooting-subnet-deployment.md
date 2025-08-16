# Troubleshooting Subnet Deployment and Validator Issues

This guide documents the manual troubleshooting steps for investigating subnet deployment and validator configuration issues in IPC.

## Problem Description

When subnets show "0 validators" in the UI despite being configured with validators, or when the UI shows "unknown permission mode", the issue is typically related to:

1. **Subnet registration**: Subnets may be deployed but not properly registered with the parent network
2. **Configuration mismatch**: The local config may not match the actual on-chain state
3. **Missing validator configuration**: Validators may not be properly configured in the subnet contracts

## Manual Troubleshooting Steps

### Step 1: Verify IPC Configuration

```bash
# Check the local IPC configuration
cat ~/.ipc/config.toml

# Look for:
# - Subnet IDs and their corresponding contract addresses
# - Gateway addresses for parent networks
# - Registry addresses
```

### Step 2: Verify Contract Deployment on Blockchain

```bash
# Check if subnet gateway contracts exist
cast code <GATEWAY_ADDRESS> --rpc-url <RPC_URL>

# Example:
cast code 0x02df3a3f960393f5b349e40a599feda91a7cc1a7 --rpc-url http://localhost:8545

# If it returns "0x" the contract doesn't exist
# If it returns bytecode, the contract exists
```

### Step 3: Convert Subnet Actor Addresses

```bash
# Convert Filecoin f4 addresses to Ethereum format
./target/release/ipc-cli util f4-to-eth-addr --addr <F4_ADDRESS>

# Example:
./target/release/ipc-cli util f4-to-eth-addr --addr "t410ffn6xkpjrmbnlm5t7dcivmltjgwlkt2yu4ogttii"
```

### Step 4: Verify Subnet Actor Contracts

```bash
# Check if the subnet actor contract exists
cast code <SUBNET_ACTOR_ETH_ADDRESS> --rpc-url <RPC_URL>

# Example:
cast code 0x2b7d753d31605ab6767f1891562e693596a9eb14 --rpc-url http://localhost:8545
```

### Step 5: Check Subnet Registration with Parent

```bash
# List subnets registered with the parent network
./target/release/ipc-cli subnet list --parent <PARENT_SUBNET_ID> --gateway-address <GATEWAY_ADDRESS>

# Example:
./target/release/ipc-cli subnet list --parent "/r31337" --gateway-address "0x02df3a3f960393f5b349e40a599feda91a7cc1a7"
```

### Step 6: Check Gateway Total Subnets

```bash
# Check how many subnets are registered in the gateway
cast call <GATEWAY_ADDRESS> "totalSubnets()" --rpc-url <RPC_URL>

# Example:
cast call 0xcd0048a5628b37b8f743cc2fea18817a29e97270 "totalSubnets()" --rpc-url http://localhost:8545
```

### Step 7: Verify Validator Configuration

```bash
# List validators for a specific subnet
./target/release/ipc-cli subnet list-validators --subnet <SUBNET_ID>

# Example:
./target/release/ipc-cli subnet list-validators --subnet "/r31337/t410ffn6xkpjrmbnlm5t7dcivmltjgwlkt2yu4ogttii"
```

### Step 8: Check Subnet Genesis Epoch

```bash
# Verify subnet genesis epoch (helps confirm subnet is operational)
./target/release/ipc-cli subnet genesis-epoch --subnet <SUBNET_ID>

# Example:
./target/release/ipc-cli subnet genesis-epoch --subnet "/r31337/t410ffn6xkpjrmbnlm5t7dcivmltjgwlkt2yu4ogttii"
```

### Step 9: Investigate Transaction History

```bash
# Find contract deployment transactions from a specific deployer
deployer="0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266"

# Check recent blocks for contract deployments
for block_num in {200..236}; do
  hex_block=$(printf '0x%x' $block_num)
  tx_hash=$(cast rpc eth_getBlockByNumber "$hex_block" true --rpc-url http://localhost:8545 2>/dev/null | jq -r '.transactions[0].hash // empty' 2>/dev/null)

  if [[ "$tx_hash" != "" && "$tx_hash" != "null" ]]; then
    contract_addr=$(cast receipt "$tx_hash" --field contractAddress --rpc-url http://localhost:8545 2>/dev/null)

    if [[ "$contract_addr" != "" && "$contract_addr" != "null" ]]; then
      from_addr=$(cast receipt "$tx_hash" --field from --rpc-url http://localhost:8545 2>/dev/null)
      echo "Block $block_num: Contract $contract_addr deployed by $from_addr (TX: $tx_hash)"
    fi
  fi
done
```

### Step 10: Check Contract Events

```bash
# Check for events emitted by gateway contracts
cast logs --address <GATEWAY_ADDRESS> --from-block 1 --to-block latest --rpc-url <RPC_URL>

# Example:
cast logs --address "0xcD0048A5628B37B8f743cC2FeA18817A29e97270" --from-block 1 --to-block latest --rpc-url http://localhost:8545
```

## Common Issues and Solutions

### Issue 1: Subnet Shows 0 Validators
**Cause**: Subnet is not properly registered with parent network
**Solution**:
1. Verify the subnet is listed in parent gateway: `ipc-cli subnet list --parent <PARENT_ID>`
2. If not listed, re-register the subnet with the parent

### Issue 2: "Unknown Permission Mode"
**Cause**: Backend cannot retrieve subnet configuration from parent network
**Solution**:
1. Check if parent network connection is properly configured
2. Verify the subnet contract exists and is accessible
3. Check network connectivity to parent RPC endpoint

### Issue 3: "Subnet Does Not Exist" Error
**Cause**: Subnet contract exists but is not registered in parent's subnet registry
**Solution**:
1. Check if subnet is listed in parent gateway: `totalSubnets()` call
2. Verify the gateway address in config matches the actual deployed gateway
3. Re-register subnet if needed

### Issue 4: Contract Address Mismatch
**Cause**: Config contains wrong contract addresses
**Solution**:
1. Find actual deployed contracts using transaction history
2. Update config.toml with correct addresses
3. Verify addresses by checking contract bytecode

## Key Debugging Commands Reference

```bash
# Essential troubleshooting commands
./target/release/ipc-cli subnet list                     # List all configured subnets
./target/release/ipc-cli subnet list --parent <ID>       # List subnets under parent
./target/release/ipc-cli subnet list-validators --subnet <ID>  # List validators
cast code <ADDRESS> --rpc-url <RPC>                     # Check if contract exists
cast call <ADDRESS> "totalSubnets()" --rpc-url <RPC>    # Check gateway registration count
cast logs --address <ADDRESS> --rpc-url <RPC>           # Check contract events
```

## Expected vs Actual State Analysis

When troubleshooting, compare:

1. **Config State**: What's in `~/.ipc/config.toml`
2. **Blockchain State**: What's actually deployed and registered on-chain
3. **UI State**: What the UI displays

The root cause is usually a mismatch between these three states.