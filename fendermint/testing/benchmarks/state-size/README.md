# State Size Benchmark

The purpose of this benchmark is to establish a baseline for state growth.
The idea is to run a simple standalone node and measure the size of the RocksDB data store as the block height goes up.


## Take measurements

Let's use the [materializer](../../materializer/) to start a node.

The following commands is assumed to be executed in this directory.


### Start a node

```bash
make setup STATE_HIST_SIZE=0
```


### Measure the database size

```bash
make measurements MEASUREMENTS_PERIOD_SECS=15
```


### Check the block height

```bash
PORT_FROM=$(cat $MATERIALIZER_DIR/materializer-state.json | jq ".port_ranges.\"testnets/$TESTNET_ID/root/nodes/$NODE_ID\".from")
```


### Stop the node

```bash
make teardown
```
