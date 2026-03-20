# IPC Subnet Manager - Relayer & Contract Info Update

## Summary of Changes

This update adds checkpoint relayer support and contract version checking to the IPC subnet manager.

## 1. Configuration Updates (`ipc-subnet-config.yml`)

### Added Child Subnet Contract Configuration
```yaml
ipc_cli:
  child:
    provider_http: "http://127.0.0.1:8545"
    gateway_addr: "0x77aa40b105843728088c0132e43fc44348881da8"
    registry_addr: "0x74539671a1d2f1c8f200826baba665179f53a1b7"
```

### Added Relayer Configuration
```yaml
relayer:
  checkpoint_interval: 10     # Checkpoint interval in seconds
  max_parallelism: 1          # Maximum parallel checkpoint submissions
```

## 2. Config Parser Updates (`lib/config.sh`)

### Updated `generate_ipc_cli_config()`
- Now reads `gateway_addr` and `registry_addr` from `ipc_cli.child` section
- Properly propagates both parent and child contract addresses to `~/.ipc/config.toml`
- Uses `subnet.id` for child subnet ID
- Uses configured `provider_http` URLs for both parent and child

## 3. Relayer Management (`lib/health.sh`)

### New Functions Added

#### `get_validator_address_from_keystore(validator_idx)`
- Extracts the validator's Ethereum address from `~/.ipc/evm_keystore.json`
- Adds `0x` prefix if not present
- Used for the `--submitter` parameter in relayer command

#### `start_relayer()`
- Starts checkpoint relayer on the primary validator
- Command format:
  ```bash
  ipc-cli checkpoint relayer \
    --subnet <subnet_id> \
    --checkpoint-interval-sec <interval> \
    --max-parallelism <parallelism> \
    --submitter <address>
  ```
- Runs in background with nohup
- Logs to `~/.ipc-relayer.log`
- Validates relayer started successfully

#### `stop_relayer()`
- Stops the checkpoint relayer on primary validator
- Uses `ssh_kill_process` to cleanly terminate

#### `check_relayer_status()`
- Checks if relayer is running
- Shows PID if active
- Displays last 20 lines of relayer logs

#### `get_contract_commit_sha(rpc_url, contract_address)`
- Calls the `commitSHA()` function on a contract (selector: `0x66a9f38a`)
- Decodes the bytes32 result to ASCII string
- Returns "N/A" if call fails or no data returned

### Updated `show_subnet_info()`
Added new section at the end that displays contract versions:

```
Contract Versions (commitSHA):
  Parent Contracts (RPC: <parent_rpc>):
    Gateway (<address>): <commitSHA>
    Registry (<address>): <commitSHA>
  Child Contracts (RPC: <child_rpc>):
    Gateway (<address>): <commitSHA>
    Registry (<address>): <commitSHA>
```

## 4. Main Script Updates (`ipc-subnet-manager.sh`)

### New Commands Added

#### `start-relayer`
```bash
./ipc-subnet-manager.sh start-relayer
```
- Starts checkpoint relayer on primary validator
- Automatically extracts submitter address from keystore
- Uses config values for checkpoint interval and parallelism
- Shows log location for monitoring

#### `stop-relayer`
```bash
./ipc-subnet-manager.sh stop-relayer
```
- Stops the running checkpoint relayer

#### `relayer-status`
```bash
./ipc-subnet-manager.sh relayer-status
```
- Checks if relayer is running
- Shows recent relayer activity from logs

## Usage Examples

### Start the Relayer
```bash
# Start relayer on primary validator
./ipc-subnet-manager.sh start-relayer

# Output will show:
# Starting Checkpoint Relayer
# Starting relayer on validator-1 (primary validator)...
# Extracting submitter address from keystore...
# Submitter address: 0x3a86c5fddd2587895965970e70a5fa2ec45ae0ba
# Starting relayer with:
#   Subnet: /r31337/t410f64rg5wfkj3kmbia633bjb4gqcxo7ifhs2e6zuwq
#   Checkpoint interval: 10s
#   Max parallelism: 1
# ✓ Relayer started successfully (PID: 12345)
# Log file: /home/ipc/.ipc-node/logs/relayer.log
# View logs with: ssh philip@34.73.187.192 "sudo su - ipc -c 'tail -f ~/.ipc-node/logs/relayer.log'"
```

### Check Relayer Status
```bash
./ipc-subnet-manager.sh relayer-status

# Output shows:
# Checkpoint Relayer Status
# Checking relayer on validator-1...
# ✓ Relayer is running (PID: 12345)
# Recent relayer activity:
# <last 20 log lines>
```

### Stop the Relayer
```bash
./ipc-subnet-manager.sh stop-relayer
```

### View Contract Versions
```bash
./ipc-subnet-manager.sh info

# Now includes at the end:
# Contract Versions (commitSHA):
#   Parent Contracts (RPC: http://localhost:8555):
#     Gateway (0x0cdd...): abc123def...
#     Registry (0x5efd...): abc123def...
#   Child Contracts (RPC: http://127.0.0.1:8545):
#     Gateway (0x77aa...): abc123def...
#     Registry (0x7453...): abc123def...
```

## Configuration Notes

1. **Child Contract Addresses**: Update `ipc_cli.child.gateway_addr` and `ipc_cli.child.registry_addr` in `ipc-subnet-config.yml` with your actual child subnet contract addresses.

2. **Relayer Settings**: Adjust `relayer.checkpoint_interval` and `relayer.max_parallelism` as needed for your use case.

3. **Provider URLs**:
   - Parent: Uses `ipc_cli.parent.provider_http`
   - Child: Uses `ipc_cli.child.provider_http` (default: `http://127.0.0.1:8545`)

4. **Submitter Address**: The relayer automatically extracts the submitter address from the primary validator's keystore at `~/.ipc/evm_keystore.json`.

## Integration with Init Workflow

The relayer can be manually started after the subnet is initialized using:
```bash
./ipc-subnet-manager.sh init
# Wait for initialization to complete
./ipc-subnet-manager.sh start-relayer
```

## Monitoring

### View Relayer Logs Directly
```bash
# Relayer logs are in the same directory as node logs
ssh philip@<primary-validator-ip> "sudo su - ipc -c 'tail -f ~/.ipc-node/logs/relayer.log'"

# Or from local machine using the script path
tail -f ~/.ipc-node/logs/relayer.log
```

### View Logs via Script
```bash
./ipc-subnet-manager.sh relayer-status
```

## Troubleshooting

### Relayer Won't Start
1. Check if keystore exists: `~/.ipc/evm_keystore.json` on primary validator
2. Verify IPC binary path in config: `paths.ipc_binary`
3. Check if already running: `./ipc-subnet-manager.sh relayer-status`

### Contract CommitSHA Shows "N/A"
1. Verify RPC endpoints are accessible
2. Check contract addresses are correct
3. Ensure contracts implement `commitSHA()` function

### Address Extraction Fails
- Ensure the keystore file exists and is valid JSON
- Check that the validator has been properly initialized with an EVM key

## Files Modified

1. `ipc-subnet-config.yml` - Added child contract config and relayer settings
2. `lib/config.sh` - Updated IPC CLI config generation
3. `lib/health.sh` - Added relayer functions and contract version checking
4. `ipc-subnet-manager.sh` - Added new commands to main script

