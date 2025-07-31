# IPC Subnet End-to-End Flow

This document provides a comprehensive guide to the complete IPC subnet lifecycle, from subnet creation to validator operation. It covers both the subnet creator's and validator's perspectives, explaining how each step can be run separately and how the workflow adapts for different subnet types.

---

## Overview

The IPC subnet lifecycle consists of several phases that can be executed either as a single automated workflow or as separate manual steps:

1. **Subnet Creation** - Deploy contracts and create the subnet
2. **Subnet Activation** - Activate the subnet with validators
3. **Genesis Generation** - Create genesis files for the subnet
4. **Validator Setup** - Initialize and start validator nodes
5. **Network Operation** - Maintain and monitor the running subnet

---

## Key Concepts

### Roles and Responsibilities

- **Subnet Creator/Admin**: Creates the subnet, manages activation, and provides configuration files to validators
- **Validators**: Join the subnet, run nodes, and participate in consensus

### Subnet Types

- **Federated**: Pre-defined set of validators with fixed power distribution
- **Static**: Similar to federated but with different activation requirements
- **Collateral**: Validators must stake FIL to join, with dynamic validator set

### Independent Steps

Each phase can be executed independently using separate commands:

- `ipc-cli subnet deploy` - Deploy contracts
- `ipc-cli subnet create` - Create subnet
- `ipc-cli subnet set-federated-power` - Activate federated/static subnet
- `ipc-cli subnet join` - Join collateral subnet
- `ipc-cli subnet create-genesis` - Generate genesis
- `ipc-cli node init` - Initialize validator node
- `ipc-cli node start` - Start validator node

---

## Phase 1: Subnet Creation (Subnet Creator)

### Option A: Automated Workflow (Recommended)

Use the `subnet init` command for a complete automated setup:

```sh
# Create subnet-init.yaml configuration file
cat > subnet-init.yaml << EOF
import-wallets:
  - wallet-type: evm
    private-key: 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80

deploy:
  enabled: true
  url: http://localhost:8545
  from: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
  chain-id: 31337

create:
  parent: /r314159
  from: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
  min-validator-stake: 1.0
  min-validators: 3
  bottomup-check-period: 50
  permission-mode: federated
  supply-source-kind: native
  genesis-subnet-ipc-contracts-owner: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266

activate:
  mode: federated
  validator-pubkeys:
    - 0x048318535b54105d4a7aae60c08fc45f9687181b4fdfc625bd1a753fa7397fed753547f11ca8696646f2f3acb08e31016afac23e630c5d11f59f61fef57b0d2aa5
    - 0x04ba5734d8f7091719471e7f7ed6b9df170dc70cc661ca05e688601ad984f068b0d67351e5f06073092499336ab0839ef8a521afd334e53807205fa2f08eec74f4
    - 0x049d9031e97dd78ff8c15aa86939de9b1e791066a0224e331bc962a2099a7b1f0464b8bbafe1535f2301c72c2cb3535b172da30b02686ab0393d348614f157fbdb
  validator-power:
    - 1
    - 1
    - 1

genesis:
  base-fee: 1000
  power-scale: 0
EOF

# Run the complete subnet initialization
ipc-cli subnet init --config subnet-init.yaml
```

### Option B: Manual Step-by-Step

Execute each step individually for more control:

#### For Federated/Static Subnets

