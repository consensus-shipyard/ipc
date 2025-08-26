# Querying Subnet Permission Mode

This guide documents how to determine whether an IPC subnet is in collateral mode or federated mode by querying the contracts directly.

## Overview

IPC subnets can operate in different permission modes:
- **0 = Collateral**: Validator power determined by staked collateral
- **1 = Federated**: Validator power assigned by subnet owner
- **2 = Static**: Validator power determined by initial collateral, doesn't change

## Problem Statement

Given a subnet ID like `/r31337/t410f5kakfhdd3amp56oqwpxbupfalxsbjjrqilas5my`, how do you determine its permission mode?

## Initial Approach: F410 Address Conversion

### Step 1: Extract F410 Address from Subnet ID

From subnet ID `/r31337/t410f5kakfhdd3amp56oqwpxbupfalxsbjjrqilas5my`:
- **Root Chain ID**: `31337`
- **F410 Address**: `t410f5kakfhdd3amp56oqwpxbupfalxsbjjrqilas5my`

### Step 2: Convert F410 to Ethereum Address

F410 addresses are Crockford base32-encoded delegated addresses containing 20-byte Ethereum addresses.

**Conversion Script** (Python):
```python
#!/usr/bin/env python3
import base64

def crockford_base32_decode(data):
    """Convert Crockford base32 to standard base32 and decode"""
    crockford_alphabet = "0123456789ABCDEFGHJKMNPQRSTVWXYZ"
    standard_alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567"
    translate_table = str.maketrans(crockford_alphabet, standard_alphabet)

    data_upper = data.upper()
    standard_data = data_upper.translate(translate_table)

    # Add padding
    while len(standard_data) % 8 != 0:
        standard_data += '='

    return base64.b32decode(standard_data)

def f410_to_eth_address(f410_addr):
    """Convert f410 address to Ethereum address"""
    # Remove t410 or f410 prefix
    without_prefix = f410_addr[4:]

    # Decode from Crockford base32
    decoded_bytes = crockford_base32_decode(without_prefix)

    # Extract last 20 bytes as Ethereum address
    if len(decoded_bytes) >= 20:
        eth_bytes = decoded_bytes[-20:]
        return '0x' + eth_bytes.hex()

    return None

# Example conversion
f410_address = 't410f5kakfhdd3amp56oqwpxbupfalxsbjjrqilas5my'
eth_address = f410_to_eth_address(f410_address)
print(f"Ethereum Address: {eth_address}")
# Output: 0x68d54b14cebf2dd5d2cf52fb95ca58ba16ac969e
```

### Step 3: Query the Contract (Initial Attempt)

```bash
cast call 0x68d54b14cebf2dd5d2cf52fb95ca58ba16ac969e "permissionMode()" --rpc-url http://localhost:8545
# Result: 0x (empty - contract doesn't exist)
```

**Problem**: The converted address didn't correspond to an actual deployed contract.

## Correct Approach: Query Gateway Contract

### Step 4: Use Gateway Contract from Configuration

From the IPC configuration file, identify the gateway contract address:
```toml
gateway_addr = "0x742489f22807ebb4c36ca6cd95c3e1c044b7b6c8"
```

### Step 5: List Registered Subnets

Query the gateway contract to get all registered subnets:

```bash
cast call 0x742489f22807ebb4c36ca6cd95c3e1c044b7b6c8 "listSubnets()" --rpc-url http://localhost:8545
```

**Result**: Returns encoded subnet data including the actual deployed contract address.

### Step 6: Extract Subnet Actor Address

From the `listSubnets()` response, extract the subnet actor contract address:
**Deployed Address**: `0xe5b6b170ab9a28c516b375465d11d77683a26550`

### Step 7: Query Permission Mode

Query the actual deployed subnet actor contract:

```bash
cast call 0xe5b6b170ab9a28c516b375465d11d77683a26550 "permissionMode()" --rpc-url http://localhost:8545
```

**Result**: `0x0000000000000000000000000000000000000000000000000000000000000001`

