# Subnet Init Configuration

Bootstraps a new child subnet end-to-end from a YAML spec and generates ready-to-use configuration files for validators.

> **Note:** While each of the underlying steps (deploy, create, set-federated-power/join, and genesis) can be invoked manually via their respective subcommands, `subnet init` provides a convenient declarative workflow that also generates validator-ready configuration files.

---

## Usage

```sh
ipc-cli subnet init --config <path/to/subnet-init.yaml>
```

---

## What This Command Does

The `subnet init` command performs the following steps in sequence:

1. **Import Wallets** (optional): Import specified wallets into the IPC CLI keystore
2. **Deploy Contracts** (optional): Deploy gateway/registry contracts on the parent chain
3. **Create Subnet**: Create the subnet actor on the parent chain
4. **Activate Subnet** (optional): Activate the subnet with validators
   - **Federated/Static**: Use `set-federated-power` to set validator power
   - **Collateral**: Multiple validators join with sufficient collateral
5. **Generate Genesis** (optional): Create genesis files for the subnet
6. **Generate Configuration Files**: Create ready-to-use files for validators:
   - `node_SUBNET_ID.yaml` - Complete node configuration file
   - `subnet-SUBNET_ID.json` - Subnet information and metadata

---

## Generated Files

After successful execution, the following files are created in `~/.ipc/`:

### `node_SUBNET_ID.yaml`

A complete, ready-to-use node configuration file that validators can use with `ipc-cli node init`. This file includes:

- **Smart Genesis Configuration**:
  - If subnet is activated: Uses existing genesis file path
  - If subnet is NOT activated: Uses genesis creation configuration
- **P2P Configuration**: Basic external IP setup for user customization
- **Join Configuration**: For collateral-based subnets (if applicable)
- **All Required Fields**: Home directory, subnet ID, parent ID, validator key setup

### `subnet-SUBNET_ID.json`

Comprehensive subnet information including:

- **General Info**: Subnet ID, parent ID, creation timestamp, network name
- **Contract Addresses**: Gateway, registry, and parent contract addresses
- **Genesis Info**: File paths, network version, base fee, power scale
- **Activation Info**: Mode, validators, collateral amounts (if applicable)

---

## YAML Configuration Schema

Top-level keys in `subnet-init.yaml`:

| Key              | Type                 | Required? | Description                                                    |
| ---------------- | -------------------- | --------- | -------------------------------------------------------------- |
| `import-wallets` | `WalletImportArgs[]` | No        | Wallet(s) to import into the IPC CLI keystore.                 |
| `deploy`         | `DeployConfig`       | No        | Deploy gateway/registry contracts on the parent chain.         |
| `create`         | `SubnetCreateConfig` | **Yes**   | Create the subnet on-chain (parent, stakes, permission mode…). |
| `activate`       | `ActivateConfig`     | No        | Activate the subnet (federated, static, or collateral).        |
| `genesis`        | `GenesisConfig`      | No        | Genesis params (network version, base fee, power scale).       |

---

### import-wallets

Each entry is a `WalletImportArgs`:

| Field         | Type     | Required?    | Description                                                |
| ------------- | -------- | ------------ | ---------------------------------------------------------- |
| `wallet-type` | `string` | Yes          | Wallet type (`evm`, `fvm`, etc.)                           |
| `path`        | `string` | one of group | Path to a key file (mutually exclusive with `private-key`) |
| `private-key` | `string` | one of group | EVM private key hex                                        |

---

### deploy

`DeployConfig`:

| Field                       | Type      | Required?             | Description                                        |
| --------------------------- | --------- | --------------------- | -------------------------------------------------- |
| `enabled`                   | `boolean` | No (default: `false`) | Whether to run contract deployment.                |
| `url`                       | `string`  | Yes                   | Ethereum provider URL.                             |
| `from`                      | `string`  | Yes                   | Deployer address (must exist in keystore).         |
| `chain-id`                  | `integer` | Yes                   | Target Ethereum chain ID.                          |
| `artifacts-path`            | `string`  | No                    | Path to compiled contracts (defaults to built-in). |
| `subnet-creation-privilege` | `Enum`    | No                    | `Unrestricted` \| `Whitelisted` \| `Restricted`    |

