# Anvil Setup with IPC Keystore Integration

This guide explains how to configure Anvil to use your IPC keystore keys with consistent funding across restarts.

## Overview

The IPC project includes scripts to automatically configure Anvil (local Ethereum node) to:
- Use deterministic accounts
- Fund all accounts from your IPC keystore (`~/.ipc/evm_keystore.json`)
- Maintain consistent state across restarts
- Simplify local development and testing

## Quick Start

### 1. Start Anvil with IPC Keys

```bash
./scripts/setup-anvil-with-ipc-keys.sh
```

This script will:
- ✅ Start Anvil on port 8545 with chain ID 31337
- ✅ Read all addresses from your IPC keystore
- ✅ Fund each address with 10,000 ETH
- ✅ Create management scripts for easy control

### 2. Verify Setup

After running the script, you should see:
- List of all funded accounts with their balances
- RPC URL and chain ID information
- Path to stop script and logs

## Scripts Reference

### `setup-anvil-with-ipc-keys.sh`

**Primary setup script** - Starts Anvil and funds all IPC keystore accounts.

```bash
./scripts/setup-anvil-with-ipc-keys.sh
```

**Features:**
- Automatically detects existing Anvil instances
- Funds all addresses from `~/.ipc/evm_keystore.json`
- Skips already-funded accounts
- Creates stop script at `/tmp/stop-anvil-ipc.sh`
- Saves PID for easy management

**Configuration:**
- Default port: 8545
- Chain ID: 31337
- Initial balance per account: 10,000 ETH
- Mnemonic: Standard test mnemonic

### `anvil-persistent-state.sh`

**State management script** - Save and restore Anvil state across restarts.

```bash
# Save current state
./scripts/anvil-persistent-state.sh save

# Load saved state (after restarting Anvil)
./scripts/anvil-persistent-state.sh load

# Show state information
./scripts/anvil-persistent-state.sh info

# Generate list of funded accounts
./scripts/anvil-persistent-state.sh list
```

**Use Cases:**
- Preserve contract deployments across restarts
- Save funded account balances
- Maintain test data between sessions

**State Location:** `~/.ipc/anvil-state/`

## Managing Anvil

### Start Anvil
```bash
./scripts/setup-anvil-with-ipc-keys.sh
```

### Stop Anvil
```bash
/tmp/stop-anvil-ipc.sh
```

Or manually:
```bash
pkill -f "anvil.*8545"
```

### View Logs
```bash
cat /tmp/anvil_ipc_keys.log
```

### Check Status
```bash
curl -X POST -H "Content-Type: application/json" \
  --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
  http://localhost:8545
```

## Configuration Options

You can customize the setup by editing `setup-anvil-with-ipc-keys.sh`:

```bash
# Configuration section (lines 15-25)
ANVIL_PORT=8545              # Change port
ANVIL_CHAIN_ID=31337         # Change chain ID
INITIAL_BALANCE="10000"       # Change funding amount (ETH)
IPC_KEYSTORE="$HOME/.ipc/evm_keystore.json"  # Keystore location
```

## Working with IPC CLI

Once Anvil is running with funded accounts, use IPC CLI:

```bash
# Set default wallet
ipc-cli wallet set-default --address 0xYOUR_ADDRESS --keystore-path ~/.ipc

# Deploy contracts
ipc-cli subnet create --parent /r31337 --name my-subnet --keystore-path ~/.ipc

# Check subnet status
ipc-cli subnet list --keystore-path ~/.ipc --subnet /r31337
```

## Keystore Format

The IPC keystore (`~/.ipc/evm_keystore.json`) is a JSON file:

```json
[
  {
    "address": "0x3a86c5fddd2587895965970e70a5fa2ec45ae0ba",
    "private_key": "867c766fa9ea9fab8929a6ec6a4fe32ccf33969035d3d7f2262f6eb8021b56d8"
  },
  {
    "address": "0x116939fd3d36122a978f5d3e000f7223c6194bf7",
    "private_key": "fea0774f59da29837584ae080f5e823ac2691766054846aebaff27615f8149fb"
  }
]
```

**Note:** The script automatically skips the `"default-key"` entry.

## Workflows

### Development Workflow

