# Anvil with IPC Keys - Quick Reference

## 🚀 Getting Started

```bash
# Start Anvil with all IPC keystore accounts funded
./scripts/setup-anvil-with-ipc-keys.sh

# Or use the quick launcher
./scripts/quick-anvil.sh
```

## 🛠️ Common Commands

### Start/Stop
```bash
# Start (or restart) Anvil
./scripts/setup-anvil-with-ipc-keys.sh

# Stop Anvil
/tmp/stop-anvil-ipc.sh

# Or manually
pkill -f "anvil.*8545"
```

### State Management
```bash
# Save current state (contracts, balances, etc.)
./scripts/anvil-persistent-state.sh save

# Load saved state
./scripts/anvil-persistent-state.sh load

# Check state info
./scripts/anvil-persistent-state.sh info
```

### Account Management
```bash
# List all IPC accounts
ipc-cli wallet list --keystore-path ~/.ipc

# Import new account
echo "YOUR_PRIVATE_KEY" | ipc-cli wallet import --keystore-path ~/.ipc

# Set default account
ipc-cli wallet set-default --address 0xYOUR_ADDRESS --keystore-path ~/.ipc

# View keystore
cat ~/.ipc/evm_keystore.json
```

### Check Balances
```bash
# View all funded accounts (after running setup)
./scripts/setup-anvil-with-ipc-keys.sh
# (choose option 1 if Anvil is running)

# Check specific account balance
curl -X POST -H "Content-Type: application/json" \
  --data '{"jsonrpc":"2.0","method":"eth_getBalance","params":["0xYOUR_ADDRESS","latest"],"id":1}' \
  http://localhost:8545
```

## 📊 Network Details

| Setting | Value |
|---------|-------|
| RPC URL | `http://localhost:8545` |
| Chain ID | `31337` |
| Block Time | 1 second |
| Default Funding | 10,000 ETH per account |

## 🔧 Convenience Aliases

```bash
# Load aliases into your shell
source scripts/aliases.sh

# Then use shortcuts:
anvil-start      # Start Anvil
anvil-stop       # Stop Anvil
anvil-save       # Save state
anvil-load       # Load state
anvil-balance    # Check balance
ipc              # IPC CLI with keystore
```

## 🐛 Troubleshooting

### Anvil won't start
```bash
# Check if port is in use
lsof -i :8545

# Kill any existing Anvil
pkill -f anvil

# Try again
./scripts/setup-anvil-with-ipc-keys.sh
```

### Accounts not funded
```bash
# Verify keystore exists
cat ~/.ipc/evm_keystore.json

# Check Anvil is responding
curl http://localhost:8545

# Re-run setup
./scripts/setup-anvil-with-ipc-keys.sh
```

### Missing dependencies
```bash
# Install Foundry (includes Anvil)
curl -L https://foundry.paradigm.xyz | bash
foundryup

# Install bc (for calculations)
# macOS:
brew install bc
# Linux:
sudo apt-get install bc
```

## 📁 Files & Locations

| Purpose | Location |
|---------|----------|
| IPC Keystore | `~/.ipc/evm_keystore.json` |
| Anvil Logs | `/tmp/anvil_ipc_keys.log` |
| Anvil PID | `/tmp/anvil_ipc.pid` |
| Stop Script | `/tmp/stop-anvil-ipc.sh` |
| Saved State | `~/.ipc/anvil-state/state.json` |
| Setup Script | `./scripts/setup-anvil-with-ipc-keys.sh` |

## 🔄 Typical Workflow

### 1. Initial Setup
```bash
# Start Anvil with funded accounts
./scripts/setup-anvil-with-ipc-keys.sh
```

### 2. Development
```bash
# Deploy contracts using IPC CLI
ipc-cli subnet create --parent /r31337 --name my-subnet --keystore-path ~/.ipc

# Run your tests, deploy contracts, etc.
```

### 3. Save Progress
```bash
# Save state before stopping
./scripts/anvil-persistent-state.sh save

# Stop Anvil
/tmp/stop-anvil-ipc.sh
```

### 4. Resume Later
```bash
# Start Anvil again
./scripts/setup-anvil-with-ipc-keys.sh

# Load previous state
./scripts/anvil-persistent-state.sh load
```

## 🎯 Use Cases

### Fresh Testing Environment
```bash
# Stop existing Anvil
/tmp/stop-anvil-ipc.sh

# Start fresh
./scripts/setup-anvil-with-ipc-keys.sh
# Choose option 2 (Restart)
```

### Add New Account
```bash
# Import new key to IPC keystore
echo "NEW_PRIVATE_KEY" | ipc-cli wallet import --keystore-path ~/.ipc

# Restart Anvil setup (it will fund the new account)
./scripts/setup-anvil-with-ipc-keys.sh
# Choose option 2 (Restart)
```

### Fund Additional Accounts
```bash
# Run setup with existing Anvil
./scripts/setup-anvil-with-ipc-keys.sh
# Choose option 1 (Use existing)
# Any new accounts will be funded
```

## 📖 Full Documentation

For comprehensive documentation, see:
- [ANVIL_IPC_SETUP.md](./ANVIL_IPC_SETUP.md) - Complete guide
- [README.md](../README.md) - Main IPC documentation

## 💡 Tips

1. **Keep Anvil running** during development to maintain contract state
2. **Save state frequently** if you're deploying important test contracts
3. **Use aliases** for faster workflow (`source scripts/aliases.sh`)
4. **Check logs** if something goes wrong (`cat /tmp/anvil_ipc_keys.log`)
5. **Restart fresh** between major test runs to avoid state conflicts

## 🔗 Related Commands

### IPC Subnet Operations
```bash
# Create subnet
ipc-cli subnet create --parent /r31337 --name test --keystore-path ~/.ipc

# Join subnet
ipc-cli subnet join --subnet /r31337/t... --keystore-path ~/.ipc

# List subnets
ipc-cli subnet list --keystore-path ~/.ipc
```

### Fund Transfer
```bash
# Send funds using cast (Foundry)
cast send 0xTO_ADDRESS --value 1ether --private-key 0xYOUR_KEY --rpc-url http://localhost:8545
```

---

**Need help?** Join **#ipc-help** in [Filecoin Slack](https://filecoin.io/slack)

