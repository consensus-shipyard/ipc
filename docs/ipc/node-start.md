# Node Start Configuration

Starts a complete IPC node with CometBFT consensus, Fendermint application layer, and ETH API.

> **Note:** This command starts all node services after the node has been initialized using `ipc-cli node init`. The node must be properly initialized before running this command.

---

## Usage

```sh
ipc-cli node start --home <path/to/node/home>
```

---

## What This Command Does

The `node start` command performs the following steps:

1. **Validation**: Validates the node home directory structure and required files
2. **Logging Setup**: Initializes comprehensive logging infrastructure
3. **Configuration Loading**: Loads Fendermint settings from the home directory
4. **Service Startup**: Starts all node services including:
   - CometBFT consensus engine
   - Fendermint application layer
   - ETH API server
   - P2P networking

---

## Command Arguments

| Argument | Type     | Required? | Description                                 |
| -------- | -------- | --------- | ------------------------------------------- |
| `--home` | `string` | **Yes**   | Path to the initialized node home directory |

**Example:**

```sh
ipc-cli node start --home ~/.node-ipc
```

---

## Prerequisites

Before running `node start`, ensure that:

1. **Node is Initialized**: The node home directory must be initialized using `ipc-cli node init`
2. **Required Files Exist**: The following files must be present:

   - `fendermint/config/default.toml` - Fendermint configuration
   - `cometbft/config/config.toml` - CometBFT configuration
   - `cometbft/config/genesis.json` - Blockchain genesis
   - Validator private keys and other required files

3. **Directory Structure**: The home directory should contain:
   ```
   <home>/
   ├── fendermint/
   │   ├── config/
   │   │   └── default.toml
   │   └── genesis_*.car
   ├── cometbft/
   │   ├── config/
   │   │   ├── config.toml
   │   │   ├── genesis.json
   │   │   ├── node_key.json
   │   │   └── priv_validator_key.json
   │   └── data/
   └── logs/
   ```

---

## Validation Process

The command validates the following before starting:

### Directory Structure

- Home directory exists and is accessible
- Required subdirectories (`fendermint`, `cometbft`) exist
- Logs directory is writable

### Configuration Files

- Fendermint configuration file exists and is valid
- CometBFT configuration file exists and is valid
- Genesis files exist and are accessible

### Data Files

- Validator keys exist and are properly formatted
- Required data directories are present

### Permissions

- All required files are readable
- Logs directory is writable
- Home directory has appropriate permissions

---

## Logging Infrastructure

The command sets up comprehensive logging with:

### Console Logging

- **Level**: Info
- **Format**: Structured logging with timestamps
- **Output**: Standard output/error

### File Logging

- **Directory**: `<home>/logs/`
- **Level**: Info
- **Rotation**: Daily rotation
- **Retention**: 5 log files maximum
- **Files**:
  - `node.log` - General node logs
  - `fendermint.log` - Fendermint application logs
  - `cometbft.log` - CometBFT consensus logs

### Log Format

```
2024-01-15T10:30:00.123Z INFO  [node_manager] Starting IPC node from home directory: ~/.node-ipc
2024-01-15T10:30:00.124Z INFO  [fendermint] Fendermint settings loaded with home: ~/.node-ipc
2024-01-15T10:30:00.125Z INFO  [cometbft] CometBFT service started successfully
```

---

## Service Architecture

The node runs the following services:

### CometBFT Service

- **Purpose**: Consensus engine and blockchain state management
- **Ports**:
  - P2P: 26656 (default)
  - RPC: 26657 (default)
- **Configuration**: `cometbft/config/config.toml`

### Fendermint Service

- **Purpose**: Application layer and smart contract execution
- **Ports**: Application-specific (configurable)
- **Configuration**: `fendermint/config/default.toml`

### ETH API Service

- **Purpose**: Ethereum-compatible JSON-RPC API
- **Ports**: 8545 (default)
- **Features**: Full Ethereum API compatibility

### Node Manager

- **Purpose**: Orchestrates all services and handles restarts
- **Features**:
  - Service health monitoring
  - Automatic restart on failure
  - Graceful shutdown handling

---

## Startup Sequence

1. **Validation Phase**

   ```
   INFO  [node_start] Validating node home directory structure
   INFO  [node_start] Checking required configuration files
   INFO  [node_start] Validating data files and permissions
   ```

