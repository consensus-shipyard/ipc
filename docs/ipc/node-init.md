# Node Init Configuration

Initializes a new CometBFT+Fendermint node from a YAML specification.

> **Note:** This command sets up a complete node environment including CometBFT and Fendermint configurations, validator keys, and genesis state.

---

## Usage

```sh
ipc-cli node init --config <path/to/node.yaml>
```

---

## YAML Configuration Schema

Top-level keys in `node.yaml`:

| Key                    | Type               | Required? | Description                                                                       |
| ---------------------- | ------------------ | --------- | --------------------------------------------------------------------------------- |
| `home`                 | `string`           | **Yes**   | Path to the node's home directory.                                                |
| `subnet`               | `string`           | **Yes**   | Subnet ID to join (e.g., `/r31337/t410fkzrz3mlkyufisiuae3scumllgalzuu3wxlxa2ly`). |
| `parent`               | `string`           | **Yes**   | Parent subnet ID (e.g., `/r31337`).                                               |
| `key`                  | `WalletImportArgs` | **Yes**   | Validator key configuration.                                                      |
| `cometbft-overrides`   | `string`           | No        | TOML overrides for CometBFT configuration.                                        |
| `fendermint-overrides` | `string`           | No        | TOML overrides for Fendermint configuration.                                      |
| `join`                 | `JoinConfig`       | No        | Join configuration for collateral-mode subnets.                                   |
| `genesis`              | `GenesisSource`    | **Yes**   | Genesis state configuration.                                                      |

---

### home

Path to the node's home directory where all configuration files, data, and keys will be stored.

**Example:**

```yaml
home: "/Users/karlem/.node-ipc"
```

---

### subnet

The subnet ID that this node will join. Must be a valid subnet identifier.

**Example:**

```yaml
subnet: "/r31337/t410fkzrz3mlkyufisiuae3scumllgalzuu3wxlxa2ly"
```

---

### parent

The parent subnet ID. Used for genesis creation and network hierarchy.

**Example:**

```yaml
parent: "/r31337"
```

---

### key

Validator key configuration. Must be a valid `WalletImportArgs` structure.

| Field         | Type     | Required?    | Description                                                |
| ------------- | -------- | ------------ | ---------------------------------------------------------- |
| `wallet-type` | `string` | Yes          | Wallet type (`evm`, `fvm`, etc.)                           |
| `path`        | `string` | one of group | Path to a key file (mutually exclusive with `private-key`) |
| `private-key` | `string` | one of group | EVM private key hex                                        |

**Example:**

```yaml
key:
  wallet-type: evm
  private-key: "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
```

---

### cometbft-overrides

Optional TOML configuration overrides for CometBFT. Uses YAML literal block syntax (`|`) to specify TOML content.

**Example:**

```yaml
cometbft-overrides: |
  [consensus]
  timeout_commit = "5s"

  [rpc]
  laddr = "tcp://0.0.0.0:26657"
```

**Common CometBFT Overrides:**

- `consensus.timeout_commit`: Block commit timeout
- `rpc.laddr`: RPC server address
- `p2p.laddr`: P2P server address
- `consensus.timeout_propose`: Block proposal timeout

---

### fendermint-overrides

Optional TOML configuration overrides for Fendermint. Uses YAML literal block syntax (`|`) to specify TOML content.

**Example:**

```yaml
fendermint-overrides: |
  [app]
  max_validators = 100

  [broadcast]
  gas_overestimation_rate = 2.0
```

**Common Fendermint Overrides:**

- `app.max_validators`: Maximum number of validators
- `broadcast.gas_overestimation_rate`: Gas estimation multiplier
- `broadcast.max_retries`: Maximum transaction retry attempts

---

### join

Join configuration for collateral-mode subnets. Only valid when the subnet uses collateral-based permission mode.

| Field             | Type     | Required? | Description                        |
| ----------------- | -------- | --------- | ---------------------------------- |
| `from`            | `string` | Yes       | Address joining the subnet.        |
| `collateral`      | `float`  | Yes       | FIL collateral to lock.            |
| `initial-balance` | `float`  | No        | Initial FIL balance on the subnet. |

