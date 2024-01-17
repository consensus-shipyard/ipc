# Integration test with The Graph

## Prerequisite 
0. Install `npm` if you haven't done so already.
1. Instsall `graph` CLI.
```bash
npm install -g @graphprotocol/graph-cli
```

2. Install `mustache`.
```bash
npm install -g mustache
```

## Run test
```bash
cargo make setup     # Start CometBFT, Fendermint and ETH API
cargo make test      # Run Graph integration test
cargo make teardown
```