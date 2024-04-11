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
make measurements STATE_HIST_SIZE=0 MEASUREMENTS_PERIOD_SECS=15
```

### Visualize

```bash
make plot-measurements
```

### Print statistics

```console
❯ make stats
{
  "block_height": 4055,
  "db_size_kb": 28444,
  "avg_growth_kb": 5.042747714356313
}
```

### Stop the node

```bash
make teardown
```
