# Local Anvil Setup Scripts

This directory contains scripts to help you quickly set up and manage a Local Anvil development environment for IPC subnet development.

## Scripts

### `setup-local-anvil.sh`

Main setup script that handles the complete Local Anvil environment setup.

**Features:**
- ✅ Detects if Anvil is already running
- ✅ Option to use existing Anvil or restart with fresh accounts
- ✅ Uses consistent mnemonic for predictable addresses
- ✅ Imports first 5 accounts into IPC keystore
- ✅ Shows account addresses and balances
- ✅ Colorized output with status indicators

**Usage:**
```bash
./scripts/setup-local-anvil.sh
```

**What it does:**
1. Checks if Anvil is running and offers options
2. Starts Anvil with consistent configuration (if needed)
3. Derives private keys from the standard test mnemonic
4. Imports accounts into `~/.ipc` keystore
5. Shows account balances and next steps

**Requirements:**
- `anvil` (from Foundry)
- `ipc-cli`
- `curl`
- `bc` (basic calculator, usually pre-installed)

### `check-local-anvil.sh`

Quick status checker to see the current state of your Local Anvil setup.

**Usage:**
```bash
./scripts/check-local-anvil.sh
```

**What it checks:**
- ✅ Anvil running status and chain ID
- ✅ IPC keystore account count
- ✅ Contract deployment status
- ✅ Next steps guidance

## Configuration

Both scripts use these default settings:

- **Chain ID:** 31337
- **Port:** 8545
- **Mnemonic:** `test test test test test test test test test test test junk`
- **Accounts:** First 5 accounts imported
- **Host:** 127.0.0.1

## Quick Start

1. **First time setup:**
   ```bash
   ./scripts/setup-local-anvil.sh
   ```

2. **Check status anytime:**
   ```bash
   ./scripts/check-local-anvil.sh
   ```

3. **Reset environment:**
   ```bash
   ./scripts/setup-local-anvil.sh
   # Choose option 2 to restart with fresh accounts
   ```

## Generated Accounts

The script uses the standard test mnemonic and generates these predictable addresses:

1. `0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266` (10000 ETH)
2. `0x70997970C51812dc3A010C7d01b50e0d17dc79C8` (10000 ETH)
3. `0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC` (10000 ETH)
4. `0x90F79bf6EB2c4f870365E785982E1f101E93b906` (10000 ETH)
5. `0x15d34AAf54267DB7D7c367839AAf71A00a2C6A65` (10000 ETH)

## Next Steps After Setup

1. **Open IPC UI:** http://localhost:3000
2. **Select Network:** Choose "Local Anvil" from the network dropdown
3. **Deploy Contracts:** Use the UI to deploy IPC gateway and registry contracts
4. **Create Subnet:** Deploy your first subnet through the UI

## Troubleshooting

**bc calculator missing:**
```bash
# On macOS (usually pre-installed)
brew install bc

# On Ubuntu/Debian
sudo apt-get install bc
```

**Anvil not found:**
```bash
# Install Foundry
curl -L https://foundry.paradigm.xyz | bash
foundryup
```

**IPC CLI not found:**
Make sure you've built the IPC CLI with:
```bash
make build-with-ui
```

**Port already in use:**
The script will automatically detect and offer to restart Anvil if needed.

## Tips

- Keep the terminal running to maintain Anvil
- Use `Ctrl+C` to stop Anvil gracefully
- The mnemonic ensures consistent addresses across restarts
- Accounts have 10,000 ETH each for testing
- The setup is perfect for development and testing