1. **Initial Setup**
   ```bash
   ./scripts/setup-anvil-with-ipc-keys.sh
   ```

2. **Deploy and Test**
   - Deploy contracts using IPC CLI
   - Run tests
   - Develop features

3. **Save State** (optional)
   ```bash
   ./scripts/anvil-persistent-state.sh save
   ```

4. **Stop Anvil**
   ```bash
   /tmp/stop-anvil-ipc.sh
   ```

### Restore Previous State

1. **Start Anvil**
   ```bash
   ./scripts/setup-anvil-with-ipc-keys.sh
   ```

2. **Load Saved State**
   ```bash
   ./scripts/anvil-persistent-state.sh load
   ```

### Adding New Accounts

1. **Import to IPC Keystore**
   ```bash
   echo "YOUR_PRIVATE_KEY" | ipc-cli wallet import --keystore-path ~/.ipc
   ```

2. **Restart Anvil Setup**
   ```bash
   ./scripts/setup-anvil-with-ipc-keys.sh
   ```

   Choose option 2 (Restart Anvil and fund accounts)

## Troubleshooting

### Anvil won't start
```bash
# Check if port is already in use
lsof -i :8545

# Kill existing Anvil
pkill -f anvil

# Try again
./scripts/setup-anvil-with-ipc-keys.sh
```

### Accounts not funded
```bash
# Check if keystore exists
cat ~/.ipc/evm_keystore.json

# Verify Anvil is running
curl http://localhost:8545

# Check account balance manually
curl -X POST -H "Content-Type: application/json" \
  --data '{"jsonrpc":"2.0","method":"eth_getBalance","params":["0xYOUR_ADDRESS","latest"],"id":1}' \
  http://localhost:8545
```

### State not loading
```bash
# Check if state file exists
ls -lh ~/.ipc/anvil-state/

# Verify Anvil RPC is responding
./scripts/anvil-persistent-state.sh info

# Try dumping fresh state
./scripts/anvil-persistent-state.sh save
```

### Dependencies missing
```bash
# Install Foundry (includes Anvil)
curl -L https://foundry.paradigm.xyz | bash
foundryup

# Install bc (for balance calculations)
# macOS:
brew install bc

# Linux:
sudo apt-get install bc
```

## Advanced Usage

### Custom Funding Amounts

Edit the script to fund different amounts per account:

```bash
# In setup-anvil-with-ipc-keys.sh, modify:
INITIAL_BALANCE="10000"  # Change to desired ETH amount
```

### Multiple Anvil Instances

Run multiple Anvil instances with different configurations:

```bash
# Copy and modify the script
cp scripts/setup-anvil-with-ipc-keys.sh scripts/setup-anvil-8546.sh

# Edit the copy to use different port
# Change: ANVIL_PORT=8546
# Change: ANVIL_CHAIN_ID=31338

./scripts/setup-anvil-8546.sh
```

### Integration with Docker

Create a Dockerfile:

```dockerfile
FROM ghcr.io/foundry-rs/foundry:latest
COPY scripts/setup-anvil-with-ipc-keys.sh /setup.sh
COPY ~/.ipc/evm_keystore.json /root/.ipc/evm_keystore.json
CMD ["/setup.sh"]
```

## Network Information

**Default Configuration:**
- **RPC URL:** `http://localhost:8545`
- **Chain ID:** `31337`
- **Network Name:** Local Anvil
- **Currency:** ETH
- **Block Time:** 1 second
- **Gas Limit:** 30,000,000
- **Gas Price:** 1 gwei

## Security Notes

⚠️ **Important Security Considerations:**

1. **Never use these keys on mainnet or production networks**
2. The keystore is stored in plain text at `~/.ipc/evm_keystore.json`
3. Private keys are exposed during funding operations
4. Only use this setup for local development
5. Clear your keystore when switching between projects

## Contributing

If you find issues or have suggestions for these scripts:
1. Open an issue in the IPC repository
2. Submit a pull request with improvements
3. Share your custom configurations

## Related Documentation

- [IPC Documentation](../docs/ipc/)
- [Fendermint Setup](../docs/fendermint/)
- [Anvil Documentation](https://book.getfoundry.sh/anvil/)
- [IPC CLI Usage](../docs/ipc/usage.md)

