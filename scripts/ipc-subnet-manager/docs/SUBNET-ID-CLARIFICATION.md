# Subnet ID Display Clarification

## Understanding IPC Subnet IDs

### Subnet ID Format
IPC subnet IDs follow a hierarchical format:
```
/r<parent_chain_id>/t<subnet_identifier>
```

### Your Configuration

**Subnet ID:** `/r31337/t410fwwa2cznrfkmmokgoc3m6xief6qrczcpxidsq4ia`

Breaking this down:
- `/r31337` - Parent chain (Anvil with chain ID 31337)
- `/t410fwwa2cznrfkmmokgoc3m6xief6qrczcpxidsq4ia` - Your actual subnet identifier

**Parent Chain:** `/r31337`
- This is the Anvil local testnet (chain ID 31337)
- Your subnet is deployed as a child of this chain

### What the Info Command Shows

```
Network Configuration:
  Subnet ID: /r31337/t410fwwa2cznrfkmmokgoc3m6xief6qrczcpxidsq4ia
  Parent Chain: /r31337
  Parent Registry: 0x01c1def3b91672704716159c9041aeca392ddffb
  Parent Gateway: 0x32eece76c2c2e8758584a83ee2f522d4788fea0f
```

### Clarification

**Q: Is the subnet ID just "31337"?**
**A:** No! The full subnet ID is `/r31337/t410fwwa2cznrfkmmokgoc3m6xief6qrczcpxidsq4ia`

- `31337` is the parent chain ID (Anvil)
- `t410fwwa2cznrfkmmokgoc3m6xief6qrczcpxidsq4ia` is your unique subnet identifier
- Together they form the complete hierarchical subnet ID

### Why This Matters

The hierarchical ID structure allows:
1. **Chain Identification** - Know which parent chain the subnet belongs to
2. **Unique Addressing** - Each subnet has a unique identifier within its parent
3. **Cross-Chain Messaging** - Route messages between parent and child subnets
4. **Multi-Level Hierarchies** - Subnets can have their own child subnets

### Example Hierarchy

```
/r31337 (Anvil - Root)
  └─ /r31337/t410fwwa2cznrfkmmokgoc3m6xief6qrczcpxidsq4ia (Your Subnet)
       └─ /r31337/t410fwwa2cznrfkmmokgoc3m6xief6qrczcpxidsq4ia/t<another_id> (Potential Child Subnet)
```

### Fix Applied

**Before:**
```
Parent Subnet: null  # Confusing - was trying to read non-existent field
```

**After:**
```
Parent Chain: /r31337  # Clear - shows the parent chain ID
```

The display now correctly shows:
- **Subnet ID** - Your complete subnet identifier
- **Parent Chain** - The chain your subnet is deployed on (Anvil in this case)

## Verification

To verify your subnet ID is correct:

```bash
# Check config file
yq eval '.subnet.id' ipc-subnet-config-local.yml

# Check IPC CLI config
cat ~/.ipc/config.toml | grep -A 5 "id = "

# View in info command
./ipc-manager --config ipc-subnet-config-local.yml info
```

All three should show the same complete subnet ID: `/r31337/t410fwwa2cznrfkmmokgoc3m6xief6qrczcpxidsq4ia`
