# Subnet Deployment Feature

## Overview

The IPC Subnet Manager now includes automatic subnet deployment functionality that runs `ipc-cli subnet init` before initializing validator nodes. This deploys the gateway contracts, creates the subnet on-chain, and generates genesis files automatically.

## What This Solves

Previously, the script would fail with errors like:
```
[ERROR] Initialization failed for validator-0
Error: failed to open file `null`: No such file or directory (os error 2)
```

This happened because the script tried to initialize nodes before the subnet actually existed on the parent chain. Now, the subnet is deployed first.

## Implementation

### New Function: `deploy_subnet()`

Location: `lib/health.sh`

This function:
1. Generates a `subnet-init.yaml` configuration from your existing config
2. Runs `ipc-cli subnet init --config subnet-init.yaml`
3. Deploys gateway and registry contracts on the parent chain
4. Creates the subnet on-chain
5. Generates genesis files in `~/.ipc/`
6. Extracts the subnet ID from the output
7. Updates your config file with the actual subnet ID

### Configuration Options

In your config file (e.g., `ipc-subnet-config-local.yml`):

```yaml
init:
  # Enable automatic subnet deployment
  deploy_subnet: true

  # Minimum number of validators
  min_validators: 3

  # Permission mode (federated, collateral, or static)
  permission_mode: "federated"

  # Supply source (native or ERC20)
  subnet_supply_source_kind: "native"

  # Genesis settings
  genesis:
    base_fee: "1000"
    power_scale: 3
    network_version: 21
```

### Workflow Changes

**Before:**
```
1. Update IPC CLI configs
2. Initialize primary node ← FAILED HERE
3. Initialize secondary nodes
...
```

**After:**
```
1. Update IPC CLI configs
2. Deploy subnet and gateway contracts ← NEW STEP
3. Initialize primary node ← Now works!
4. Initialize secondary nodes
...
```

## Usage

### First Time Setup

1. Make sure Anvil is running (in local mode):
   ```bash
   anvil --port 8545
   ```

2. Verify your config has the new settings:
   ```yaml
   init:
     deploy_subnet: true
     min_validators: 3
     permission_mode: "federated"
     subnet_supply_source_kind: "native"
   ```

3. Run the initialization:
   ```bash
   ./ipc-subnet-manager.sh init --config ipc-subnet-config-local.yml
   ```

4. The script will:
   - ✅ Deploy gateway contracts to Anvil
   - ✅ Create the subnet on-chain
   - ✅ Generate genesis files
   - ✅ Update your config with the real subnet ID
   - ✅ Initialize all validator nodes
   - ✅ Start the subnet

### Debug Mode

To see detailed output from the subnet deployment:

```bash
./ipc-subnet-manager.sh init --config ipc-subnet-config-local.yml --debug
```

This will show:
- The generated `subnet-init.yaml` configuration
- Real-time output from `ipc-cli subnet init`
- Contract deployment addresses
- Genesis file locations

### Skipping Subnet Deployment

If you already have a subnet deployed and just want to initialize nodes:

```yaml
init:
  deploy_subnet: false  # Skip deployment
```

The script will use the existing `subnet.id` from your config.

## What Gets Deployed

When `deploy_subnet: true`:

1. **Gateway Diamond Contract** - Manages cross-subnet messaging
2. **Registry Diamond Contract** - Tracks subnet registrations
3. **Subnet Actor** - The on-chain subnet instance
4. **Genesis Files** - In `~/.ipc/`:
   - `genesis_<subnet_id>.car`
   - `genesis_sealed_<subnet_id>.car`

## Address Mapping

The function automatically maps known Anvil test account private keys to their addresses:

| Private Key (last 4 chars) | Address |
|----------------------------|---------|
| `...2ff80` | `0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266` |
| `...8690d` | `0x70997970C51812dc3A010C7d01b50e0d17dc79C8` |
| `...ab365a` | `0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC` |

For custom addresses, add an `address` field to your validator config:

```yaml
validators:
  - name: "validator-0"
    private_key: "0x..."
    address: "0x..."  # Add this
```

## Troubleshooting

### Subnet deployment fails

**Check Anvil is running:**
```bash
lsof -i :8545
```

**Check logs:**
```bash
./ipc-subnet-manager.sh init --debug
```

### Cannot extract subnet ID

The script looks for subnet IDs in the format `/r<chain_id>/t<address>`.

Make sure the deployment succeeded and check the full output with `--debug`.

### Wrong contract addresses

The parent gateway and registry addresses are taken from your config:
```yaml
subnet:
  parent_registry: "0x74539671a1d2f1c8f200826baba665179f53a1b7"
  parent_gateway: "0x77aa40b105843728088c0132e43fc44348881da8"
```

These should match what's deployed on your parent chain (Anvil).

## Files Modified

- `lib/health.sh` - Added `deploy_subnet()` function
- `ipc-subnet-manager.sh` - Added subnet deployment step
- `ipc-subnet-config-local.yml` - Added `init.deploy_subnet` flag

## Example Output

```
>>> Deploying Subnet and Gateway Contracts

[INFO] Deploying subnet with gateway contracts...
[INFO] Generating subnet-init.yaml configuration...
[INFO] Running ipc-cli subnet init...
[INFO] This will deploy gateway contracts, create the subnet, and generate genesis files...
[INFO] Subnet init completed. Output summary:
Deployed Gateway: 0x77aa40b105843728088c0132e43fc44348881da8
Deployed Registry: 0x74539671a1d2f1c8f200826baba665179f53a1b7
Created subnet: /r31337/t410fkzrz3mlkyufisiuae3scumllgalzuu3wxlxa2ly
[SUCCESS] Subnet deployed successfully: /r31337/t410fkzrz3mlkyufisiuae3scumllgalzuu3wxlxa2ly
[INFO] Updating configuration with new subnet ID...
[INFO] Reading deployed contract addresses from IPC config...
[INFO] ✅ Subnet deployment complete!
[INFO]    Subnet ID: /r31337/t410fkzrz3mlkyufisiuae3scumllgalzuu3wxlxa2ly
[INFO]    Genesis files generated in ~/.ipc/
[INFO]    IPC config updated at ~/.ipc/config.toml
```

## Next Steps

After subnet deployment, the script continues with:
1. Node initialization (using the deployed subnet)
2. Peer discovery
3. Configuration updates
4. Node startup
5. Federated power setup (if applicable)

Everything should now work end-to-end!

