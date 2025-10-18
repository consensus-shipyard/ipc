# Bottom-Up Checkpoint Broadcasting Fix

## 🎯 Problem

Validators were getting this error every ~10 blocks:
```
ERROR: error broadcasting checkpoint signature
failed to broadcast checkpoint signature
Caused by:
    0: failed to broadcast signature
    1: failed to get broadcaster sequence
    2: broadcaster actor t1k6ahqshczp3x75z4gpe6kk7wir4dqqovv23rg6a cannot be found
```

## 🔍 Root Cause Analysis

### Issue
The validators were configured with `AccountKind::Regular` which derives **`t1` (Filecoin native) addresses** from the validator secret keys. These addresses did not exist in the subnet state.

### Code Location
`fendermint/app/src/service/node.rs:490-496`:
```rust
fn to_address(sk: &SecretKey, kind: &AccountKind) -> anyhow::Result<Address> {
    let pk = sk.public_key().serialize();
    match kind {
        AccountKind::Regular => Ok(Address::new_secp256k1(&pk)?),     // ← Creates t1 address
        AccountKind::Ethereum => Ok(Address::from(EthAddress::new_secp256k1(&pk)?)),  // ← Creates f410/EVM address
    }
}
```

### Why It Failed
1. Validator config had: `kind = "regular"`
2. This created `t1` addresses for broadcasting checkpoint signatures
3. The `t1` addresses didn't exist in the subnet state (which uses EVM addresses)
4. Querying the actor state failed: `broadcaster actor t1k... cannot be found`
5. Checkpoint signatures couldn't be broadcast

## ✅ The Fix

### Change validator_key kind to "ethereum"

**File:** `~/.ipc-node/fendermint/config/default.toml`

```toml
[validator_key]
path = "validator.sk"
kind = "ethereum"  # Changed from "regular"
```

### Result
- **Before:** `t1k6ahqshczp3x75z4gpe6kk7wir4dqqovv23rg6a` (Filecoin native address - doesn't exist)
- **After:** `t410fhkdml7o5ewdyswlfs4hhbjp2f3cfvyf2ficvxtq` (EVM address - exists with balance)

## 🚀 Implementation

### Manual Fix (Applied to Running Subnet)

```bash
# Fix all validators
for ip in 34.73.187.192 35.237.175.224 34.75.205.89; do
    echo "Fixing $ip..."
    ssh philip@$ip "sudo su - ipc -c 'cd ~/.ipc-node/fendermint/config && \
        sed -i.bak-keyfix \"s/kind = \\\"regular\\\"/kind = \\\"ethereum\\\"/\" default.toml'"
done

# Restart validators
./ipc-manager restart --yes
```

### Automatic Fix (For Future Subnets)

Updated `lib/config.sh:379-383`:
```bash
  [validator_key]
  path = "validator.sk"
  # Use "ethereum" for EVM-based subnets (federated/collateral with EVM addresses)
  # Use "regular" only for native Filecoin address subnets
  kind = "ethereum"
```

## 📊 Verification

### Before Fix
```
ERROR: broadcaster actor t1k6ahqshczp3x75z4gpe6kk7wir4dqqovv23rg6a cannot be found
```
Occurred every ~10 blocks (checkpoint period)

### After Fix
```json
{
  "level": "INFO",
  "message": "validator key address: t410fhkdml7o5ewdyswlfs4hhbjp2f3cfvyf2ficvxtq detected"
}
{
  "level": "INFO",
  "message": "broadcasted signature",
  "tx_hash": "9268473A2BC803861AF418B4D351EC0958A493DCA2462C1E1D62FB191F3C7DB1"
}
{
  "level": "INFO",
  "message": "broadcasted signature",
  "tx_hash": "D43F97EFD7D66C6A280BE07DD5AEB0575588F8418FE0AAE902E13249DC35C9F3"
}
... (10+ successful broadcasts observed)
```

### Occasional Benign Errors
```
Internal error: tx already exists in cache (code: -32603)
```
This is a normal mempool collision when multiple validators submit similar transactions. Not critical.

## 🧪 Testing

### Verify Fix is Working
```bash
# Check validator is using t410 address
ssh philip@34.73.187.192 "sudo su - ipc -c 'grep \"validator key address\" ~/.ipc-node/logs/*.log | tail -1'"
# Should show: "validator key address: t410..."

# Check for successful signature broadcasts
ssh philip@34.73.187.192 "sudo su - ipc -c 'grep \"broadcasted signature\" ~/.ipc-node/logs/*.log | tail -10'"
# Should show multiple "broadcasted signature" with tx_hash

# Check for old errors
ssh philip@34.73.187.192 "sudo su - ipc -c 'grep \"broadcaster actor.*cannot be found\" ~/.ipc-node/logs/*.log | tail -1'"
# Should show no new errors (only old ones from before the fix)
```

## 📝 When to Use Each Kind

### Use "ethereum"
- ✅ Federated subnets with EVM addresses
- ✅ Collateral subnets using EVM
- ✅ Any subnet where validators use EVM private keys
- ✅ **Most common case**

### Use "regular"
- ⚠️ Native Filecoin address subnets
- ⚠️ Subnets not using EVM compatibility
- ⚠️ **Rare case**

## 🔧 Upstream Fix Needed

### In IPC Codebase

**File:** `ipc/cli/src/commands/node/init.rs` (or equivalent)

The `node init` command should:
1. Detect if the subnet is EVM-based (by checking genesis or subnet config)
2. Automatically set `validator_key.kind = "ethereum"` for EVM subnets
3. Only use `kind = "regular"` for native Filecoin subnets

**Suggested Implementation:**
```rust
// In node init logic
let validator_key_kind = if subnet_uses_evm_addresses(&subnet_id) {
    AccountKind::Ethereum  // For EVM subnets
} else {
    AccountKind::Regular   // For native Filecoin subnets
};
```

This would prevent users from encountering this issue in the first place.

## 📚 Related Issues

### Address Formats in IPC

| Format | Prefix | Use Case | Created By |
|--------|--------|----------|-----------|
| **t1** | `t1...` | Filecoin native secp256k1 | `AccountKind::Regular` |
| **t2** | `t2...` | Filecoin native actor address | N/A |
| **t3** | `t3...` | Filecoin native BLS | N/A |
| **t4** | `t4...` | Delegated address namespace | N/A |
| **f410** | `t410...` | EVM address (delegated to actor 10) | `AccountKind::Ethereum` |

### Key Derivation

Both `t1` and `t410` addresses are derived from the same secp256k1 secret key, but:
- **t1:** Direct secp256k1 public key hash (Filecoin native)
- **t410:** EVM-style address (keccak256 hash of public key, last 20 bytes)

## 🎯 Summary

- **Problem:** Validators using wrong address format for broadcasting
- **Cause:** `validator_key.kind = "regular"` instead of `"ethereum"`
- **Fix:** Change to `kind = "ethereum"` and restart
- **Result:** ✅ Bottom-up checkpointing now fully operational
- **Prevention:** Updated `ipc-subnet-manager` to use correct setting by default

---

**Fixed:** October 18, 2025
**Tested:** ✅ Verified with 10+ successful checkpoint signature broadcasts
**Status:** 🟢 Production Ready

