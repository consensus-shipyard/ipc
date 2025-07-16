# Subnet Init Configuration

Bootstraps a new child subnet end-to-end from a YAML spec.

> **Note:** While each of the underlying steps (deploy, create, activate, and genesis) can be invoked manually via their respective subcommands, `subnet init` provides a convenient declarative workflow.

---

## Usage

```sh
ipc-cli subnet init --config <path/to/subnet-init.yaml>
```

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