### Step 8: Convert Result

```bash
cast --to-dec 0x0000000000000000000000000000000000000000000000000000000000000001
# Result: 1 (Federated mode)
```

## Complete Working Process

### Prerequisites
- Access to the parent chain RPC endpoint
- Gateway contract address from configuration
- `cast` tool (from Foundry)

### Commands Summary

1. **Get gateway address from config**:
   ```bash
   # From your IPC config file
   gateway_addr = "0x742489f22807ebb4c36ca6cd95c3e1c044b7b6c8"
   ```

2. **List all registered subnets**:
   ```bash
   cast call 0x742489f22807ebb4c36ca6cd95c3e1c044b7b6c8 "listSubnets()" --rpc-url http://localhost:8545
   ```

3. **Extract subnet actor address from response** (manual step - parse the returned data)

4. **Query permission mode**:
   ```bash
   cast call [SUBNET_ACTOR_ADDRESS] "permissionMode()" --rpc-url [PARENT_CHAIN_RPC]
   ```

5. **Convert result**:
   ```bash
   cast --to-dec [HEX_RESULT]
   ```

### Example Complete Flow

```bash
# Step 1: Query gateway for registered subnets
cast call 0x742489f22807ebb4c36ca6cd95c3e1c044b7b6c8 "listSubnets()" --rpc-url http://localhost:8545

# Step 2: Extract subnet actor address (example result)
# Found: 0xe5b6b170ab9a28c516b375465d11d77683a26550

# Step 3: Query permission mode
cast call 0xe5b6b170ab9a28c516b375465d11d77683a26550 "permissionMode()" --rpc-url http://localhost:8545
# Returns: 0x0000000000000000000000000000000000000000000000000000000000000001

# Step 4: Convert to decimal
cast --to-dec 0x0000000000000000000000000000000000000000000000000000000000000001
# Returns: 1 (Federated mode)
```

## Alternative Methods

### Using Gateway's getSubnet Function

If you know the exact subnet ID structure:

```bash
# Query specific subnet by ID (requires proper encoding)
cast call 0x742489f22807ebb4c36ca6cd95c3e1c044b7b6c8 "getSubnet((uint64,address[]))" --rpc-url http://localhost:8545
```

### Using Registry Contract

Query the registry contract for deployed subnets by owner:

```bash
# Get latest subnet deployed by an owner
cast call 0x1d8d70ad07c8e7e442ad78e4ac0a16f958eba7f0 "latestSubnetDeployed(address)" [OWNER_ADDRESS] --rpc-url http://localhost:8545
```

## Key Insights

1. **Subnet actor contracts are deployed on the parent chain**, not on the subnet itself
2. **F410 addresses in subnet IDs don't directly correspond to deployed contract addresses**
3. **The gateway contract maintains the registry of actual deployed subnet actors**
4. **Only fully deployed and registered subnets appear in the gateway's subnet list**

## Troubleshooting

### Empty Results (`0x`)
- Contract doesn't exist at that address
- Wrong RPC endpoint
- Subnet not fully deployed

### Connection Errors
- Verify RPC endpoint is accessible
- Check if the chain is running
- Confirm chain ID matches

### Address Conversion Issues
- F410 conversion is for reference only
- Use gateway contract to find actual deployed addresses
- Deployment process may assign different addresses

## Contract ABIs

### Subnet Actor Getter Facet
```solidity
interface ISubnetActorGetterFacet {
    function permissionMode() external view returns (uint8);
    function getParent() external view returns (tuple(uint64, address[]));
}
```

### Gateway Getter Facet
```solidity
interface IGatewayGetterFacet {
    function listSubnets() external view returns (tuple[]);
    function getSubnet(tuple(uint64, address[])) external view returns (bool, tuple);
}
```

## Permission Mode Reference

| Value | Mode | Description |
|-------|------|-------------|
| 0 | Collateral | Validator power determined by staked collateral |
| 1 | Federated | Validator power assigned by subnet owner |
| 2 | Static | Validator power fixed at initial collateral amount |