---

### create

`SubnetCreateConfig`:

| Field                                | Type      | Required?               | Description                                              |
| ------------------------------------ | --------- | ----------------------- | -------------------------------------------------------- |
| `parent`                             | `string`  | Yes                     | Parent subnet namespace (e.g. `/r31337`).                |
| `from`                               | `string`  | No                      | Address creating the subnet (defaults to global sender). |
| `min-validator-stake`                | `float`   | Yes                     | Minimum collateral per validator (whole FIL).            |
| `min-validators`                     | `integer` | Yes                     | Minimum number of validators to bootstrap.               |
| `bottomup-check-period`              | `integer` | Yes                     | Bottom-up checkpoint period (in epochs).                 |
| `active-validators-limit`            | `integer` | No                      | Maximum active validators in the subnet.                 |
| `min-cross-msg-fee`                  | `float`   | No (default `0.000001`) | Minimum fee for cross-network messages (FIL).            |
| `permission-mode`                    | `Enum`    | Yes                     | `collateral` \| `federated` \| `static`.                 |
| `supply-source-kind`                 | `Enum`    | Yes                     | `native` \| `erc20`.                                     |
| `supply-source-address`              | `string`  | No                      | ERC-20 contract address (if `erc20`).                    |
| `validator-gater`                    | `string`  | No                      | Validator gating contract address.                       |
| `validator-rewarder`                 | `string`  | No                      | Validator rewarder contract address.                     |
| `collateral-source-kind`             | `Enum`    | No                      | `native` \| `erc20`.                                     |
| `collateral-source-address`          | `string`  | No                      | ERC-20 collateral contract address (if `erc20`).         |
| `genesis-subnet-ipc-contracts-owner` | `string`  | Yes                     | Subnet-local owner for IPC diamond contracts at genesis. |

---

### activate

Select one mode:

#### Federated / Static

```yaml
activate:
  mode: federated # or `static`
  validator-pubkeys:
    - <0x04...> # 65-byte uncompressed pubkey hex
    - …
  validator-power:
    - 1 # matching power values
    - …
```

- **validator-pubkeys**: array of `0x04`-prefixed pubkeys.
- **validator-power**: array of unsigned integers.

#### Collateral

```yaml
activate:
  mode: collateral
  validators:
    - from: <address>
      collateral: <float>
      initial-balance: <float?> # optional
    - …
```

Each entry is a `JoinConfig`:

- **from** (string): address joining.
- **collateral** (float): FIL to lock.
- **initial-balance** (float, optional): starting FIL balance on subnet.

---

### genesis

`GenesisConfig`:

| Field             | Type      | Required?           | Description                                   |
| ----------------- | --------- | ------------------- | --------------------------------------------- |
| `network-version` | `integer` | No (default `21`)   | Filecoin network version for built-in actors. |
| `base-fee`        | `integer` | No (default `1000`) | Base tx fee in attoFIL.                       |
| `power-scale`     | `integer` | No (default `3`)    | Decimals for FIL→Power conversion.            |

---

## Example `subnet-init.yaml`

```yaml
import-wallets:
  - wallet-type: evm
    private-key: 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80

deploy:
  enabled: false
  url: http://localhost:8545
  from: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
  chain-id: 31337

create:
  parent: /r31337
  from: 0xf39Fd6e51aad88F6F4ce6aA8827279cffFb92266
  min-validator-stake: 1.0
  min-validators: 3
  bottomup-check-period: 50
  permission-mode: federated
  supply-source-kind: native
  min-cross-msg-fee: 0.1
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
```

---

## Output Example

After running `subnet init`, you'll see output like:

```
✅ Subnet created successfully: /r31337/t410fkzrz3mlkyufisiuae3scumllgalzuu3wxlxa2ly
✅ Genesis files generated
📄 Node config saved to: node_r31337_t410fkzrz3mlkyufisiuae3scumllgalzuu3wxlxa2ly.yaml

📋 Subnet Information:
{
  "subnet_info": {
    "subnet_id": "/r31337/t410fkzrz3mlkyufisiuae3scumllgalzuu3wxlxa2ly",
    "parent_id": "/r31337",
    "name": null,
    "created_at": "2024-01-15T10:30:00Z",
    "network": "testnet"
  },
  "contracts": {
    "gateway_address": "0x1234...",
    "registry_address": "0x5678...",
    "parent_gateway": "0x9abc...",
    "parent_registry": "0xdef0..."
  },
  "genesis": {
    "genesis_path": "~/.ipc/genesis_r31337_t410fkzrz3mlkyufisiuae3scumllgalzuu3wxlxa2ly.car",
    "sealed_genesis_path": "~/.ipc/genesis_sealed_r31337_t410fkzrz3mlkyufisiuae3scumllgalzuu3wxlxa2ly.car",
    "network_version": 21,
    "base_fee": "1000",
    "power_scale": 0
  },
  "activation": {
    "mode": "federated",
    "validators": [
      {
        "address": "0x1234...",
        "public_key": "0x04...",
        "power": 1
      }
    ]
  }
}
```

---

## Workflow for Validators

After the subnet creator runs `subnet init`, validators can:

1. **Get Configuration Files**: The subnet creator shares the generated `node_SUBNET_ID.yaml` file
2. **Initialize Node**: Run `ipc-cli node init --config node_SUBNET_ID.yaml`
3. **Start Node**: Run `ipc-cli node start --home ~/.node-ipc`

For collateral-based subnets, validators must first join the subnet using `ipc-cli subnet join` before running `node init`.

---

## Manual Commands (Alternative to subnet init)

If you prefer to run steps manually instead of using `subnet init`:

### For Federated/Static Subnets

```sh
# 1. Deploy contracts (if needed)
ipc-cli subnet deploy --url http://localhost:8545 --from 0x... --chain-id 31337

# 2. Create subnet
ipc-cli subnet create --parent /r31337 --from 0x... --min-validator-stake 1.0 --min-validators 3 --permission-mode federated

# 3. Set federated power (activates the subnet)
ipc-cli subnet set-federated-power \
  --subnet /r31337/t410fkzrz3mlkyufisiuae3scumllgalzuu3wxlxa2ly \
  --validator-pubkeys 0x04...,0x04...,0x04... \
  --validator-power 1,1,1

# 4. Generate genesis
ipc-cli subnet create-genesis --subnet /r31337/t410fkzrz3mlkyufisiuae3scumllgalzuu3wxlxa2ly
```

### For Collateral Subnets

```sh
# 1. Deploy contracts (if needed)
ipc-cli subnet deploy --url http://localhost:8545 --from 0x... --chain-id 31337

# 2. Create subnet
ipc-cli subnet create --parent /r31337 --from 0x... --min-validator-stake 10.0 --min-validators 3 --permission-mode collateral

# 3. Validators join individually (activates when enough join with sufficient collateral)
ipc-cli subnet join --subnet /r31337/t410fkzrz3mlkyufisiuae3scumllgalzuu3wxlxa2ly --from 0x... --collateral 10.0
ipc-cli subnet join --subnet /r31337/t410fkzrz3mlkyufisiuae3scumllgalzuu3wxlxa2ly --from 0x... --collateral 10.0
ipc-cli subnet join --subnet /r31337/t410fkzrz3mlkyufisiuae3scumllgalzuu3wxlxa2ly --from 0x... --collateral 10.0

# 4. Generate genesis (after enough validators have joined)
ipc-cli subnet create-genesis --subnet /r31337/t410fkzrz3mlkyufisiuae3scumllgalzuu3wxlxa2ly
```

---

## Important Notes

- **Each Step is Independent**: You can run individual steps (deploy, create, set-federated-power/join, genesis) separately using their respective commands
- **Configuration Files**: Generated files are placed in `~/.ipc/` for easy access
- **Smart Genesis**: The generated `node.yaml` automatically uses the correct genesis configuration based on whether the subnet is activated
- **Validator Workflow**: Validators receive ready-to-use configuration files, making node setup much simpler
- **Activation Commands**:
  - Federated/Static: Use `ipc-cli subnet set-federated-power`
  - Collateral: Use `ipc-cli subnet join` (multiple validators must join with sufficient collateral)
