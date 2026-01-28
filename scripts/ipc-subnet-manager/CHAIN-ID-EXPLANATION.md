# Chain ID vs Subnet ID Explanation

## Current Observation

When querying your subnet's `eth_chainId`, it returns **31337** (0x7a69), which is the same as the parent chain (Anvil).

```
Chain IDs:
  Parent Chain ID: 31337 (from config: /r31337)
  Subnet eth_chainId: 0x7a69 (decimal: 31337)
```

## Understanding the Difference

### Subnet ID (IPC-specific)
- **Format:** `/r31337/t410fwwa2cznrfkmmokgoc3m6xief6qrczcpxidsq4ia`
- **Purpose:** Hierarchical addressing for IPC cross-chain messaging
- **Components:**
  - `/r31337` - Parent chain identifier
  - `/t410fwwa2cznrfkmmokgoc3m6xief6qrczcpxidsq4ia` - Unique subnet identifier
- **Used for:** IPC protocol operations (cross-chain messages, finality, etc.)

### eth_chainId (EVM-specific)
- **Format:** `31337` (0x7a69)
- **Purpose:** EVM chain identification for transactions and wallets
- **Used for:** Ethereum RPC calls, MetaMask, transaction signing

## Why Are They The Same?

There are a few possible explanations:

### 1. Expected Behavior for Local Development
In local/test environments, subnets might inherit the parent's chain ID for simplicity. This allows:
- Using the same wallet configuration
- Simplified testing without reconfiguring MetaMask
- Easier development workflow

### 2. Configuration Option
The subnet's EVM chain ID might be configurable during deployment. Check if there's a setting in the genesis or init configuration.

### 3. Derived from Subnet ID
Some IPC implementations derive the EVM chain ID from the subnet ID hash. The `t410f...` part might be used to calculate a unique chain ID.

## What This Means for Your Setup

### Current State
- **Subnet ID:** `/r31337/t410fwwa2cznrfkmmokgoc3m6xief6qrczcpxidsq4ia` ✅ Unique
- **Parent Chain ID:** `31337` ✅ Correct
- **Subnet eth_chainId:** `31337` ⚠️ Same as parent

### Implications

**Pros:**
- ✅ Simpler wallet configuration
- ✅ Same MetaMask network works for both
- ✅ Easier local development

**Cons:**
- ⚠️ Potential confusion between parent and subnet
- ⚠️ May cause issues with some tools that rely on unique chain IDs
- ⚠️ Transactions might be replayed between chains (if not prevented by other means)

## Verification

### Check if this is intentional:

1. **Check genesis configuration:**
```bash
cat ~/.ipc/genesis.json | jq '.chain_id'
```

2. **Check fendermint config:**
```bash
cat ~/.ipc-local/validator-0/fendermint/config/default.toml | grep chain
```

3. **Query via RPC:**
```bash
curl -s -X POST -H "Content-Type: application/json" \
  --data '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' \
  http://localhost:8546 | jq -r '.result'
```

## Recommendation

For production deployments, subnets should typically have unique chain IDs to:
- Prevent transaction replay attacks
- Enable proper wallet/tool integration
- Maintain clear separation between chains

For local development (like your current setup), using the same chain ID is often acceptable and simplifies testing.

## Updated Info Display

The info command now clearly shows both:

```
Chain IDs:
  Parent Chain ID: 31337 (from config: /r31337)
  Subnet eth_chainId: 0x7a69 (decimal: 31337)
```

This makes it clear:
1. What the parent chain ID is (from config)
2. What the subnet's actual EVM chain ID is (from RPC query)
3. Whether they're the same or different

## Next Steps

If you want the subnet to have a unique chain ID:

1. Check the IPC documentation for chain ID configuration
2. Look for genesis parameters during subnet initialization
3. Consider if this is necessary for your use case (local dev vs production)

For now, the display clearly shows both values so you can see what's configured.
