# Filecoin/IPC Real Blockchain Benchmark

## Overview

This benchmark tests the **actual performance** of your Filecoin/IPC subnet by sending **real FIL token transactions** using the native Filecoin transaction format. Unlike simulation-based benchmarks, this measures your subnet's true throughput, latency, and gas consumption.

## Key Features

### ✅ **Real Blockchain Transactions**
- Sends actual FIL token transfers (0.001 FIL per transaction)
- Uses native Filecoin transaction format (`ChainMessage`, `SignedMessage`)
- Connects to your actual Filecoin/IPC node via Tendermint RPC
- Measures real gas consumption and block inclusion

### ✅ **Comprehensive Metrics**
- **Throughput**: Target vs actual TPS, efficiency percentage
- **Latency**: Average, 95th, 99th percentile response times
- **Success Rate**: Transaction success/failure ratio
- **Gas Usage**: Total and per-transaction gas consumption
- **Block Heights**: Real blockchain state changes

### ✅ **Production-Ready**
- Multi-threaded concurrent transaction sending
- Proper sequence number (nonce) management
- Transaction timeouts and error handling
- Real-time progress monitoring
- Detailed JSON reporting

## Usage

### Basic Usage

```bash
# Quick test with your Filecoin testnet node
./filecoin_real_benchmark.rs --endpoint http://your-server:26657

# Custom parameters
./filecoin_real_benchmark.rs \
  --endpoint http://your-server:26657 \
  --target-tps 50 \
  --duration 60 \
  --concurrent-users 20 \
  --output results.json
```

### Command Line Options

| Option | Default | Description |
|--------|---------|-------------|
| `--endpoint` | `http://127.0.0.1:26657` | Your Filecoin/IPC node's Tendermint RPC endpoint |
| `--target-tps` | `100` | Target transactions per second |
| `--duration` | `30` | Test duration in seconds |
| `--concurrent-users` | `10` | Number of concurrent transaction senders |
| `--output` | (none) | Save results to JSON file |
| `--verbose` | `false` | Enable debug logging |

### Example Usage Scenarios

#### 1. **Connect to Your Testnet Node**
```bash
# Replace with your actual server details
./filecoin_real_benchmark.rs \
  --endpoint http://192.168.1.100:26657 \
  --target-tps 25 \
  --duration 30 \
  --concurrent-users 5 \
  --output testnet_results.json
```

#### 2. **High-Throughput Test**
```bash
# Test subnet limits
./filecoin_real_benchmark.rs \
  --endpoint http://your-server:26657 \
  --target-tps 500 \
  --duration 120 \
  --concurrent-users 100 \
  --output high_throughput.json
```

#### 3. **Latency-Focused Test**
```bash
# Lower TPS for latency measurement
./filecoin_real_benchmark.rs \
  --endpoint http://your-server:26657 \
  --target-tps 10 \
  --duration 60 \
  --concurrent-users 1 \
  --output latency_test.json
```

## Prerequisites

### 1. **Filecoin/IPC Node Requirements**
- Running Fendermint node with Tendermint RPC enabled
- Node must be accessible on the network
- RPC endpoint typically on port `26657`

### 2. **Node Configuration**
Your Filecoin/IPC node should have:
- Tendermint RPC enabled (usually `http://node:26657`)
- Sufficient FIL tokens for test transactions
- Proper network configuration

### 3. **Test Wallet Funding**
The benchmark generates test wallets automatically, but they need minimal FIL:
- Each wallet needs ~0.001 FIL per transaction
- Gas fees (minimal on testnets)
- **Note**: The benchmark transfers between self-generated addresses

## Connection Guide

### Finding Your Node Endpoint

1. **Local Node**: `http://127.0.0.1:26657`
2. **Remote Server**: `http://your-server-ip:26657`
3. **Custom Port**: `http://your-server:custom-port`

### Verifying Connection

Test your connection first:
```bash
# Check if node is accessible
curl -X POST -H "Content-Type: application/json" \
  --data '{"jsonrpc":"2.0","method":"status","params":[],"id":1}' \
  http://your-server:26657

# Should return chain info and sync status
```

## Benchmark Output