```sh
# 1. Deploy contracts
ipc-cli subnet deploy \
  --url http://localhost:8545 \
  --from 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 \
  --chain-id 31337

# 2. Create subnet
ipc-cli subnet create \
  --parent /r314159 \
  --from 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 \
  --min-validator-stake 1.0 \
  --min-validators 3 \
  --bottomup-check-period 50 \
  --permission-mode federated \
  --supply-source-kind native \
  --genesis-subnet-ipc-contracts-owner 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266

# 3. Set federated power (activates the subnet)
ipc-cli subnet set-federated-power \
  --subnet /r314159/t410fkzrz3mlkyufisiuae3scumllgalzuu3wxlxa2ly \
  --validator-pubkeys 0x048318535b54105d4a7aae60c08fc45f9687181b4fdfc625bd1a753fa7397fed753547f11ca8696646f2f3acb08e31016afac23e630c5d11f59f61fef57b0d2aa5,0x04ba5734d8f7091719471e7f7ed6b9df170dc70cc661ca05e688601ad984f068b0d67351e5f06073092499336ab0839ef8a521afd334e53807205fa2f08eec74f4,0x049d9031e97dd78ff8c15aa86939de9b1e791066a0224e331bc962a2099a7b1f0464b8bbafe1535f2301c72c2cb3535b172da30b02686ab0393d348614f157fbdb \
  --validator-power 1,1,1

# 4. Generate genesis
ipc-cli subnet create-genesis \
  --subnet /r314159/t410fkzrz3mlkyufisiuae3scumllgalzuu3wxlxa2ly \
  --base-fee 1000 \
  --power-scale 0
```

#### For Collateral Subnets

```sh
# 1. Deploy contracts
ipc-cli subnet deploy \
  --url http://localhost:8545 \
  --from 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 \
  --chain-id 31337

# 2. Create subnet
ipc-cli subnet create \
  --parent /r314159 \
  --from 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 \
  --min-validator-stake 10.0 \
  --min-validators 3 \
  --bottomup-check-period 50 \
  --permission-mode collateral \
  --supply-source-kind native \
  --genesis-subnet-ipc-contracts-owner 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266

# 3. Validators join individually (activates when enough join with sufficient collateral)
ipc-cli subnet join \
  --subnet /r314159/t410fkzrz3mlkyufisiuae3scumllgalzuu3wxlxa2ly \
  --from 0x1234567890123456789012345678901234567890 \
  --collateral 10.0 \
  --initial-balance 50.0

ipc-cli subnet join \
  --subnet /r314159/t410fkzrz3mlkyufisiuae3scumllgalzuu3wxlxa2ly \
  --from 0x2345678901234567890123456789012345678901 \
  --collateral 10.0 \
  --initial-balance 50.0

ipc-cli subnet join \
  --subnet /r314159/t410fkzrz3mlkyufisiuae3scumllgalzuu3wxlxa2ly \
  --from 0x3456789012345678901234567890123456789012 \
  --collateral 10.0 \
  --initial-balance 50.0

# 4. Generate genesis (after enough validators have joined)
ipc-cli subnet create-genesis \
  --subnet /r314159/t410fkzrz3mlkyufisiuae3scumllgalzuu3wxlxa2ly \
  --base-fee 1000 \
  --power-scale 0
```

### Generated Files

After subnet creation, the following files are generated in `~/.ipc/`:

- **`node_SUBNET_ID.yaml`** - Complete node configuration for validators
- **`subnet-SUBNET_ID.json`** - Subnet information and metadata
- **`genesis_SUBNET_ID.car`** - Genesis file (if activated)
- **`genesis_sealed_SUBNET_ID.car`** - Sealed genesis file (if activated)

---

## Phase 2: Validator Setup

### For Federated/Static Subnets

Validators receive the generated `node_SUBNET_ID.yaml` file from the subnet creator and can immediately set up their nodes:

```sh
# 1. Initialize node using the provided configuration
ipc-cli node init --config node_r314159_t410fkzrz3mlkyufisiuae3scumllgalzuu3wxlxa2ly.yaml

# 2. Start the node
ipc-cli node start --home ~/.node-ipc
```

### For Collateral-Based Subnets

Validators must first join the subnet before setting up their nodes:

```sh
# 1. Join the subnet (required before node init)
ipc-cli subnet join \
  --subnet /r314159/t410fkzrz3mlkyufisiuae3scumllgalzuu3wxlxa2ly \
  --from 0x1234567890123456789012345678901234567890 \
  --collateral 10.0 \
  --initial-balance 50.0

# 2. Initialize node using the provided configuration
ipc-cli node init --config node_r314159_t410fkzrz3mlkyufisiuae3scumllgalzuu3wxlxa2ly.yaml

# 3. Start the node
ipc-cli node start --home ~/.node-ipc
```

---

## Smart Genesis Configuration

The generated `node.yaml` file includes intelligent genesis configuration that adapts based on the subnet's activation status:

### Activated Subnets

```yaml
genesis: !path
  genesis: "~/.ipc/genesis_r314159_t410fkzrz3mlkyufisiuae3scumllgalzuu3wxlxa2ly.car"
  sealed: "~/.ipc/genesis_sealed_r314159_t410fkzrz3mlkyufisiuae3scumllgalzuu3wxlxa2ly.car"
```

### Non-Activated Subnets

```yaml
genesis: !create
  network-version: 21
  base-fee: "1000"
  power-scale: 3
```

This means:

- **Activated subnets**: Validators use the existing genesis file created during subnet activation
- **Non-activated subnets**: Validators create their own genesis with sensible defaults

---

## Genesis Creation Timing

### When Genesis is Created

Genesis can be created at different times depending on the subnet type and activation mode:

1. **During Subnet Activation** (Federated/Static): Genesis is created automatically when `set-federated-power` is called
2. **After Validator Joining** (Collateral): Genesis is created after enough validators have joined and staked sufficient collateral
3. **Manual Creation**: Genesis can be created manually using `ipc-cli subnet create-genesis`

### Collateral-Based Subnet Workflow

For collateral-based subnets, the workflow is:

1. **Subnet Creator**: Creates subnet without activation
2. **Validators**: Join the subnet individually using `ipc-cli subnet join`
3. **Genesis Creation**: Once enough validators have joined with sufficient stake, genesis is created
4. **Node Setup**: Validators can then initialize and start their nodes

### First Validator Setup

The first set of validators in a collateral-based subnet need to:

1. **Join the subnet** using `ipc-cli subnet join`
2. **Wait for genesis creation** (either automatic or manual)
3. **Initialize their nodes** using the generated `node.yaml`
4. **Start their nodes** using `ipc-cli node start`

---

## Logs and Monitoring

### Log Locations

All logs are stored in the node's home directory:

```
~/.node-ipc/
├── logs/
│   ├── node.log          # General node logs
│   ├── fendermint.log    # Fendermint application logs
│   └── cometbft.log      # CometBFT consensus logs
└── cometbft/
    └── data/
        └── logs/         # Additional CometBFT logs
```

### Monitoring Commands

```sh
# Monitor node logs
tail -f ~/.node-ipc/logs/node.log

# Monitor Fendermint logs
tail -f ~/.node-ipc/logs/fendermint.log

# Monitor CometBFT logs
tail -f ~/.node-ipc/logs/cometbft.log

# Check node status
curl http://localhost:26657/status

# Check ETH API
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'
```

---

## Configuration File Sharing

### What to Share

The subnet creator should share the following files with validators:

1. **`node_SUBNET_ID.yaml`** - Complete node configuration (most important)
2. **`subnet-SUBNET_ID.json`** - Subnet information for reference
3. **Genesis files** (if applicable) - For activated subnets

### What NOT to Share

- **Private keys** - Each validator should generate their own
- **Sensitive configuration** - Keep internal network details private
- **Admin credentials** - Subnet creator credentials should remain private

### File Distribution

```sh
# Example: Share configuration files
cp ~/.ipc/node_r314159_t410fkzrz3mlkyufisiuae3scumllgalzuu3wxlxa2ly.yaml /shared/config/
cp ~/.ipc/subnet-r314159_t410fkzrz3mlkyufisiuae3scumllgalzuu3wxlxa2ly.json /shared/config/

# Validators download and use
wget https://example.com/config/node_r314159_t410fkzrz3mlkyufisiuae3scumllgalzuu3wxlxa2ly.yaml
```

