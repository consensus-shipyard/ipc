# IPC Scripts

This directory contains utility scripts for the IPC project.

## validate-subnet-deployment.sh

An automated script that validates all configured subnets in `~/.ipc/config.toml` and checks their deployment status on the blockchain.

### Features

- **Comprehensive Validation**: Checks 7 different aspects of subnet deployment:
  1. Gateway contract existence
  2. Registry contract existence (if configured)
  3. Subnet actor contract existence
  4. Subnet registration with parent network
  5. Gateway total subnets count
  6. Validator configuration
  7. Genesis epoch status

- **Colored Output**: Uses colored output to clearly indicate success/failure
- **Detailed Reporting**: Provides percentage completion and specific error messages
- **Flexible Configuration**: Supports custom RPC URLs, config files, and IPC CLI paths

### Usage

```bash
# Basic usage (uses defaults)
./scripts/validate-subnet-deployment.sh

# With custom RPC URL
./scripts/validate-subnet-deployment.sh --rpc-url http://custom-rpc:8545

# With custom config file
./scripts/validate-subnet-deployment.sh --config /path/to/custom/config.toml

# With custom IPC CLI binary
./scripts/validate-subnet-deployment.sh --ipc-cli /path/to/ipc-cli

# Show help
./scripts/validate-subnet-deployment.sh --help
```

### Requirements

- **foundry**: For `cast` command (contract interaction)
- **jq**: For JSON parsing
- **ipc-cli**: Built IPC CLI binary (default: `./target/release/ipc-cli`)

### Output Example

```
IPC Subnet Deployment Validation
=================================
Checking dependencies...
✓ All dependencies found
Parsing IPC configuration...
✓ Found 2 configured subnet(s)
  - /r31337
  - /r31337/t410ffn6xkpjrmbnlm5t7dcivmltjgwlkt2yu4ogttii

=== Validating Subnet: /r31337 ===
  Configuration:
    Gateway: 0x95775fd3afb1f4072794ca4dda27f2444bcf8ac3
    Registry: 0xd9fec8238711935d6c8d79bef2b9546ef23fc046
    RPC URL: http://localhost:8545/
  Validation Results:
    ✓ Gateway contract exists
    ✓ Registry contract exists
    ✓ Subnet actor contract exists at 0x...
    ✓ Subnet is registered with parent
    ✓ Gateway has 2 total subnet(s) registered
    ✓ Found 1 validator(s)
    ✓ Genesis epoch: 123
  Summary: 7/7 checks passed (100%)
  ✓ Subnet appears to be fully operational
```

### Troubleshooting

If the script finds issues, see `docs/troubleshooting-subnet-deployment.md` for detailed manual troubleshooting steps.

### Exit Codes

- **0**: All subnets passed validation
- **>0**: Number of total issues found across all subnets