### Console Output
```
📊 FILECOIN/IPC BLOCKCHAIN BENCHMARK RESULTS
═══════════════════════════════════════════════
🌍 Network: Filecoin/IPC Subnet
🔗 Chain ID: 1234567890
📡 Endpoint: http://your-server:26657
⏱️  Duration: 30.45s
👥 Concurrent Users: 10

🎯 THROUGHPUT METRICS:
   Target TPS: 100
   Actual TPS: 87.32
   Efficiency: 87.3%

📈 TRANSACTION METRICS:
   Total Transactions: 2659
   Successful: 2543
   Failed: 116
   Success Rate: 95.6%

⚡ LATENCY METRICS:
   Average Latency: 234.56ms
   95th Percentile: 456.78ms
   99th Percentile: 678.90ms

⛽ GAS METRICS:
   Total Gas Used: 55,923,000
   Average Gas/TX: 21,024

🏆 PERFORMANCE SCORE: 0.89/1.0
```

### JSON Output
Detailed results including:
- Individual transaction data
- Error logs
- Performance metrics
- Chain state information

## Performance Interpretation

### Success Rate
- **>95%**: Excellent network stability
- **90-95%**: Good performance, some congestion
- **<90%**: Network issues or configuration problems

### Latency
- **<100ms**: Excellent responsiveness
- **100-500ms**: Good for most applications
- **>500ms**: High latency, investigate network

### TPS Efficiency
- **>90%**: Network handling load well
- **70-90%**: Some bottlenecks present
- **<70%**: Significant performance constraints

## Troubleshooting

### Common Issues

#### 1. **Connection Refused**
```
Error: Connection refused (os error 61)
```
**Solution**:
- Verify node is running
- Check endpoint URL and port
- Ensure firewall allows connections

#### 2. **Transaction Failures**
```
Error: TX failed: check=Err(...), deliver=Err(...)
```
**Solution**:
- Check node logs for errors
- Verify gas parameters
- Ensure sufficient network capacity

#### 3. **Timeout Errors**
```
Error: Transaction timeout
```
**Solution**:
- Reduce target TPS
- Increase concurrent users
- Check network latency

### Debug Mode
Enable verbose logging for detailed information:
```bash
./filecoin_real_benchmark.rs --verbose --endpoint http://your-server:26657
```

## Comparison with Previous Benchmarks

| Feature | Previous (Ethereum) | **New (Filecoin/IPC)** |
|---------|---------------------|-------------------------|
| Transaction Type | Simulated ETH transfers | **Real FIL transfers** |
| Network | Ethereum-compatible | **Native Filecoin/IPC** |
| Gas Model | Ethereum gas | **Filecoin gas** |
| Address Format | Ethereum addresses | **Filecoin addresses** |
| Blockchain State | Simulated | **Real state changes** |
| Accuracy | Simulation only | **Production-accurate** |

## Advanced Configuration

### Custom Gas Parameters
Modify the script to adjust gas parameters:
```rust
static ref GAS_PARAMS: GasParams = GasParams {
    gas_limit: 10_000_000_000,    // Adjust for your network
    gas_fee_cap: TokenAmount::from_atto(1000),
    gas_premium: TokenAmount::from_atto(1000),
};
```

### Transaction Amount
Change the FIL transfer amount:
```rust
// Current: 0.001 FIL
let transfer_amount = TokenAmount::from_atto(1_000_000_000_000_000u64);
```

## Integration with Test Scripts

Update your test scripts to use the new benchmark:

```bash
# In run_tests.sh
case "$1" in
    "filecoin")
        ./filecoin_real_benchmark.rs --endpoint "$2" --output results.json
        ;;
    "basic")
        ./filecoin_real_benchmark.rs --endpoint http://127.0.0.1:26657 --target-tps 50 --duration 30
        ;;
esac
```

## Next Steps

1. **Test with your node**: Run a basic test to verify connectivity
2. **Baseline performance**: Establish your subnet's current performance
3. **Stress testing**: Gradually increase load to find limits
4. **Optimization**: Use results to tune your subnet configuration

This benchmark provides the **real performance data** you need to optimize your Filecoin/IPC subnet for production use!