---

## Troubleshooting Common Issues

### Subnet Creation Issues

**Contract Deployment Fails**

```sh
# Check if contracts are already deployed
ipc-cli subnet deploy --check-only

# Verify network connectivity
curl http://localhost:8545
```

**Subnet Creation Fails**

```sh
# Check subnet parameters
ipc-cli subnet create --dry-run

# Verify parent subnet exists
ipc-cli subnet list --parent /r314159
```

### Validator Setup Issues

**Node Init Fails**

```sh
# Check configuration file
cat node_r314159_t410fkzrz3mlkyufisiuae3scumllgalzuu3wxlxa2ly.yaml

# Verify subnet exists
ipc-cli subnet info --subnet /r314159/t410fkzrz3mlkyufisiuae3scumllgalzuu3wxlxa2ly
```

**Node Start Fails**

```sh
# Check if node is already running
ps aux | grep fendermint

# Check port conflicts
netstat -tulpn | grep :26656

# Check logs
tail -f ~/.node-ipc/logs/node.log
```

### Collateral Subnet Issues

**Join Fails**

```sh
# Check subnet status
ipc-cli subnet info --subnet /r314159/t410fkzrz3mlkyufisiuae3scumllgalzuu3wxlxa2ly

# Verify sufficient balance
ipc-cli wallet balance --address 0x1234567890123456789012345678901234567890
```

**Genesis Not Created**

```sh
# Check validator count and stake
ipc-cli subnet validators --subnet /r314159/t410fkzrz3mlkyufisiuae3scumllgalzuu3wxlxa2ly

# Manually create genesis if needed
ipc-cli subnet create-genesis --subnet /r314159/t410fkzrz3mlkyufisiuae3scumllgalzuu3wxlxa2ly
```

---

## Best Practices

### For Subnet Creators

1. **Test Configuration**: Test subnet creation on testnet before mainnet
2. **Document Parameters**: Keep detailed records of subnet parameters
3. **Secure Key Management**: Use secure key management practices
4. **Validator Communication**: Establish clear communication channels with validators
5. **Backup Configuration**: Keep backups of all configuration files

### For Validators

1. **Verify Configuration**: Double-check all configuration parameters
2. **Test Setup**: Test node setup on testnet first
3. **Monitor Resources**: Monitor CPU, memory, and disk usage
4. **Regular Backups**: Regularly backup configuration and keys
5. **Stay Updated**: Keep up with subnet updates and announcements

### Security Considerations

1. **Private Key Security**: Never share private keys
2. **Network Security**: Use firewalls and secure network configurations
3. **Access Control**: Limit access to node directories
4. **Regular Updates**: Keep software and dependencies updated
5. **Monitoring**: Monitor for suspicious activity

---

## Summary

The IPC subnet lifecycle provides flexibility for both automated and manual workflows:

- **Automated**: Use `subnet init` for complete setup with generated configuration files
- **Manual**: Execute individual commands for maximum control
- **Hybrid**: Combine automated and manual steps as needed

Key points to remember:

1. **Each step can be run separately** - You're not locked into the automated workflow
2. **Subnet creator creates, validators run** - Clear separation of responsibilities
3. **Configuration files simplify validator setup** - No need for manual configuration
4. **Logs are created in the node home directory** - Easy to find and monitor
5. **Genesis timing depends on subnet type** - Collateral subnets require validator joining first
6. **First validators in collateral mode need manual join** - Before genesis creation
7. **Activation commands differ by subnet type**:
   - Federated/Static: Use `ipc-cli subnet set-federated-power`
   - Collateral: Use `ipc-cli subnet join` (multiple validators must join with sufficient collateral)

This workflow ensures that subnet creation is accessible to administrators while making validator setup as simple as possible.