2. **Logging Setup**

   ```
   INFO  [node_start] Initializing tracing infrastructure
   INFO  [node_start] Logs directory: ~/.node-ipc/logs
   ```

3. **Configuration Loading**

   ```
   INFO  [node_start] Loading Fendermint settings
   INFO  [node_start] Fendermint settings loaded successfully
   ```

4. **Service Startup**
   ```
   INFO  [node_manager] Starting CometBFT service
   INFO  [node_manager] Starting Fendermint service
   INFO  [node_manager] Starting ETH API service
   INFO  [node_manager] All services started successfully
   ```

---

## Monitoring and Health Checks

### Service Health

The node manager continuously monitors service health:

- **CometBFT**: Checks RPC endpoint availability
- **Fendermint**: Monitors application state
- **ETH API**: Validates API endpoint responses

### Automatic Restart

If a service fails, the node manager will:

1. **Log the failure** with detailed error information
2. **Attempt restart** with exponential backoff
3. **Continue monitoring** for subsequent failures
4. **Graceful degradation** if restart attempts are exhausted

### Health Check Endpoints

- **CometBFT**: `http://localhost:26657/health`
- **ETH API**: `http://localhost:8545/health`

---

## Stopping the Node

### Graceful Shutdown

To stop the node gracefully:

1. **Send SIGINT/SIGTERM**: `Ctrl+C` or `kill <pid>`
2. **Wait for shutdown**: The node will:
   - Stop accepting new connections
   - Complete in-flight transactions
   - Save state to disk
   - Shut down all services

### Force Shutdown

If graceful shutdown fails:

1. **Send SIGKILL**: `kill -9 <pid>`
2. **Manual cleanup**: May require manual intervention

---

## Troubleshooting

### Common Issues

**Node Not Initialized**

```
Error: Node home directory does not exist: /path/to/home
```

**Solution**: Run `ipc-cli node init` first

**Missing Configuration Files**

```
Error: Required configuration file missing: fendermint/config/default.toml
```

**Solution**: Ensure the node was properly initialized

**Permission Errors**

```
Error: Cannot write to logs directory
```

**Solution**: Check directory permissions and ownership

**Port Conflicts**

```
Error: Address already in use: 26656
```

**Solution**:

- Check if another node is running
- Modify port configuration in `cometbft/config/config.toml`

**Genesis File Issues**

```
Error: Genesis file not found or invalid
```

**Solution**:

- Verify genesis file exists
- Check file permissions
- Ensure genesis file matches subnet configuration

### Debug Information

Enable debug logging for detailed troubleshooting:

```sh
RUST_LOG=debug ipc-cli node start --home /path/to/home
```

### Log Analysis

Check logs for specific issues:

```sh
# Check node logs
tail -f /path/to/home/logs/node.log

# Check Fendermint logs
tail -f /path/to/home/logs/fendermint.log

# Check CometBFT logs
tail -f /path/to/home/cometbft/data/logs/
```

---

## Performance Considerations

### Resource Requirements

- **CPU**: Minimum 2 cores, recommended 4+ cores
- **Memory**: Minimum 4GB RAM, recommended 8GB+ RAM
- **Storage**: SSD recommended for better performance
- **Network**: Stable internet connection for P2P networking

### Optimization Tips

1. **Use SSD storage** for better I/O performance
2. **Allocate sufficient memory** for the JVM (if applicable)
3. **Configure appropriate timeouts** in CometBFT config
4. **Monitor resource usage** during operation
5. **Use production-grade hardware** for mainnet nodes

---

## Security Considerations

### Key Management

- **Secure storage**: Store validator keys securely
- **Access control**: Limit access to node home directory
- **Backup**: Regularly backup configuration and keys
- **Rotation**: Rotate keys periodically

### Network Security

- **Firewall**: Configure firewall rules appropriately
- **P2P ports**: Only expose necessary P2P ports
- **RPC access**: Restrict RPC access to trusted networks
- **TLS**: Use TLS for external connections when possible

### Monitoring

- **Log monitoring**: Monitor logs for suspicious activity
- **Resource monitoring**: Track CPU, memory, and disk usage
- **Network monitoring**: Monitor network connections and traffic
- **Alerting**: Set up alerts for critical issues