**Example:**

```yaml
join:
  from: "/r31337/t410fobdhc7jsv7fekdof23tvojgufaw7vulodh2rj4a"
  collateral: 10
  initial-balance: 50
```

---

### genesis

Genesis state configuration. Can either create new genesis or use existing genesis file.

#### Create New Genesis

```yaml
genesis: !create
  base-fee: "1000"
  power-scale: 0
```

| Field         | Type      | Required?           | Description                        |
| ------------- | --------- | ------------------- | ---------------------------------- |
| `base-fee`    | `string`  | No (default `1000`) | Base transaction fee in attoFIL.   |
| `power-scale` | `integer` | No (default `3`)    | Decimals for FIL→Power conversion. |

#### Use Existing Genesis

```yaml
genesis: !path
  path: "/path/to/genesis.json"
```

---

## Example `node.yaml`

```yaml
home: "/Users/karlem/.node-ipc"
subnet: "/r31337/t410fkzrz3mlkyufisiuae3scumllgalzuu3wxlxa2ly"
parent: "/r31337"

key:
  wallet-type: evm
  private-key: "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"

# Optional TOML overrides for CometBFT configuration
cometbft-overrides: |
  [consensus]
  timeout_commit = "5s"

  [rpc]
  laddr = "tcp://0.0.0.0:26657"

# Optional TOML overrides for FenderMint configuration
fendermint-overrides: |
  [app]
  max_validators = 100

# JoinConfig — only valid for collateral‐mode subnets
# join:
#   from: "/r31337/t410fobdhc7jsv7fekdof23tvojgufaw7vulodh2rj4a"
#   collateral: 10
#   initial-balance: 50

# Genesis configuration
genesis: !create
  base-fee: "1000"
  power-scale: 0
```

---

## Generated Directory Structure

After running `node init`, the following directory structure is created:

```
<home>/
├── cometbft/
│   ├── config/
│   │   ├── config.toml          # CometBFT configuration (with overrides applied)
│   │   ├── genesis.json         # Genesis state
│   │   ├── node_key.json        # Node identity key
│   │   └── priv_validator_key.json  # Validator key
│   └── data/
│       └── priv_validator_state.json # Validator state
└── fendermint/
    ├── config/
    │   └── default.toml         # Fendermint configuration (with overrides applied)
    ├── genesis_r31337_t410fkzrz3mlkyufisiuae3scumllgalzuu3wxlxa2ly.json
    └── genesis_sealed_r31337_t410fkzrz3mlkyufisiuae3scumllgalzuu3wxlxa2ly.json
```

---

## Configuration Overrides

The `cometbft-overrides` and `fendermint-overrides` fields allow you to customize the default configurations without manually editing the generated files.

### How Overrides Work

1. **Default Configuration**: The system generates default configurations for both CometBFT and Fendermint
2. **Override Application**: Your TOML overrides are merged with the defaults using deep merge
3. **File Generation**: The final merged configuration is written to the respective config files

### Deep Merge Behavior

- **Tables**: Nested TOML tables are merged recursively
- **Values**: Leaf values are replaced with override values
- **Preservation**: Default values not specified in overrides are preserved

### Best Practices

- Use YAML literal blocks (`|`) for multi-line TOML content
- Test overrides with a small configuration first
- Keep overrides minimal - only override what you need to change
- Document your overrides for team consistency

---

## Troubleshooting

### Common Issues

**Invalid TOML Syntax**

```
Error: unsupported rust type
```

- Check that your TOML overrides use valid TOML syntax
- Ensure proper indentation in YAML literal blocks

**Configuration Not Applied**

- Verify that the override fields are correctly formatted
- Check the debug logs for override application status
- Ensure the config files exist in the expected locations

**Permission Errors**

- Ensure the home directory is writable
- Check file permissions on existing config files

### Debug Information

The command provides detailed logging about:

- Configuration parsing
- Override application
- File generation
- Genesis creation

Use `RUST_LOG=debug` for additional debug information:

```sh
RUST_LOG=debug ipc-cli node init --config node.yaml
```
